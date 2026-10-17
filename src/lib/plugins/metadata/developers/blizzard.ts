import type { DeveloperDefinition } from "./types";

export const blizzard: DeveloperDefinition = {
  id: "blizzard",
  name: "Blizzard Entertainment",
  aliases: ["Blizzard", "Blizzard Entertainment", "Activision Blizzard", "Battle.net"],
  franchises: [
    "Warcraft",
    "World of Warcraft",
    "Diablo",
    "StarCraft",
    "Overwatch",
    "Hearthstone",
    "Heroes of the Storm",
  ],
  primaryGenres: ["MMORPG", "Action RPG", "Hero Shooter", "Real-Time Strategy", "Card Game"],
  website: "https://www.blizzard.com",
  games: [
    {
      title: "World of Warcraft",
      aliases: ["WoW", "World of Warcraft: The War Within", "World of Warcraft: Dragonflight"],
      franchise: "Warcraft",
      developers: ["Blizzard Entertainment"],
      publishers: ["Blizzard Entertainment"],
      genres: ["MMORPG", "Fantasy", "Multiplayer", "Adventure"],
      releaseDate: "2004-11-23",
      releaseYear: 2004,
      shortDescription:
        "World of Warcraft is a massively multiplayer online role-playing game set in the high-fantasy Warcraft universe.",
      capsuleImage: "https://images.igdb.com/igdb/image/upload/t_cover_big/co1r7f.jpg",
      heroImage: "https://images.igdb.com/igdb/image/upload/t_1080p/sc723w.jpg",
    },
    {
      title: "Diablo IV",
      aliases: ["Diablo 4", "D4"],
      franchise: "Diablo",
      developers: ["Blizzard Entertainment"],
      publishers: ["Blizzard Entertainment"],
      genres: ["Action RPG", "Hack and Slash", "Dark Fantasy", "Loot"],
      releaseDate: "2023-06-05",
      releaseYear: 2023,
      shortDescription:
        "Diablo IV is the next-gen action RPG experience with endless evil to slaughter, countless abilities to master, nightmarish dungeons, and legendary loot.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2344520/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2344520/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2344520/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2344520/header.jpg",
    },
    {
      title: "Overwatch 2",
      aliases: ["Overwatch", "OW2"],
      franchise: "Overwatch",
      developers: ["Blizzard Entertainment"],
      publishers: ["Blizzard Entertainment"],
      genres: ["Hero Shooter", "First-Person Shooter", "Multiplayer", "Competitive"],
      releaseDate: "2022-10-04",
      releaseYear: 2022,
      shortDescription:
        "Overwatch 2 is a critically acclaimed, team-based shooter game set in an optimistic future with an evolving roster of heroes.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2357570/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2357570/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2357570/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2357570/header.jpg",
    },
  ],
};
