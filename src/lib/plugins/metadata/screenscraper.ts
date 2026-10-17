import type { UniversalPlugin, TestConnectionResult, PluginMetadataResult, PluginArtworkResult } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

const BUILTIN_DEV_ID = "helixorigin";
const BUILTIN_DEV_PASS = "";

export const screenscraperPlugin: UniversalPlugin = {
  id: "metadata-screenscraper",
  name: "ScreenScraper Multi-Console ROM Provider",
  version: "1.0.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description: "Keyless retro gaming metadata and artwork provider supporting 2D/3D box art, wheel logos, screenshot titles, and video snaps from screenscraper.fr.",
  homepage: "https://www.screenscraper.fr",
  features: ["retro_metadata", "boxart_2d_3d", "wheel_logos", "video_snaps", "screenshots", "multi_language"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "userId",
          label: "ScreenScraper Username",
          type: "text",
          required: false,
          default: "",
          description: "Keyless: ScreenScraper does not use API keys. Entering your screenscraper.fr website account username (ssid) enables user account verification for higher speeds and daily quota.",
          placeholder: "Your screenscraper.fr username…"
        },
        {
          key: "userPassword",
          label: "ScreenScraper Password",
          type: "password",
          required: false,
          default: "",
          description: "Your personal screenscraper.fr website account password (sspassword).",
          placeholder: "Your screenscraper.fr password…"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const userId = (config?.userId || "").trim();
    const userPass = (config?.userPassword || "").trim();
    const devId = (config?.devId || BUILTIN_DEV_ID).trim();
    const devPassword = (config?.devPassword || BUILTIN_DEV_PASS).trim();

    // If user provided account credentials, verify the account directly via ssuserInfos.php
    if (userId && userPass) {
      try {
        const url = `https://api.screenscraper.fr/api2/ssuserInfos.php?devid=${encodeURIComponent(devId)}&devpassword=${encodeURIComponent(devPassword)}&softname=UniversalGameLauncher&output=json&ssid=${encodeURIComponent(userId)}&sspassword=${encodeURIComponent(userPass)}`;
        const res = await fetch(url, { signal: AbortSignal.timeout(7000) });
        if (res.ok) {
          const data = await res.json().catch(() => null);
          const user = data?.response?.ssuser;
          if (user?.id) {
            const threads = user.maxthreads || 1;
            const speed = user.maxdownloadspeed ? `${user.maxdownloadspeed} KB/s` : "standard";
            return {
              success: true,
              message: `User account verified! Welcome ${user.id} (Level: ${user.niveau || 1}, Allowed Threads: ${threads}, Speed: ${speed}).`
            };
          }
        }
        const text = await res.text().catch(() => "");
        if (text.includes("Erreur") || text.includes("login")) {
          return {
            success: false,
            message: "User account verification failed: Invalid ScreenScraper username or password."
          };
        }
        return {
          success: true,
          message: `User account '${userId}' submitted for ScreenScraper.`
        };
      } catch (err: any) {
        return { success: false, message: `Account verification error: ${err?.message || err}` };
      }
    }

    // Keyless mode check
    return {
      success: true,
      message: "Keyless mode active: ScreenScraper operates without an API key. Optional: enter your screenscraper.fr username and password above for higher scrape speeds and threads."
    };
  },

  async queryMetadata(query, config): Promise<PluginMetadataResult | null> {
    const name = query.name?.trim();
    if (!name) return null;

    const devId = (config?.devId || BUILTIN_DEV_ID).trim();
    const devPassword = (config?.devPassword || BUILTIN_DEV_PASS).trim();
    const userId = (config?.userId || "").trim();
    const userPass = (config?.userPassword || "").trim();

    const cleanName = name
      .replace(/\(USA\)|\(Europe\)|\(Japan\)|\(World\)|\(Rev \d+\)|\[!\]/gi, "")
      .replace(/\.(nes|snes|z64|n64|iso|rvz|gcz|ciso|wbfs|bin|cue|chd|gba|gbc|gb|nds|3ds|nsp|xci)$/i, "")
      .trim();

    try {
      // 1. Search for game via jeuRecherche.php
      let searchUrl = `https://api.screenscraper.fr/api2/jeuRecherche.php?devid=${encodeURIComponent(devId)}&devpassword=${encodeURIComponent(devPassword)}&softname=UniversalGameLauncher&output=json&recherche=${encodeURIComponent(cleanName)}`;
      if (userId && userPass) {
        searchUrl += `&ssid=${encodeURIComponent(userId)}&sspassword=${encodeURIComponent(userPass)}`;
      }

      const searchRes = await fetch(searchUrl, { signal: AbortSignal.timeout(8000) });
      let gameId: string | number | undefined;
      if (searchRes.ok) {
        const searchData = await searchRes.json().catch(() => null);
        const jeux = searchData?.response?.jeux;
        gameId = Array.isArray(jeux) ? jeux[0]?.id : searchData?.response?.jeu?.id;
      }

      // 2. Query full game metadata & media via jeuInfos.php
      let infoUrl = `https://api.screenscraper.fr/api2/jeuInfos.php?devid=${encodeURIComponent(devId)}&devpassword=${encodeURIComponent(devPassword)}&softname=UniversalGameLauncher&output=json`;
      if (gameId) {
        infoUrl += `&gameid=${encodeURIComponent(gameId)}`;
      } else {
        infoUrl += `&romnom=${encodeURIComponent(cleanName)}`;
      }
      if (userId && userPass) {
        infoUrl += `&ssid=${encodeURIComponent(userId)}&sspassword=${encodeURIComponent(userPass)}`;
      }

      const res = await fetch(infoUrl, { signal: AbortSignal.timeout(8000) });
      if (!res.ok) return null;
      const data = await res.json().catch(() => null);
      const jeu = data?.response?.jeu;
      if (!jeu) return null;

      // Extract synopsis in preferred locale or English
      const synopses = jeu.synopsis || [];
      const synopsisObj = Array.isArray(synopses)
        ? synopses.find((s: any) => s.langue === "en" || s.langue === "us") || synopses[0]
        : synopses;
      const synopsis = synopsisObj?.texte;

      // Extract media artwork (2D box, 3D box, wheel, fanart, video)
      let box2d: string | undefined;
      let box3d: string | undefined;
      let wheel: string | undefined;
      let fanart: string | undefined;
      let video: string | undefined;
      const screenshots: string[] = [];

      const medias = jeu.medias || [];
      if (Array.isArray(medias)) {
        for (const m of medias) {
          if (m.type === "box-2D" && !box2d) box2d = m.url;
          if (m.type === "box-3D" && !box3d) box3d = m.url;
          if (m.type === "wheel" && !wheel) wheel = m.url;
          if (m.type === "fanart" && !fanart) fanart = m.url;
          if (m.type === "video" && !video) video = m.url;
          if (m.type === "ss" && screenshots.length < 10) screenshots.push(m.url);
        }
      }

      const genres = Array.isArray(jeu.genres) ? jeu.genres.map((g: any) => g.nom || g.nom_us || g).filter(Boolean) : [];
      const releaseDate = jeu.dates?.[0]?.date || undefined;
      const releaseYear = releaseDate ? parseInt(releaseDate.slice(0, 4), 10) : undefined;

      return {
        name: jeu.noms?.[0]?.nom || cleanName,
        shortDescription: synopsis ? synopsis.slice(0, 300) : undefined,
        detailedDescription: synopsis || undefined,
        releaseDate,
        releaseYear: isNaN(releaseYear as number) ? undefined : releaseYear,
        developers: jeu.developpeur?.nom ? [jeu.developpeur.nom] : [],
        publishers: jeu.editeur?.nom ? [jeu.editeur.nom] : [],
        genres,
        players: jeu.joueurs?.texte || undefined,
        rating: jeu.note ? parseFloat(jeu.note) * 5 : undefined,
        communityRating: jeu.note ? parseFloat(jeu.note) * 5 : undefined,
        capsuleImage: box3d || box2d,
        heroImage: fanart,
        logoImage: wheel,
        screenshots: screenshots.length > 0 ? screenshots : undefined,
        movies: video ? [video] : undefined
      };
    } catch {
      return null;
    }
  },

  async queryArtwork(query, config): Promise<PluginArtworkResult | null> {
    const meta = await this.queryMetadata!(query, config);
    if (!meta) return null;
    return {
      capsuleImage: meta.capsuleImage,
      heroImage: meta.heroImage,
      logoImage: meta.logoImage
    };
  }
};
