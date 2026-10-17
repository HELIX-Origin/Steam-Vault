import type { DeveloperDefinition } from "./types";

export const rockstar: DeveloperDefinition = {
  id: "rockstar",
  name: "Rockstar Games",
  aliases: [
    "Rockstar Games",
    "Rockstar North",
    "Rockstar",
    "Rockstar San Diego",
    "Rockstar New York",
    "Rockstar Toronto",
  ],
  franchises: [
    "Grand Theft Auto",
    "GTA",
    "Red Dead Redemption",
    "Red Dead",
    "Max Payne",
    "Bully",
    "L.A. Noire",
    "Midnight Club",
    "The Warriors",
    "Manhunt",
  ],
  primaryGenres: ["Open World", "Action-Adventure", "Crime", "Third-Person Shooter"],
  website: "https://www.rockstargames.com",
  games: [
    {
      title: "Grand Theft Auto V",
      aliases: ["GTA V", "GTA 5", "Grand Theft Auto 5"],
      franchise: "Grand Theft Auto",
      developers: ["Rockstar North"],
      publishers: ["Rockstar Games"],
      genres: ["Open World", "Action-Adventure", "Crime", "Multiplayer"],
      releaseDate: "2015-04-14",
      releaseYear: 2015,
      shortDescription:
        "When a young street hustler, a retired bank robber and a terrifying psychopath land themselves in trouble with the most frightening elements of the criminal underworld, they must pull off a series of dangerous heists.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/271590/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/271590/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/271590/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/271590/header.jpg",
    },
    {
      title: "Red Dead Redemption 2",
      aliases: ["RDR2", "Red Dead 2"],
      franchise: "Red Dead Redemption",
      developers: ["Rockstar Games"],
      publishers: ["Rockstar Games"],
      genres: ["Open World", "Western", "Story Rich", "Action-Adventure"],
      releaseDate: "2019-12-05",
      releaseYear: 2019,
      shortDescription:
        "Winner of over 175 Game of the Year Awards and recipient of over 250 perfect scores, RDR2 is the epic tale of outlaw Arthur Morgan and the infamous Van der Linde gang on the run across America at the dawn of the modern age.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1174180/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1174180/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1174180/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1174180/header.jpg",
    },
  ],
};
