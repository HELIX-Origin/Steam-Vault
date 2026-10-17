import type { UniversalPlugin } from "../framework";
import type { AppMeta, PluginSettingsConfig } from "$lib/types";

export interface MediaVideoItem {
  id: string;
  title: string;
  url: string;
  thumbnailUrl?: string;
  isEmbed?: boolean;
}

export interface MediaSoundtrackItem {
  id: string;
  title: string;
  artist?: string;
  streamUrl: string;
  durationSec?: number;
}

/**
 * Checks whether a URL points to a direct HTML5 video stream (`.mp4`, `.webm`, `.m3u8`).
 */
export function isDirectVideoUrl(url?: string | null): boolean {
  if (!url) return false;
  const lower = url.toLowerCase().split("?")[0];
  return (
    lower.endsWith(".mp4") ||
    lower.endsWith(".webm") ||
    lower.endsWith(".ogg") ||
    lower.includes("steamstatic.com/store_item_assets/steam/apps/") ||
    lower.includes("video.akamai.steamstatic.com")
  );
}

/**
 * Converts a YouTube watch or short URL into an embeddable player URL if applicable.
 */
export function toEmbedVideoUrl(url: string, autoPlay = false, muted = true): string {
  if (!url) return "";
  try {
    const u = new URL(url);
    if (u.hostname.includes("youtube.com") && u.searchParams.get("v")) {
      const vid = u.searchParams.get("v");
      return `https://www.youtube-nocookie.com/embed/${vid}?autoplay=${autoPlay ? 1 : 0}&mute=${muted ? 1 : 0}`;
    }
    if (u.hostname === "youtu.be") {
      const vid = u.pathname.replace(/^\//, "");
      return `https://www.youtube-nocookie.com/embed/${vid}?autoplay=${autoPlay ? 1 : 0}&mute=${muted ? 1 : 0}`;
    }
  } catch {
    // Return original URL
  }
  return url;
}

/**
 * Global helper to extract normalized video trailer items from `AppMeta`.
 */
export function extractMediaVideos(meta?: AppMeta | null): MediaVideoItem[] {
  if (!meta?.movies || !Array.isArray(meta.movies)) return [];
  const out: MediaVideoItem[] = [];
  for (let i = 0; i < meta.movies.length; i++) {
    const m: any = meta.movies[i];
    if (typeof m === "string" && m.trim()) {
      out.push({
        id: `movie-${i}`,
        title: `${meta.name || "Game"} Trailer #${i + 1}`,
        url: m.trim(),
        isEmbed: !isDirectVideoUrl(m),
      });
    } else if (m && typeof m === "object") {
      const url = m.mp4Max || m.webmMax || m.url || m.mp4 || m.webm;
      if (url && typeof url === "string") {
        out.push({
          id: String(m.id ?? `movie-${i}`),
          title: m.name || `${meta.name || "Game"} Trailer #${i + 1}`,
          url,
          thumbnailUrl: m.thumbnail || undefined,
          isEmbed: !isDirectVideoUrl(url),
        });
      }
    }
  }
  return out;
}

/**
 * Global helper to extract normalized ambient soundtrack streams from `AppMeta`.
 */
export function extractMediaSoundtracks(meta?: AppMeta | null): MediaSoundtrackItem[] {
  if (!meta?.soundtracks || !Array.isArray(meta.soundtracks)) return [];
  const out: MediaSoundtrackItem[] = [];
  for (let i = 0; i < meta.soundtracks.length; i++) {
    const s: any = meta.soundtracks[i];
    if (typeof s === "string" && s.trim()) {
      out.push({
        id: `ost-${i}`,
        title: `${meta.name || "Game"} Theme #${i + 1}`,
        streamUrl: s.trim(),
      });
    } else if (s && typeof s === "object") {
      const streamUrl = s.streamUrl || s.url || s.previewUrl;
      if (streamUrl && typeof streamUrl === "string") {
        out.push({
          id: String(s.id ?? `ost-${i}`),
          title: s.title || s.name || `${meta.name || "Game"} Theme #${i + 1}`,
          artist: s.artist || meta.developers?.[0] || undefined,
          streamUrl,
          durationSec: typeof s.durationSec === "number" ? s.durationSec : undefined,
        });
      }
    }
  }
  return out;
}

/**
 * Universal Media Plugin (`media-universal`) consolidating video trailer playback and ambient soundtracks.
 */
export const mediaUniversalPlugin: UniversalPlugin = {
  id: "media-universal",
  name: "Universal Media & Ambient Engine",
  version: "1.0.0",
  pluginType: "media",
  author: "HELIX Origin",
  description:
    "Unified media engine for streaming official HD video trailers and playing ambient game soundtracks on game selection.",
  homepage: "https://github.com/HELIX-Origin/Universal-Game-Launcher",
  features: ["video_playback", "trailers", "audio_playback", "ambiance"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "autoPlay",
          label: "Autoplay Video Trailers",
          type: "boolean",
          default: false,
          description: "Automatically stream game trailers upon opening the game details view."
        },
        {
          key: "muted",
          label: "Start Video Muted",
          type: "boolean",
          default: true,
          description: "Mute trailer video audio by default when playback begins."
        },
        {
          key: "volume",
          label: "Master Ambient Audio Volume %",
          type: "number",
          default: 50,
          description: "Background playback volume for ambient game themes and soundtracks."
        },
        {
          key: "loopPlayback",
          label: "Loop Ambient Audio Track",
          type: "boolean",
          default: true,
          description: "Seamlessly repeat soundtrack themes while browsing title details."
        }
      ]
    };
  }
};
