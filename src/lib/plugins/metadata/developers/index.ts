import type { DeveloperDefinition, HandRolledGame } from "./types";
import type { PluginMetadataResult } from "../../framework";
import { canonicalTitleKey } from "../../utility/title-normalizer";

export * from "./types";

/**
 * Authoritative studio profiles for major game developers, publishers, and creators.
 * Replaces hardcoded static game arrays with dynamic HELIX Gaming DB Three-Tier REST API resolution
 * while preserving high-precision studio matching and algorithmic candidate validation.
 */
export const DEVELOPER_CATALOG: DeveloperDefinition[] = [
  {
    id: "square-enix",
    name: "Square Enix",
    aliases: ["Square", "Enix", "Squaresoft", "Square Enix Holdings"],
    franchises: [
      "Final Fantasy",
      "Dragon Quest",
      "Kingdom Hearts",
      "Chrono",
      "NieR",
      "Star Ocean",
      "Bravely",
      "Octopath Traveler",
      "SaGa",
      "Mana",
      "Valkyrie Profile",
      "Front Mission",
    ],
    primaryGenres: ["JRPG", "RPG", "Action RPG"],
    website: "https://square-enix-games.com",
    games: [],
  },
  {
    id: "falcom",
    name: "Nihon Falcom",
    aliases: ["Falcom"],
    franchises: [
      "Ys",
      "The Legend of Heroes",
      "Trails",
      "Trails in the Sky",
      "Trails of Cold Steel",
      "Trails from Zero",
      "Trails to Azure",
      "Trails into Reverie",
      "Tokyo Xanadu",
      "Xanadu",
      "Brandish",
      "Zwei",
      "Gurumin",
    ],
    primaryGenres: ["JRPG", "Action RPG"],
    website: "https://www.falcom.co.jp",
    games: [],
  },
  {
    id: "nintendo",
    name: "Nintendo",
    aliases: ["Nintendo EPD", "Nintendo EAD", "Nintendo Software Planning & Development"],
    franchises: [
      "Super Mario",
      "Mario",
      "The Legend of Zelda",
      "Zelda",
      "Pokemon",
      "Metroid",
      "Fire Emblem",
      "Kirby",
      "Donkey Kong",
      "Xenoblade",
      "Super Smash Bros",
      "Pikmin",
      "Animal Crossing",
      "Star Fox",
      "F-Zero",
      "Splatoon",
    ],
    primaryGenres: ["Action", "Adventure", "Platformer", "RPG", "Strategy"],
    website: "https://www.nintendo.com",
    games: [],
  },
  {
    id: "capcom",
    name: "Capcom",
    aliases: ["Capcom Production Studio", "Capcom Development Division"],
    franchises: [
      "Monster Hunter",
      "Resident Evil",
      "Biohazard",
      "Devil May Cry",
      "Street Fighter",
      "Mega Man",
      "Rockman",
      "Ace Attorney",
      "Dragon's Dogma",
      "Onimusha",
      "Dead Rising",
      "Ghosts 'n Goblins",
      "Okami",
    ],
    primaryGenres: ["Action", "Action RPG", "Survival Horror", "Fighting"],
    website: "https://www.capcom.com",
    games: [],
  },
  {
    id: "fromsoftware",
    name: "FromSoftware",
    aliases: ["From Software", "FromSoft"],
    franchises: [
      "Dark Souls",
      "Demon's Souls",
      "Bloodborne",
      "Elden Ring",
      "Armored Core",
      "Sekiro",
      "King's Field",
      "Tenchu",
    ],
    primaryGenres: ["Action RPG", "Soulslike", "Mecha"],
    website: "https://www.fromsoftware.jp",
    games: [],
  },
  {
    id: "valve",
    name: "Valve",
    aliases: ["Valve Corporation", "Valve Software"],
    franchises: [
      "Half-Life",
      "Portal",
      "Left 4 Dead",
      "Counter-Strike",
      "Dota",
      "Team Fortress",
      "Day of Defeat",
      "Ricochet",
    ],
    primaryGenres: ["First-Person Shooter", "Puzzle", "Multiplayer", "MOBA"],
    website: "https://www.valvesoftware.com",
    games: [],
  },
  {
    id: "cd-projekt",
    name: "CD PROJEKT RED",
    aliases: ["CD Projekt", "CDPR", "CD Projekt Red"],
    franchises: ["The Witcher", "Cyberpunk", "Gwent"],
    primaryGenres: ["RPG", "Action RPG", "Open World"],
    website: "https://en.cdprojektred.com",
    games: [],
  },
  {
    id: "sega-atlus",
    name: "SEGA / Atlus",
    aliases: ["SEGA", "Atlus", "Ryu Ga Gotoku Studio", "Sonic Team"],
    franchises: [
      "Sonic the Hedgehog",
      "Sonic",
      "Yakuza",
      "Like a Dragon",
      "Persona",
      "Shin Megami Tensei",
      "Total War",
      "Phantasy Star",
      "Judgment",
      "Super Monkey Ball",
      "Bayonetta",
      "Valkyria Chronicles",
    ],
    primaryGenres: ["JRPG", "Action", "Adventure", "Platformer"],
    website: "https://www.sega.com",
    games: [],
  },
  {
    id: "bandai-namco",
    name: "Bandai Namco",
    aliases: ["Bandai Namco Entertainment", "Namco", "Bandai"],
    franchises: [
      "Tales of",
      "Tekken",
      "Pac-Man",
      "Ace Combat",
      "Soulcalibur",
      "God Eater",
      "Scarlet Nexus",
      "Digimon",
      "Dragon Ball",
      "Naruto",
      "One Piece",
    ],
    primaryGenres: ["JRPG", "Action RPG", "Fighting", "Arcade"],
    website: "https://www.bandainamcoent.com",
    games: [],
  },
  {
    id: "bethesda",
    name: "Bethesda Softworks",
    aliases: ["Bethesda", "Bethesda Game Studios", "BGS", "ZeniMax"],
    franchises: [
      "The Elder Scrolls",
      "Skyrim",
      "Morrowind",
      "Oblivion",
      "Fallout",
      "Doom",
      "Wolfenstein",
      "Dishonored",
      "Starfield",
      "Quake",
      "Prey",
      "The Evil Within",
    ],
    primaryGenres: ["RPG", "Action RPG", "FPS", "Open World"],
    website: "https://bethesda.net",
    games: [],
  },
  {
    id: "rockstar",
    name: "Rockstar Games",
    aliases: ["Rockstar", "Rockstar North", "Rockstar San Diego"],
    franchises: [
      "Grand Theft Auto",
      "GTA",
      "Red Dead",
      "Max Payne",
      "Bully",
      "L.A. Noire",
      "Midnight Club",
      "Manhunt",
    ],
    primaryGenres: ["Open World", "Action-Adventure", "Third-Person Shooter"],
    website: "https://www.rockstargames.com",
    games: [],
  },
  {
    id: "electronic-arts",
    name: "Electronic Arts",
    aliases: ["EA", "EA Games", "Respawn Entertainment", "BioWare", "DICE", "Criterion", "EA Sports"],
    franchises: [
      "Mass Effect",
      "Dragon Age",
      "Dead Space",
      "Titanfall",
      "Apex Legends",
      "Star Wars Jedi",
      "Battlefield",
      "Need for Speed",
      "The Sims",
      "Mirror's Edge",
      "Skate",
    ],
    primaryGenres: ["RPG", "Action RPG", "FPS", "Sci-Fi"],
    website: "https://www.ea.com",
    games: [],
  },
  {
    id: "ubisoft",
    name: "Ubisoft",
    aliases: ["Ubisoft Montreal", "Ubisoft Quebec", "Ubisoft Paris", "Ubi"],
    franchises: [
      "Assassin's Creed",
      "Far Cry",
      "Tom Clancy",
      "Splinter Cell",
      "Rainbow Six",
      "Ghost Recon",
      "Watch Dogs",
      "Rayman",
      "Prince of Persia",
      "The Division",
      "For Honor",
    ],
    primaryGenres: ["Action-Adventure", "Open World", "Stealth", "Tactical Shooter"],
    website: "https://www.ubisoft.com",
    games: [],
  },
  {
    id: "blizzard",
    name: "Blizzard Entertainment",
    aliases: ["Blizzard", "Blizzard North", "Activision Blizzard"],
    franchises: [
      "Warcraft",
      "World of Warcraft",
      "Diablo",
      "Starcraft",
      "Overwatch",
      "Hearthstone",
      "Heroes of the Storm",
    ],
    primaryGenres: ["RPG", "Action RPG", "MMORPG", "Strategy", "Hero Shooter"],
    website: "https://www.blizzard.com",
    games: [],
  },
  {
    id: "sony",
    name: "Sony Interactive Entertainment",
    aliases: [
      "PlayStation Studios",
      "Sony Computer Entertainment",
      "Naughty Dog",
      "Santa Monica Studio",
      "Insomniac Games",
      "Sucker Punch",
      "Guerrilla Games",
    ],
    franchises: [
      "The Last of Us",
      "Uncharted",
      "God of War",
      "Spider-Man",
      "Horizon",
      "Ghost of Tsushima",
      "Gran Turismo",
      "Ratchet & Clank",
      "Jak and Daxter",
      "Killzone",
    ],
    primaryGenres: ["Action-Adventure", "Story-Rich", "Open World"],
    website: "https://www.playstation.com",
    games: [],
  },
  {
    id: "microsoft",
    name: "Xbox Game Studios",
    aliases: [
      "Microsoft Game Studios",
      "343 Industries",
      "Playground Games",
      "Turn 10",
      "Rare",
      "The Coalition",
      "Obsidian Entertainment",
      "Ninja Theory",
    ],
    franchises: [
      "Halo",
      "Gears of War",
      "Forza",
      "Forza Horizon",
      "Sea of Thieves",
      "Fable",
      "Age of Empires",
      "Avowed",
      "The Outer Worlds",
      "Hellblade",
      "Pentiment",
      "Banjo-Kazooie",
    ],
    primaryGenres: ["FPS", "Racing", "RPG", "Action-Adventure", "Strategy"],
    website: "https://www.xbox.com",
    games: [],
  },
  {
    id: "konami",
    name: "Konami",
    aliases: ["Konami Digital Entertainment"],
    franchises: [
      "Metal Gear",
      "Metal Gear Solid",
      "Silent Hill",
      "Castlevania",
      "Contra",
      "eFootball",
      "Pro Evolution Soccer",
      "Yu-Gi-Oh!",
      "Suikoden",
    ],
    primaryGenres: ["Stealth", "Action", "Survival Horror", "Metroidvania"],
    website: "https://www.konami.com",
    games: [],
  },
  {
    id: "koei-tecmo",
    name: "Koei Tecmo",
    aliases: ["Koei", "Tecmo", "Team Ninja", "Gust", "Omega Force"],
    franchises: [
      "Dynasty Warriors",
      "Samurai Warriors",
      "Warriors",
      "Nioh",
      "Ninja Gaiden",
      "Atelier",
      "Dead or Alive",
      "Fatal Frame",
      "Wo Long",
    ],
    primaryGenres: ["Action", "Musou", "Action RPG", "JRPG", "Fighting"],
    website: "https://www.koeitecmo.co.jp",
    games: [],
  },
  {
    id: "spike-chunsoft",
    name: "Spike Chunsoft",
    aliases: ["Chunsoft", "Spike"],
    franchises: [
      "Danganronpa",
      "Zero Escape",
      "Nonary Games",
      "Mystery Dungeon",
      "AI: The Somnium Files",
      "Shiren the Wanderer",
      "Steins;Gate",
    ],
    primaryGenres: ["Visual Novel", "Mystery", "Adventure", "Roguelike"],
    website: "https://www.spike-chunsoft.co.jp",
    games: [],
  },
  {
    id: "remedy",
    name: "Remedy Entertainment",
    aliases: ["Remedy"],
    franchises: ["Alan Wake", "Control", "Max Payne", "Quantum Break"],
    primaryGenres: ["Action-Adventure", "Psychological Horror", "Third-Person Shooter", "Story-Rich"],
    website: "https://www.remedygames.com",
    games: [],
  },
  {
    id: "io-interactive",
    name: "IO Interactive",
    aliases: ["IOI"],
    franchises: ["Hitman", "Project 007", "Freedom Fighters"],
    primaryGenres: ["Stealth", "Action-Adventure", "Simulation"],
    website: "https://ioi.dk",
    games: [],
  },
  {
    id: "proxxie",
    name: "Proxxie",
    aliases: ["Team Proxxie"],
    franchises: ["The Double Agent"],
    primaryGenres: ["Role-playing (RPG)", "Adult", "Adventure", "Fantasy", "Stealth"],
    games: [],
  },
  {
    id: "sierra-lee",
    name: "Sierra Lee",
    aliases: ["Sierra Lee Games", "Anduo Games"],
    franchises: ["The Last Sovereign", "Third Crisis"],
    primaryGenres: ["Role-playing (RPG)", "Adult", "Strategy", "Political", "Adventure"],
    games: [],
  },
  {
    id: "kompas",
    name: "Kompas",
    aliases: ["Studio Kompas"],
    franchises: ["FreshWomen"],
    primaryGenres: ["Visual Novel", "Adult", "Comedy", "Slice of Life", "Romance"],
    games: [],
  },
  {
    id: "neyor",
    name: "Neyor",
    aliases: ["Studio Neyor"],
    franchises: ["Love & Sex: Second Base"],
    primaryGenres: ["Visual Novel", "Adult", "Comedy", "Fantasy", "Romance"],
    games: [],
  },
  {
    id: "nlt-media",
    name: "NLT Media",
    aliases: ["NLT"],
    franchises: ["The Genesis Order", "Lust Epidemic", "Treasure of Nadia"],
    primaryGenres: ["Adventure", "Adult", "Puzzle", "Mystery"],
    games: [],
  },
  {
    id: "caribdis",
    name: "Caribdis",
    aliases: ["Caribdis Games"],
    franchises: ["Eternum", "Once in a Lifetime"],
    primaryGenres: ["Visual Novel", "Adult", "Cyberpunk", "Comedy", "Sci-Fi"],
    games: [],
  },
  {
    id: "dr-pink-cake",
    name: "Dr Pink Cake",
    aliases: ["DrPinkCake", "DPC"],
    franchises: ["Being a DIK", "Acting Lessons"],
    primaryGenres: ["Visual Novel", "Adult", "Comedy", "College", "Drama"],
    games: [],
  },
];

/** Backwards-compatible aliases for existing callers */
export const KNOWN_DEVELOPERS = DEVELOPER_CATALOG;
export const KNOWN_STUDIOS = DEVELOPER_CATALOG;
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
 * Backward-compatible helper. Hand-rolled static games have been migrated
 * to the HELIX Gaming DB Three-Tier REST API. Returns null so callers delegate
 * to dynamic API discovery.
 */
export function findHandRolledGame(_title: string): HandRolledGame | null {
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
 * Provides authoritative studio-level fallback metadata (creators, genres) for a title
 * when online metadata providers have not returned results.
 */
export function backupMetadataForTitle(title: string): PluginMetadataResult | null {
  const dev = detectKnownDeveloper(title);
  if (!dev) return null;

  return {
    name: title,
    developers: [dev.name],
    publishers: [dev.name],
    genres: [...dev.primaryGenres],
  };
}
