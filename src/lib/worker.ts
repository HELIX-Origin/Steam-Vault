//! Global Background Worker & Job Queue (`src/lib/worker.ts`)
//!
//! Ensures ALL heavy data fetching, platform scanning, ROM extraction, and metadata
//! enhancement execute asynchronously in the background without ever hanging or
//! locking up the UI thread.
//!
//! Features:
//! - Deduplicates identical in-flight tasks by key (`scan-platforms`, `refresh-meta:<id>`, etc.)
//! - Enforces a strict safety timeout on every background job so a hung network call
//!   never leaves a button or spinner stuck forever
//! - Publishes reactive status (`activeJobs`, `workerBusy`, `workerStatusText`) for non-blocking UI feedback

import { writable, derived } from "svelte/store";
import { api } from "./api";
import { pluginEngine } from "./plugins";
import { games, platformScan, selectedGameId, error } from "./stores";
import type { AppMeta, Game } from "./types";

export interface BackgroundJob {
  id: string;
  label: string;
  startedAt: number;
}

const activeJobsMap = writable<Map<string, BackgroundJob>>(new Map());
const inFlightPromises = new Map<string, Promise<any>>();

/** Reactive list of currently running background jobs. */
export const activeWorkerJobs = derived(activeJobsMap, ($map) => Array.from($map.values()));

/** Reactive boolean indicating whether the global background worker has any active tasks. */
export const workerBusy = derived(activeJobsMap, ($map) => $map.size > 0);

/** Reactive status text for the most recent active background job. */
export const workerStatusText = derived(activeJobsMap, ($map) => {
  const list = Array.from($map.values());
  return list.length > 0 ? list[list.length - 1].label : null;
});

/**
 * Wrap any promise with an upper-bound timeout so no network or IPC call can ever hang indefinitely.
 */
export function withTimeout<T>(promise: Promise<T>, ms: number, fallbackValue: T): Promise<T> {
  return new Promise<T>((resolve) => {
    const timer = setTimeout(() => {
      resolve(fallbackValue);
    }, ms);

    promise
      .then((val) => {
        clearTimeout(timer);
        resolve(val);
      })
      .catch(() => {
        clearTimeout(timer);
        resolve(fallbackValue);
      });
  });
}

class GlobalBackgroundWorker {
  /**
   * Enqueues and executes an asynchronous task on the global background worker.
   * If a job with the same `id` is already in-flight, returns the existing promise.
   */
  async enqueue<T>(
    id: string,
    label: string,
    taskFn: () => Promise<T>,
    timeoutMs = 25000
  ): Promise<T> {
    const existing = inFlightPromises.get(id);
    if (existing) {
      return existing as Promise<T>;
    }

    activeJobsMap.update((m) => {
      const next = new Map(m);
      next.set(id, { id, label, startedAt: Date.now() });
      return next;
    });

    // Yield to the browser's next frame before starting so UI click animations render at 60fps
    const promise = (async () => {
      await new Promise((r) => setTimeout(r, 0));
      try {
        return await Promise.race([
          taskFn(),
          new Promise<T>((_, reject) =>
            setTimeout(() => reject(new Error(`Task '${label}' timed out after ${timeoutMs}ms`)), timeoutMs)
          ),
        ]);
      } finally {
        inFlightPromises.delete(id);
        activeJobsMap.update((m) => {
          const next = new Map(m);
          next.delete(id);
          return next;
        });
      }
    })();

    inFlightPromises.set(id, promise);
    return promise;
  }

  /**
   * Fire-and-forget a background task without awaiting its completion in the UI handler.
   */
  dispatch(id: string, label: string, taskFn: () => Promise<any>, timeoutMs = 25000): void {
    this.enqueue(id, label, taskFn, timeoutMs).catch((err) => {
      console.warn(`[BackgroundWorker] Job '${id}' warning:`, err);
    });
  }

  /**
   * Global non-blocking platform & ROM scan.
   * Immediately updates the library as soon as local directories and ROM archives are scanned,
   * then warms store metadata in a separate background pass without holding up the scan button.
   */
  async scanAndRefreshLibrary(options?: {
    onProgress?: (msg: string) => void;
    currentSelectedId?: number | null;
  }): Promise<Game[]> {
    return this.enqueue(
      "global-platform-scan",
      "Scanning platforms & ROMs…",
      async () => {
        options?.onProgress?.("Scanning platforms & ROM directories…");

        // 1. Scan all platforms on the Rust blocking worker pool
        const scan = await withTimeout(api.scanAllPlatforms(true), 20000, null);
        if (scan) {
          platformScan.set(scan);
          if (scan.games && scan.games.length > 0) {
            await withTimeout(api.importPlatformGames(scan.games), 10000, null);
          }
        }

        // 2. If user configured a custom ROMs path in Universal Platform plugin, scan it too
        try {
          const uniCfg = await pluginEngine.loadPluginConfig("platform-universal");
          const esCfg = await pluginEngine.loadPluginConfig("platform-emulationstation");
          const customPath = (uniCfg?.romsPath || esCfg?.romsPath || "").trim();
          if (customPath && customPath !== "C:\\ROMs") {
            const normalize = (uniCfg?.normalizeTitles ?? esCfg?.normalizeTitles) !== false;
            const roms = await withTimeout(api.scanEmulationRoms(customPath, normalize), 20000, []);
            if (roms.length > 0) {
              await withTimeout(api.importPlatformGames(roms), 10000, null);
            }
          }
        } catch (esErr) {
          console.warn("Custom EmulationStation scan warning:", esErr);
        }

        // 3. Reload library immediately so games appear in the UI right away
        const updated = await withTimeout(api.loadLibrary(undefined, true), 10000, []);
        if (updated.length > 0) {
          games.set(updated);
          if (!options?.currentSelectedId) {
            selectedGameId.set(updated[0].appId);
          }
        }

        // 4. Fire-and-forget background metadata warming so the scan button finishes immediately!
        this.dispatch(
          "background-metadata-sync",
          "Syncing library metadata in background…",
          async () => {
            await api.syncLibraryMetadata(undefined, false, true);
            const refreshedLib = await api.loadLibrary(undefined, true);
            if (refreshedLib.length > 0) {
              games.set(refreshedLib);
            }
          },
          45000
        );

        return updated;
      },
      30000
    );
  }

