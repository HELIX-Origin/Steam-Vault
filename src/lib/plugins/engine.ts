import { ALL_PLUGINS } from "./catalog";
import type { UniversalPlugin, TestConnectionResult, PluginMetadataResult, PluginArtworkResult } from "./framework";
import type { PluginInfo, PluginSettingsConfig, AppMeta } from "$lib/types";
import { api } from "$lib/api";

class PluginEngineManager {
  private registry = new Map<string, UniversalPlugin>();

  constructor() {
    this.ensureInitialized();
  }

  private ensureInitialized() {
    if (this.registry.size === 0 && Array.isArray(ALL_PLUGINS)) {
      for (const plugin of ALL_PLUGINS) {
        this.registry.set(plugin.id, plugin);
      }
    }
  }

  /** Retrieve all in-memory native plugins. */
  getAllPlugins(): UniversalPlugin[] {
    this.ensureInitialized();
    return Array.from(this.registry.values());
  }

  /** Lookup a plugin by its unique identifier. */
  getPlugin(id: string): UniversalPlugin | undefined {
    this.ensureInitialized();
    return this.registry.get(id);
  }

  /**
   * Retrieves the settings configuration schema for a plugin.
   */
  getPluginSettingsSchema(id: string): PluginSettingsConfig | null {
    const plugin = this.registry.get(id);
    if (plugin?.getSettingsSchema) {
      return plugin.getSettingsSchema();
    }
    return null;
  }

  /**
   * Tests an API connection or credential validity for a plugin.
   */
  async testPluginConnection(id: string, config: Record<string, any>): Promise<TestConnectionResult> {
    const plugin = this.registry.get(id);
    if (plugin?.testConnection) {
      return await plugin.testConnection(config);
    }
    return { success: true, message: "No verification required for this plugin." };
  }

  /**
   * Loads saved settings for a specific plugin from persistent in-memory KV / SQLite.
   * Default values are populated directly from the plugin's schema fields.
   */
  async loadPluginConfig(id: string): Promise<Record<string, any>> {
    const key = `plugin_cfg:${id}`;
    const stored = await api.kvGetJson<Record<string, any>>(key);
    const defaults: Record<string, any> = {};

    const schema = this.getPluginSettingsSchema(id);
    if (schema?.fields) {
      for (const field of schema.fields) {
        if (field.default !== undefined) {
          defaults[field.key] = field.default;
        }
      }
    }

    if (stored && Object.keys(stored).length > 0) {
      return { ...defaults, ...stored };
    }

    return defaults;
  }

  /**
   * Saves settings for a specific plugin directly to persistent KV / SQLite.
   */
  async savePluginConfig(id: string, config: Record<string, any>): Promise<boolean> {
    const key = `plugin_cfg:${id}`;
    await api.kvSetJson(key, config);
    return true;
  }

  /**
   * Combines registered native plugins with active enabled flags.
   */
  mergePluginInfo(backendPlugins: PluginInfo[]): PluginInfo[] {
    this.ensureInitialized();
    const enabledMap = new Map<string, boolean>();
    for (const bp of backendPlugins) {
      enabledMap.set(bp.id, bp.enabled);
    }

    return Array.from(this.registry.values()).map(plugin => {
      const schema = plugin.getSettingsSchema ? plugin.getSettingsSchema() : undefined;
      const isEnabled = enabledMap.has(plugin.id) ? enabledMap.get(plugin.id)! : true;

      return {
        id: plugin.id,
        name: plugin.name,
        version: plugin.version,
        pluginType: plugin.pluginType,
        author: plugin.author ?? "HELIX Origin",
        description: plugin.description ?? null,
        enabled: isEnabled,
        builtin: true,
        icon: plugin.icon ?? null,
        homepage: plugin.homepage ?? null,
        features: plugin.features ?? [],
        path: `native://src/lib/plugins/${plugin.pluginType === "platform" ? "platforms" : plugin.pluginType}/${plugin.id}`,
        settings: schema
      };
    });
  }

