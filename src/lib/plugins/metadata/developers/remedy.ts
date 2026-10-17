import type { DeveloperDefinition } from "./types";

export const remedy: DeveloperDefinition = {
  id: "remedy",
  name: "Remedy Entertainment",
  aliases: ["Remedy Entertainment", "Remedy"],
  franchises: ["Alan Wake", "Control", "Max Payne", "Quantum Break", "CrossfireX"],
  primaryGenres: ["Psychological Thriller", "Action-Adventure", "Surreal", "Third-Person Shooter"],
  website: "https://www.remedygames.com",
  games: [
    {
      title: "Control Ultimate Edition",
      aliases: ["Control"],
      franchise: "Control",
      developers: ["Remedy Entertainment"],
      publishers: ["505 Games"],
      genres: ["Action-Adventure", "Supernatural", "Sci-Fi", "Third-Person Shooter"],
      releaseDate: "2020-08-27",
      releaseYear: 2020,
      shortDescription:
        "Winner of over 80 awards, Control is a visually stunning third-person action-adventure that will keep you on the edge of your seat.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/870780/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/870780/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/870780/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/870780/header.jpg",
    },
    {
      title: "Alan Wake 2",
      aliases: ["Alan Wake II"],
      franchise: "Alan Wake",
      developers: ["Remedy Entertainment"],
      publishers: ["Epic Games Publishing"],
      genres: ["Survival Horror", "Psychological Horror", "Action-Adventure"],
      releaseDate: "2023-10-27",
      releaseYear: 2023,
      shortDescription:
        "Saga Anderson arrives to investigate ritualistic murders in a small town. Alan Wake pens a dark story to shape the reality around him.",
      capsuleImage: "https://images.igdb.com/igdb/image/upload/t_cover_big/co6eec.jpg",
      heroImage: "https://images.igdb.com/igdb/image/upload/t_1080p/scop1l.jpg",
    },
  ],
};