  /**
   * Non-blocking single-game metadata fetch & enhancement.
   * Runs backend store/local lookup and frontend metadata plugins with strict per-plugin timeouts
   * so no slow provider can ever hang the UI.
   */
  async enhanceGameMetadata(
    game: Game,
    currentMeta: AppMeta | null,
    options?: { providerId?: string; override?: boolean }
  ): Promise<{ meta: AppMeta; enhancedBy: string[] }> {
    return this.enqueue(
      `enhance-meta:${game.appId}`,
      `Enhancing metadata for ${game.name}…`,
      async () => {
        const shouldOverride = options?.override ?? false;
        const targetProvider =
          options?.providerId && options.providerId !== "all" ? options.providerId : undefined;

        let baseMeta = currentMeta;

        // 1. Run backend Universal Metadata Engine & Local Inspector (with 8s max timeout)
        if (!targetProvider || targetProvider === "metadata-universal" || targetProvider === "metadata-steam") {
          const backendMeta = await withTimeout(
            api.refreshGameMetadata(game.appId, shouldOverride),
            8000,
            null
          );
          if (backendMeta) {
            baseMeta = backendMeta;
          }
        }

        // 2. Run frontend Universal Metadata Plugin (with 10s max timeout)
        const fallbackResult = {
          meta:
            baseMeta ??
            ({
              appId: game.appId,
              name: game.name,
              developers: [],
              publishers: [],
              genres: [],
              categories: [],
              tags: [],
              releaseDate: null,
              releaseYear: null,
              shortDescription: null,
              headerImage: null,
              capsuleImage: null,
              heroImage: null,
              logoImage: null,
              iconImage: null,
              userScore: null,
              fetchedAt: Date.now(),
              screenshots: [],
              movies: [],
              soundtracks: [],
            } as AppMeta),
          enhancedBy: [] as string[],
        };

        const { meta, enhancedBy } = await withTimeout(
          pluginEngine.enhanceGameMetadata(baseMeta, game, {
            providerId: targetProvider,
            override: shouldOverride,
          }),
          10000,
          fallbackResult
        );

        if (
          !enhancedBy.length &&
          baseMeta &&
          (baseMeta.shortDescription || baseMeta.heroImage || baseMeta.capsuleImage || baseMeta.developers?.length)
        ) {
          enhancedBy.push("Universal Metadata Engine");
        }

        // Update library list item artwork in store if enhanced
        if (meta.capsuleImage || meta.headerImage || meta.heroImage) {
          games.update((list) =>
            list.map((g) =>
              g.appId === game.appId
                ? {
                    ...g,
                    capsuleImage: meta.capsuleImage || meta.headerImage || g.capsuleImage,
                    heroImage: meta.heroImage || g.heroImage,
                  }
                : g
            )
          );
        }

        return { meta, enhancedBy };
      },
      15000
    );
  }

  /**
   * Non-blocking single-game metadata refresh from provider source.
   */
  async refreshGameMetadata(game: Game, currentMeta: AppMeta | null, isReset = false): Promise<AppMeta | null> {
    return this.enqueue(
      `refresh-meta:${game.appId}`,
      `Refreshing metadata for ${game.name}…`,
      async () => {
        let meta = currentMeta;

        // Run provider data fetch and backend refresh concurrently with strict 8s timeouts
        const [providerData, refreshedMeta] = await Promise.all([
          withTimeout(pluginEngine.fetchProviderGameData(game), 6000, null),
          withTimeout(api.refreshGameMetadata(game.appId, isReset), 8000, null),
        ]);

        if (refreshedMeta) {
          meta = refreshedMeta;
        }
        if (providerData) {
          meta = pluginEngine.mergeProviderData(meta, providerData, game, isReset);
        }
        if (meta) {
          await withTimeout(api.kvSetJson(`game_meta:${game.appId}`, meta), 3000, false);
        }
        return meta;
      },
      12000
    );
  }
}

export const backgroundWorker = new GlobalBackgroundWorker();
