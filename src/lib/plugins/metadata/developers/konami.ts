import type { DeveloperDefinition } from "./types";

export const konami: DeveloperDefinition = {
  id: "konami",
  name: "Konami",
  aliases: ["Konami", "Konami Digital Entertainment", "KCET"],
  franchises: [
    "Metal Gear",
    "Metal Gear Solid",
    "Silent Hill",
    "Castlevania",
    "Contra",
    "Yu-Gi-Oh!",
    "eFootball",
    "Pro Evolution Soccer",
    "PES",
    "Suikoden",
    "Zone of the Enders",
    "Bomberman",
    "Gradius",
    "Dance Dance Revolution",
  ],
  primaryGenres: ["Stealth", "Action-Adventure", "Survival Horror", "Card Game", "Sports"],
  website: "https://www.konami.com",
  games: [
    {
      title: "Metal Gear Solid V: The Phantom Pain",
      aliases: ["MGSV", "MGS5", "The Phantom Pain"],
      franchise: "Metal Gear",
      developers: ["Konami Digital Entertainment"],
      publishers: ["Konami Digital Entertainment"],
      genres: ["Stealth", "Open World", "Action-Adventure", "Tactical"],
      releaseDate: "2015-09-01",
      releaseYear: 2015,
      shortDescription:
        "Ushering in a new era for the METAL GEAR franchise with cutting-edge technology powered by the Fox Engine, METAL GEAR SOLID V: The Phantom Pain provides players a top-tier gaming experience as they are offered tactical freedom to carry out open-world missions.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/287700/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/287700/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/287700/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/287700/header.jpg",
    },
    {
      title: "Silent Hill 2",
      aliases: ["Silent Hill 2 (2024)", "SH2 Remake"],
      franchise: "Silent Hill",
      developers: ["Bloober Team"],
      publishers: ["KONAMI"],
      genres: ["Psychological Horror", "Survival Horror", "Story Rich"],
      releaseDate: "2024-10-08",
      releaseYear: 2024,
      shortDescription:
        "Having received a letter from his deceased wife, James heads to where they shared so many memories, in the hope of seeing her one more time: Silent Hill.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2124490/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2124490/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2124490/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2124490/header.jpg",
    },
  ],
};
