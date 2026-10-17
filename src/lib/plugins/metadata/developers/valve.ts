import type { DeveloperDefinition } from "./types";

export const valve: DeveloperDefinition = {
  id: "valve",
  name: "Valve",
  aliases: ["Valve", "Valve Corporation", "Valve Software"],
  franchises: [
    "Half-Life",
    "Portal",
    "Left 4 Dead",
    "Team Fortress",
    "Counter-Strike",
    "Dota",
    "Day of Defeat",
    "Deadlock",
    "Alien Swarm",
    "Ricochet",
  ],
  primaryGenres: ["First-Person Shooter", "Puzzle", "Multiplayer", "Action", "Sci-Fi"],
  website: "https://www.valvesoftware.com",
  games: [
    {
      title: "Half-Life 2",
      aliases: ["HL2", "Half Life 2"],
      franchise: "Half-Life",
      developers: ["Valve"],
      publishers: ["Valve"],
      genres: ["First-Person Shooter", "Sci-Fi", "Action", "Classic"],
      releaseDate: "2004-11-16",
      releaseYear: 2004,
      shortDescription:
        "1998. HALF-LIFE sends a shock through the game industry with its combination of pounding action and continuous, immersive storytelling. Valve's debut title wins more than 50 game-of-the-year awards on its way to being named 'Best PC Game Ever'.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/220/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/220/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/220/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/220/header.jpg",
    },
    {
      title: "Portal 2",
      aliases: ["Portal 2"],
      franchise: "Portal",
      developers: ["Valve"],
      publishers: ["Valve"],
      genres: ["Puzzle", "Sci-Fi", "Comedy", "Co-op"],
      releaseDate: "2011-04-19",
      releaseYear: 2011,
      shortDescription:
        "The 'Perpetual Testing Initiative' has been expanded to allow you to design co-op puzzles for you and your friends!",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/620/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/620/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/620/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/620/header.jpg",
    },
  ],
};