  /**
   * Queries metadata across enabled metadata plugins.
   */
  async queryMetadata(
    query: { appId?: number | string; name?: string; platform?: string },
    enabledPlugins?: Record<string, boolean>
  ): Promise<PluginMetadataResult | null> {
    this.ensureInitialized();
    const metadataPlugins = Array.from(this.registry.values()).filter(
      p => p.pluginType === "metadata" && (enabledPlugins ? enabledPlugins[p.id] !== false : true)
    );

    for (const plugin of metadataPlugins) {
      if (!plugin.queryMetadata) continue;
      try {
        const config = await this.loadPluginConfig(plugin.id);
        const result = await plugin.queryMetadata(query, config);
        if (result && (result.name || result.shortDescription || result.releaseDate || result.genres?.length)) {
          return result;
        }
      } catch (e) {
        console.warn(`Plugin ${plugin.id} queryMetadata failed:`, e);
      }
    }
    return null;
  }

  /**
   * Queries artwork across enabled metadata plugins.
   */
  async queryArtwork(
    query: { appId?: number | string; name?: string; platform?: string },
    enabledPlugins?: Record<string, boolean>
  ): Promise<PluginArtworkResult | null> {
    this.ensureInitialized();
    const artworkPlugins = Array.from(this.registry.values()).filter(
      p => p.pluginType === "metadata" && (enabledPlugins ? enabledPlugins[p.id] !== false : true)
    );

    for (const plugin of artworkPlugins) {
      if (!plugin.queryArtwork) continue;
      try {
        const config = await this.loadPluginConfig(plugin.id);
        const result = await plugin.queryArtwork(query, config);
        if (result && (result.capsuleImage || result.heroImage || result.logoImage || result.iconImage)) {
          return result;
        }
      } catch (e) {
        console.warn(`Plugin ${plugin.id} queryArtwork failed:`, e);
      }
    }
    return null;
  }

  /**
   * Look up the platform/provider plugin responsible for a given platform identifier.
   */
  getProviderPlugin(platformId?: string | null): UniversalPlugin | undefined {
    this.ensureInitialized();
    if (!platformId) return undefined;
    const norm = platformId.toLowerCase().trim();
    return Array.from(this.registry.values()).find(p => {
      if (p.pluginType !== "platform") return false;
      if (p.platformId && p.platformId.toLowerCase() === norm) return true;
      if (p.id === `platform-${norm}`) return true;
      if ((norm === "other" || norm === "custom") && p.id === "platform-standalone") return true;
      return false;
    });
  }

  /**
   * Fetches game data directly from the game's native platform provider plugin.
   * Game providers are the default, authoritative source for games in the library.
   */
  async fetchProviderGameData(
    game: { appId?: number | string; name?: string; platform?: string | null; console?: string | null; exe?: string | null; installPath?: string | null }
  ): Promise<PluginMetadataResult | null> {
    this.ensureInitialized();
    const provider = this.getProviderPlugin(game.platform);
    if (!provider || !provider.queryMetadata) return null;

    try {
      const config = await this.loadPluginConfig(provider.id);
      const meta = await provider.queryMetadata(
        {
          appId: game.appId,
          name: game.name,
          platform: (game.console || game.platform) ?? undefined,
          platformId: game.platform ?? undefined
        },
        config
      );

      if (meta && provider.queryArtwork) {
        try {
          const artwork = await provider.queryArtwork(
            {
              appId: game.appId,
              name: game.name,
              platform: (game.console || game.platform) ?? undefined,
              platformId: game.platform ?? undefined
            },
            config
          );
          if (artwork) {
            meta.capsuleImage = meta.capsuleImage || artwork.capsuleImage || undefined;
            meta.heroImage = meta.heroImage || artwork.heroImage || undefined;
            meta.logoImage = meta.logoImage || artwork.logoImage || undefined;
            meta.iconImage = meta.iconImage || artwork.iconImage || undefined;
          }
        } catch (e) {
          console.warn(`Provider ${provider.id} queryArtwork failed:`, e);
        }
      }

      return meta;
    } catch (e) {
      console.warn(`Provider ${provider.id} fetchProviderGameData failed:`, e);
      return null;
    }
  }

