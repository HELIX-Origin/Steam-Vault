import type { DeveloperDefinition } from "./types";

export const electronicArts: DeveloperDefinition = {
  id: "electronic-arts",
  name: "Electronic Arts",
  aliases: [
    "Electronic Arts",
    "EA",
    "EA Sports",
    "BioWare",
    "Respawn Entertainment",
    "DICE",
    "Maxis",
    "Criterion Games",
    "EA Vancouver",
    "PopCap Games",
    "Codemasters",
  ],
  franchises: [
    "Mass Effect",
    "Dragon Age",
    "Apex Legends",
    "Titanfall",
    "Battlefield",
    "Star Wars Jedi",
    "The Sims",
    "Dead Space",
    "Need for Speed",
    "FIFA",
    "EA SPORTS FC",
    "Madden NFL",
    "Command & Conquer",
    "Mirror's Edge",
    "Burnout",
    "Crysis",
    "Skate",
  ],
  primaryGenres: ["Shooter", "RPG", "Sports", "Racing", "Action-Adventure"],
  website: "https://www.ea.com",
  games: [
    {
      title: "Mass Effect Legendary Edition",
      aliases: ["Mass Effect", "MELE"],
      franchise: "Mass Effect",
      developers: ["BioWare"],
      publishers: ["Electronic Arts"],
      genres: ["RPG", "Sci-Fi", "Third-Person Shooter", "Story Rich"],
      releaseDate: "2021-05-14",
      releaseYear: 2021,
      shortDescription:
        "The Mass Effect Legendary Edition includes single-player base content and over 40 DLC from Mass Effect, Mass Effect 2, and Mass Effect 3 games, including promo weapons, armors and packs.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1328670/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1328670/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1328670/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1328670/header.jpg",
    },
    {
      title: "Star Wars Jedi: Survivor",
      aliases: ["Jedi Survivor"],
      franchise: "Star Wars Jedi",
      developers: ["Respawn Entertainment"],
      publishers: ["Electronic Arts"],
      genres: ["Action-Adventure", "Souls-like", "Sci-Fi"],
      releaseDate: "2023-04-28",
      releaseYear: 2023,
      shortDescription:
        "The story of Cal Kestis continues in Star Wars Jedi: Survivor, a third-person, galaxy-spanning, action-adventure game from Respawn Entertainment.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1774580/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1774580/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1774580/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1774580/header.jpg",
    },
  ],
};
