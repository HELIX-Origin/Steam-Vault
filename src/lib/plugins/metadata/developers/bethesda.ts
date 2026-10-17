import type { DeveloperDefinition } from "./types";

export const bethesda: DeveloperDefinition = {
  id: "bethesda",
  name: "Bethesda Softworks",
  aliases: [
    "Bethesda",
    "Bethesda Softworks",
    "Bethesda Game Studios",
    "ZeniMax",
    "id Software",
    "Arkane Studios",
    "MachineGames",
    "Tango Gameworks",
  ],
  franchises: [
    "The Elder Scrolls",
    "Skyrim",
    "Morrowind",
    "Oblivion",
    "Fallout",
    "Doom",
    "DOOM",
    "Wolfenstein",
    "Dishonored",
    "Prey",
    "Quake",
    "Starfield",
    "The Evil Within",
    "Deathloop",
    "Rage",
  ],
  primaryGenres: ["Role-playing (RPG)", "Open World", "First-Person Shooter", "Immersive Sim"],
  website: "https://bethesda.net",
  games: [
    {
      title: "The Elder Scrolls V: Skyrim Special Edition",
      aliases: ["Skyrim", "Skyrim SE", "TES V"],
      franchise: "The Elder Scrolls",
      developers: ["Bethesda Game Studios"],
      publishers: ["Bethesda Softworks"],
      genres: ["Open World", "RPG", "Fantasy", "Action RPG"],
      releaseDate: "2016-10-27",
      releaseYear: 2016,
      shortDescription:
        "Winner of more than 200 Game of the Year Awards, Skyrim Special Edition brings the epic fantasy to life in stunning detail.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/489830/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/489830/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/489830/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/489830/header.jpg",
    },
    {
      title: "DOOM Eternal",
      aliases: ["Doom Eternal"],
      franchise: "Doom",
      developers: ["id Software"],
      publishers: ["Bethesda Softworks"],
      genres: ["First-Person Shooter", "Action", "Fast-Paced", "Gore"],
      releaseDate: "2020-03-19",
      releaseYear: 2020,
      shortDescription:
        "Hell’s armies have invaded Earth. Become the Slayer in an epic single-player campaign to conquer demons across dimensions and stop the final destruction of humanity.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/782330/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/782330/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/782330/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/782330/header.jpg",
    },
  ],
};
