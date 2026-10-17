import type { DeveloperDefinition } from "./types";

export const ubisoft: DeveloperDefinition = {
  id: "ubisoft",
  name: "Ubisoft",
  aliases: [
    "Ubisoft",
    "Ubisoft Montreal",
    "Ubisoft Paris",
    "Ubisoft Quebec",
    "Ubisoft Montpellier",
    "Massive Entertainment",
    "Red Storm",
    "Ubisoft Toronto",
  ],
  franchises: [
    "Assassin's Creed",
    "Far Cry",
    "Tom Clancy's",
    "Rainbow Six",
    "The Division",
    "Ghost Recon",
    "Splinter Cell",
    "Watch Dogs",
    "Rayman",
    "Prince of Persia",
    "Beyond Good & Evil",
    "Just Dance",
    "For Honor",
    "The Crew",
    "Immortals Fenyx Rising",
    "Driver",
    "Trials",
    "Rabbids",
    "Anno",
    "South Park",
    "Avatar: Frontiers of Pandora",
    "Star Wars Outlaws",
  ],
  primaryGenres: ["Open World", "Action-Adventure", "Stealth", "Shooter", "Tactical"],
  website: "https://www.ubisoft.com",
  games: [
    {
      title: "Assassin's Creed Mirage",
      aliases: ["AC Mirage"],
      franchise: "Assassin's Creed",
      developers: ["Ubisoft Bordeaux"],
      publishers: ["Ubisoft"],
      genres: ["Action-Adventure", "Stealth", "Open World", "Parkour"],
      releaseDate: "2023-10-05",
      releaseYear: 2023,
      shortDescription:
        "Experience the story of Basim, a cunning street thief seeking answers and justice as he navigates the bustling streets of ninth-century Baghdad.",
      capsuleImage: "https://images.igdb.com/igdb/image/upload/t_cover_big/co7v21.jpg",
      heroImage: "https://images.igdb.com/igdb/image/upload/t_1080p/scou6w.jpg",
    },
    {
      title: "Tom Clancy's Rainbow Six Siege",
      aliases: ["Rainbow Six Siege", "R6 Siege"],
      franchise: "Rainbow Six",
      developers: ["Ubisoft Montreal"],
      publishers: ["Ubisoft"],
      genres: ["Tactical Shooter", "Multiplayer", "Competitive", "FPS"],
      releaseDate: "2015-12-01",
      releaseYear: 2015,
      shortDescription:
        "Tom Clancy's Rainbow Six Siege is an elite, tactical team-based shooter where superior planning and execution triumph.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/359550/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/359550/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/359550/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/359550/header.jpg",
    },
  ],
};
