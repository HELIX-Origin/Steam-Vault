import type { DeveloperDefinition } from "./types";

export const koeiTecmo: DeveloperDefinition = {
  id: "koei-tecmo",
  name: "Koei Tecmo",
  aliases: ["Koei Tecmo", "Koei", "Tecmo", "Team Ninja", "Gust", "Omega Force", "KOEI TECMO GAMES CO., LTD."],
  franchises: [
    "Dynasty Warriors",
    "Samurai Warriors",
    "Warriors Orochi",
    "Ninja Gaiden",
    "Nioh",
    "Wo Long",
    "Atelier",
    "Fatal Frame",
    "Dead or Alive",
    "Toukiden",
    "Romance of the Three Kingdoms",
    "Nobunaga's Ambition",
    "Wild Hearts",
  ],
  primaryGenres: ["Hack and Slash", "Action RPG", "Souls-like", "Strategy", "JRPG"],
  website: "https://www.koeitecmo.co.jp",
  games: [
    {
      title: "Nioh 2 - The Complete Edition",
      aliases: ["Nioh 2"],
      franchise: "Nioh",
      developers: ["Team NINJA", "KOEI TECMO GAMES CO., LTD."],
      publishers: ["KOEI TECMO GAMES CO., LTD."],
      genres: ["Action RPG", "Souls-like", "Dark Fantasy", "Difficult"],
      releaseDate: "2021-02-05",
      releaseYear: 2021,
      shortDescription:
        "Battle hordes of yokai in this masocore Action RPG. Create your protagonist and embark on an adventure through a brutalized Sengoku-period Japan.",
      capsuleImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1325200/library_600x900_2x.jpg",
      heroImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1325200/library_hero.jpg",
      logoImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1325200/logo.png",
      headerImage:
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1325200/header.jpg",
    },
  ],
};
