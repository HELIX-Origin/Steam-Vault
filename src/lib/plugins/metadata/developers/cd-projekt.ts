import type { DeveloperDefinition } from "./types";

export const cdProjekt: DeveloperDefinition = {
  id: "cd-projekt",
  name: "CD Projekt Red",
  aliases: ["CD Projekt Red", "CD Projekt", "CD PROJEKT S.A.", "CD PROJEKT RED"],
  franchises: ["The Witcher", "Cyberpunk 2077", "Cyberpunk", "Gwent"],
  primaryGenres: ["Role-playing (RPG)", "Open World", "Action RPG", "Story Rich"],
  website: "https://www.cdprojektred.com",
  games: [
    {
      title: "The Witcher 3: Wild Hunt",
      aliases: ["Witcher 3", "The Witcher 3: Wild Hunt - Complete Edition", "TW3"],
      franchise: "The Witcher",
      developers: ["CD PROJEKT RED"],
      publishers: ["CD PROJEKT S.A."],
      genres: ["Action RPG", "Open World", "Dark Fantasy", "Story Rich"],
      releaseDate: "2015-05-18",
      releaseYear: 2015,
      shortDescription:
        "You are Geralt of Rivia, mercenary monster slayer. Before you stands a war-torn, monster-infested continent you can explore at will. Your current contract? Tracking down Ciri — the Child of Prophecy, a living weapon that can alter the shape of the world.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/292030/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/292030/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/292030/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/292030/header.jpg",
    },
    {
      title: "Cyberpunk 2077",
      aliases: ["Cyberpunk 2077: Phantom Liberty", "CP2077"],
      franchise: "Cyberpunk 2077",
      developers: ["CD PROJEKT RED"],
      publishers: ["CD PROJEKT S.A."],
      genres: ["Cyberpunk", "Open World", "Action RPG", "First-Person"],
      releaseDate: "2020-12-10",
      releaseYear: 2020,
      shortDescription:
        "Cyberpunk 2077 is an open-world, action-adventure RPG set in the megalopolis of Night City, where you play as a cyberpunk mercenary wrapped up in a do-or-die fight for survival.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1091500/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1091500/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1091500/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1091500/header.jpg",
    },
  ],
};