  /**
   * Converts provider metadata into an AppMeta record and merges with existing metadata.
   * INVARIANT: Game providers should ONLY be allowed to override the metadata from
   * metadata providers if the user chooses to reset the metadata cache (isReset = true).
   */
  mergeProviderData(
    existing: AppMeta | null,
    providerMeta: PluginMetadataResult,
    game: { appId: number; name: string },
    isReset = false
  ): AppMeta {
    const isEnhanced = existing?.userEnhanced === true && !isReset;

    const base: AppMeta = {
      appId: Number(game.appId),
      name: isEnhanced && existing?.name ? existing.name : (providerMeta.name || existing?.name || game.name),
      developers: isEnhanced && existing?.developers?.length ? existing.developers : (providerMeta.developers || existing?.developers || []),
      publishers: isEnhanced && existing?.publishers?.length ? existing.publishers : (providerMeta.publishers || existing?.publishers || []),
      genres: isEnhanced && existing?.genres?.length ? existing.genres : (providerMeta.genres || existing?.genres || []),
      categories: isEnhanced && existing?.categories?.length ? existing.categories : (providerMeta.categories || existing?.categories || []),
      tags: isEnhanced && existing?.tags?.length ? existing.tags : (providerMeta.tags || existing?.tags || []),
      releaseDate: isEnhanced && existing?.releaseDate ? existing.releaseDate : (providerMeta.releaseDate || existing?.releaseDate || null),
      releaseYear: isEnhanced && existing?.releaseYear ? existing.releaseYear : (providerMeta.releaseYear || existing?.releaseYear || null),
      shortDescription: isEnhanced && existing?.shortDescription ? existing.shortDescription : (providerMeta.shortDescription || existing?.shortDescription || null),
      headerImage: isEnhanced && existing?.headerImage ? existing.headerImage : (providerMeta.headerImage || existing?.headerImage || null),
      capsuleImage: isEnhanced && existing?.capsuleImage ? existing.capsuleImage : (providerMeta.capsuleImage || existing?.capsuleImage || null),
      heroImage: isEnhanced && existing?.heroImage ? existing.heroImage : (providerMeta.heroImage || existing?.heroImage || null),
      logoImage: isEnhanced && existing?.logoImage ? existing.logoImage : (providerMeta.logoImage || existing?.logoImage || null),
      iconImage: isEnhanced && existing?.iconImage ? existing.iconImage : (providerMeta.iconImage || existing?.iconImage || null),
      userScore: isEnhanced && existing?.userScore != null ? existing.userScore : (providerMeta.rating != null ? providerMeta.rating : existing?.userScore || null),
      fetchedAt: Date.now(),
      screenshots: isEnhanced && existing?.screenshots?.length ? existing.screenshots : (providerMeta.screenshots || existing?.screenshots || []),
      movies: isEnhanced && existing?.movies?.length ? existing.movies : (providerMeta.movies || existing?.movies || []),
      userEnhanced: isEnhanced ? true : false,
      enhancedBy: isEnhanced ? (existing?.enhancedBy || []) : []
    };

    return base;
  }

