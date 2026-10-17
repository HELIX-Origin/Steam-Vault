import type { DeveloperDefinition, HandRolledGame } from "./types";
import type { PluginMetadataResult } from "../../framework";
import { canonicalTitleKey } from "../../utility/title-normalizer";

import { squareEnix } from "./square-enix";
import { falcom } from "./falcom";
import { nintendo } from "./nintendo";
import { capcom } from "./capcom";
import { fromsoftware } from "./fromsoftware";
import { valve } from "./valve";
import { cdProjekt } from "./cd-projekt";
import { segaAtlus } from "./sega-atlus";
import { bandaiNamco } from "./bandai-namco";
import { bethesda } from "./bethesda";
import { rockstar } from "./rockstar";
import { electronicArts } from "./electronic-arts";
import { ubisoft } from "./ubisoft";
import { blizzard } from "./blizzard";
import { sony } from "./sony";
import { microsoft } from "./microsoft";
import { konami } from "./konami";
import { koeiTecmo } from "./koei-tecmo";
import { spikeChunsoft } from "./spike-chunsoft";
import { remedy } from "./remedy";
import { ioInteractive } from "./io-interactive";
import { proxxie } from "./proxxie";
import { sierraLee } from "./sierra-lee";
import { kompas } from "./kompas";
import { neyor } from "./neyor";
import { nltMedia } from "./nlt-media";
import { caribdis } from "./caribdis";
import { drPinkCake } from "./dr-pink-cake";

export * from "./types";

/**
 * Authoritative catalog of major game developers, publishers, and their hand-rolled games.
 * Modeled after hand-rolled localization catalogs with per-developer data files.
 */
export const DEVELOPER_CATALOG: DeveloperDefinition[] = [
  squareEnix,
  falcom,
  nintendo,
  capcom,
  fromsoftware,
  valve,
  cdProjekt,
  segaAtlus,
  bandaiNamco,
  bethesda,
  rockstar,
  electronicArts,
  ubisoft,
  blizzard,
  sony,
  microsoft,
  konami,
  koeiTecmo,
  spikeChunsoft,
  remedy,
  ioInteractive,
  proxxie,
  sierraLee,
  kompas,
  neyor,
  nltMedia,
  caribdis,
  drPinkCake,
];

/** Backwards-compatible alias for existing callers */
export const KNOWN_DEVELOPERS = DEVELOPER_CATALOG;
export type KnownDeveloper = DeveloperDefinition;

export const DEVELOPERS_BY_ID: Record<string, DeveloperDefinition> = Object.fromEntries(
  DEVELOPER_CATALOG.map((dev) => [dev.id, dev])
);

/**
 * Checks whether candidate developer or publisher name matches a known developer studio.
 */
export function matchesKnownDeveloper(dev: DeveloperDefinition, candidateName: string): boolean {
  const cLower = candidateName.trim().toLowerCase();
  for (const alias of dev.aliases) {
    const aLower = alias.toLowerCase();
    if (cLower === aLower || cLower.includes(aLower) || aLower.includes(cLower)) {
      return true;
    }
  }
  return false;
}

/**
 * Detects whether a game title belongs to a known developer's flagship franchise or studio aliases.
 */
export function detectKnownDeveloper(title: string): DeveloperDefinition | null {
  const tKey = canonicalTitleKey(title);
  if (!tKey) return null;

  for (const dev of DEVELOPER_CATALOG) {
    for (const franchise of dev.franchises) {
      const fKey = canonicalTitleKey(franchise);
      if (!fKey) continue;
      if (tKey === fKey || tKey.startsWith(fKey) || tKey.includes(fKey)) {
        return dev;
      }
    }
    for (const alias of dev.aliases) {
      const aKey = canonicalTitleKey(alias);
      if (!aKey || aKey.length < 4) continue;
      if (tKey === aKey || tKey.startsWith(aKey)) {
        return dev;
      }
    }
  }
  return null;
}

/**
 * Finds a specific hand-rolled game across the per-developer catalogs.
 */
export function findHandRolledGame(title: string): HandRolledGame | null {
  const tKey = canonicalTitleKey(title);
  if (!tKey) return null;

  for (const dev of DEVELOPER_CATALOG) {
    for (const game of dev.games) {
      if (canonicalTitleKey(game.title) === tKey) {
        return game;
      }
      if (game.aliases) {
        for (const alias of game.aliases) {
          if (canonicalTitleKey(alias) === tKey) {
            return game;
          }
        }
      }
    }
  }
  return null;
}

/**
 * Validates whether candidate title and creators match the target title's known developer.
 * Prevents cross-studio mismatches (e.g. Ys VIII from Nihon Falcom matched to Final Fantasy XIV from Square Enix).
 */
export function validateMetadataCandidate(
  targetTitle: string,
  candidateTitle: string,
  candidateDevelopers?: string[],
  candidatePublishers?: string[]
): boolean {
  const targetDev = detectKnownDeveloper(targetTitle);
  const candidateDev = detectKnownDeveloper(candidateTitle);

  // If both titles are identified with different known major developers, reject!
  if (targetDev && candidateDev && targetDev.id !== candidateDev.id) {
    return false;
  }

  // If target belongs to a known developer, ensure candidate creators do not belong to another major studio
  if (targetDev) {
    const creators = [...(candidateDevelopers || []), ...(candidatePublishers || [])];
    for (const c of creators) {
      for (const otherDev of DEVELOPER_CATALOG) {
        if (otherDev.id !== targetDev.id && matchesKnownDeveloper(otherDev, c)) {
          return false;
        }
      }
    }
  }

  return true;
}

/**
 * Verifies that candidate title matches target title and does not represent auxiliary media.
 */
export function titlesMatchStrict(target: string, candidate: string): boolean {
  const tLower = target.toLowerCase();
  const cLower = candidate.toLowerCase();

  const nonGameMedia = [
    "soundtrack",
    "ost",
    "original soundtrack",
    "album",
    "score",
    "discography",
    "music of",
    "artbook",
    "art book",
    "dlc",
    "expansion pack",
    "season pass",
    "strategy guide",
    "walkthrough",
    "trailer",
  ];

  const hasMedia = (s: string) => nonGameMedia.some((m) => s.includes(m));
  if (hasMedia(cLower) && !hasMedia(tLower)) {
    return false;
  }

  const k1 = canonicalTitleKey(target);
  const k2 = canonicalTitleKey(candidate);
  if (!k1 || !k2) return false;
  if (k1 === k2) return true;

  if (k1.includes(k2) || k2.includes(k1)) {
    const longer = k1.length > k2.length ? k1 : k2;
    const shorter = k1.length > k2.length ? k2 : k1;
    const remainder = longer.replace(shorter, "");

    const hasSequel = /[0-9]|ii|iii|iv|v|vi/.test(remainder);
    if (hasSequel) return false;

    const benign = [
      "edition",
      "definitive",
      "remastered",
      "goty",
      "complete",
      "hd",
      "deluxe",
      "director",
      "online",
    ];
    if (benign.some((b) => remainder.includes(b)) || remainder.length <= 12) {
      return true;
    }
  }

  return false;
}

/**
 * Provides authoritative backup metadata (and curated artwork where available) for a title
 * using the hand-rolled games database and per-developer data files.
 */
export function backupMetadataForTitle(title: string): PluginMetadataResult | null {
  // 1. Check for an exact or aliased hand-rolled game entry first
  const handRolled = findHandRolledGame(title);
  if (handRolled) {
    return {
      name: handRolled.title,
      shortDescription: handRolled.shortDescription,
      detailedDescription: handRolled.detailedDescription,
      releaseDate: handRolled.releaseDate,
      releaseYear: handRolled.releaseYear,
      developers: handRolled.developers || [],
      publishers: handRolled.publishers || [],
      genres: handRolled.genres || [],
      tags: handRolled.tags || [],
      categories: handRolled.categories || [],
      userScore: handRolled.userScore,
      rating: handRolled.userScore,
      ageRating: handRolled.ageRating,
      capsuleImage: handRolled.capsuleImage,
      heroImage: handRolled.heroImage,
      logoImage: handRolled.logoImage,
      headerImage: handRolled.headerImage,
      iconImage: handRolled.iconImage,
      screenshots: handRolled.screenshots || [],
      movies: handRolled.movies || [],
      soundtracks: handRolled.soundtracks || [],
    };
  }

  // 2. Otherwise fall back to the developer studio's flagship knowledge
  const dev = detectKnownDeveloper(title);
  if (!dev) return null;

  return {
    name: title,
    developers: [dev.name],
    publishers: [dev.name],
    genres: [...dev.primaryGenres],
  };
}