  /**
   * Runs manual enhancement using metadata providers (e.g. ScreenScraper, TheGamesDB, IGDB, SteamGridDB).
   * Enhances game details by filling in missing metadata, or overrides if options.override is true.
   */
  async enhanceGameMetadata(
    existing: AppMeta | null,
    game: { appId: number; name: string; platform?: string | null; console?: string | null },
    options?: { providerId?: string; override?: boolean; enabledPlugins?: Record<string, boolean> }
  ): Promise<{ meta: AppMeta; enhancedBy: string[] }> {
    this.ensureInitialized();
    const targetPlugins = Array.from(this.registry.values()).filter(p => {
      if (p.pluginType !== "metadata") return false;
      if (options?.providerId) return p.id === options.providerId;
      return options?.enabledPlugins ? options.enabledPlugins[p.id] !== false : true;
    });

    const usedProviders: string[] = existing?.enhancedBy ? [...existing.enhancedBy] : [];
    let current: AppMeta = existing ? { ...existing } : {
      appId: Number(game.appId),
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
      movies: []
    };

    const override = options?.override === true;

    for (const plugin of targetPlugins) {
      if (!plugin.queryMetadata) continue;
      try {
        const config = await this.loadPluginConfig(plugin.id);
        const res = await plugin.queryMetadata({
          appId: game.appId,
          name: current.name || game.name,
          platform: (game.console || game.platform) ?? undefined
        }, config);

        if (!res) continue;

        let contributed = false;

        if (res.name && (override || !current.name)) {
          current.name = res.name;
          contributed = true;
        }
        if (res.shortDescription && (override || !current.shortDescription)) {
          current.shortDescription = res.shortDescription;
          contributed = true;
        }
        if (res.releaseDate && (override || !current.releaseDate)) {
          current.releaseDate = res.releaseDate;
          contributed = true;
        }
        if (res.releaseYear && (override || !current.releaseYear)) {
          current.releaseYear = res.releaseYear;
          contributed = true;
        }
        if (res.developers?.length && (override || !current.developers.length)) {
          current.developers = res.developers;
          contributed = true;
        }
        if (res.publishers?.length && (override || !current.publishers.length)) {
          current.publishers = res.publishers;
          contributed = true;
        }
        if (res.genres?.length && (override || !current.genres.length)) {
          current.genres = res.genres;
          contributed = true;
        }
        if (res.categories?.length && (override || !current.categories.length)) {
          current.categories = res.categories;
          contributed = true;
        }
        if (res.tags?.length && (override || !current.tags.length)) {
          current.tags = res.tags;
          contributed = true;
        }
        if (res.rating != null && (override || current.userScore == null)) {
          current.userScore = res.rating;
          contributed = true;
        }

        if (plugin.queryArtwork) {
          try {
            const art = await plugin.queryArtwork({
              appId: game.appId,
              name: current.name || game.name,
              platform: (game.console || game.platform) ?? undefined
            }, config);
            if (art) {
              if (art.capsuleImage && (override || !current.capsuleImage)) {
                current.capsuleImage = art.capsuleImage;
                contributed = true;
              }
              if (art.heroImage && (override || !current.heroImage)) {
                current.heroImage = art.heroImage;
                contributed = true;
              }
              if (art.logoImage && (override || !current.logoImage)) {
                current.logoImage = art.logoImage;
                contributed = true;
              }
              if (art.iconImage && (override || !current.iconImage)) {
                current.iconImage = art.iconImage;
                contributed = true;
              }
            }
          } catch {
            // Artwork query non-fatal
          }
        }

        if (res.capsuleImage && (override || !current.capsuleImage)) {
          current.capsuleImage = res.capsuleImage;
          contributed = true;
        }
        if (res.heroImage && (override || !current.heroImage)) {
          current.heroImage = res.heroImage;
          contributed = true;
        }
        if (res.headerImage && (override || !current.headerImage)) {
          current.headerImage = res.headerImage;
          contributed = true;
        }
        if (res.screenshots?.length && (override || !current.screenshots?.length)) {
          current.screenshots = res.screenshots;
          contributed = true;
        }

        if (contributed) {
          if (!usedProviders.includes(plugin.name)) {
            usedProviders.push(plugin.name);
          }
          if (options?.providerId) break;
        }
      } catch (e) {
        console.warn(`Plugin ${plugin.id} enhanceGameMetadata failed:`, e);
      }
    }

    current.userEnhanced = true;
    current.enhancedBy = usedProviders;
    current.fetchedAt = Date.now();

    // Persist enhanced metadata in KV cache
    await api.kvSetJson(`game_meta:${game.appId}`, current);

    return { meta: current, enhancedBy: usedProviders };
  }

  /**
   * Resets metadata cache for a game back to its provider defaults.
   * This is the only way a game provider is allowed to overwrite user/metadata-provider enhanced metadata.
   */
  async resetGameMetadataToProvider(
    game: { appId: number; name: string; platform: string; console?: string | null; exe?: string | null }
  ): Promise<AppMeta | null> {
    await api.kvDelete(`game_meta:${game.appId}`);
    const providerMeta = await this.fetchProviderGameData(game);
    if (!providerMeta) {
      return null;
    }
    const fresh = this.mergeProviderData(null, providerMeta, game, true);
    await api.kvSetJson(`game_meta:${game.appId}`, fresh);
    return fresh;
  }
}

export const pluginEngine = new PluginEngineManager();
