//! Definitions of known major game developers, publishers, and flagship franchises.
//!
//! Provides authoritative knowledge of major studios, their subsidiaries/aliases,
//! flagship franchises, and primary genres. Used to:
//! 1. Validate that metadata candidates match the target game's developer and franchise,
//!    preventing cross-studio mismatches (e.g. assigning Ys VIII / Nihon Falcom artwork to
//!    Final Fantasy XIV / Square Enix).
//! 2. Provide authoritative backup metadata (developers, publishers, genres, franchises)
//!    for local and standalone games when external storefronts return incomplete or zero results.

use crate::plugins::title_normalizer::canonical_title_key;
use crate::steam::models::AppMeta;

#[derive(Debug, Clone)]
pub struct HandRolledGame {
    pub title: &'static str,
    pub aliases: &'static [&'static str],
    pub franchise: &'static str,
    pub developers: &'static [&'static str],
    pub publishers: &'static [&'static str],
    pub genres: &'static [&'static str],
    pub tags: &'static [&'static str],
    pub release_date: Option<&'static str>,
    pub release_year: Option<i32>,
    pub short_description: Option<&'static str>,
    pub capsule_image: Option<&'static str>,
    pub hero_image: Option<&'static str>,
    pub logo_image: Option<&'static str>,
    pub header_image: Option<&'static str>,
    pub screenshots: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub struct KnownDeveloper {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub franchises: &'static [&'static str],
    pub primary_genres: &'static [&'static str],
}

pub static KNOWN_DEVELOPERS: &[KnownDeveloper] = &[
    KnownDeveloper {
        name: "Square Enix",
        aliases: &[
            "Square Enix",
            "Square",
            "SquareSoft",
            "Enix",
            "Eidos",
            "Luminous Productions",
            "Square Enix Holdings",
        ],
        franchises: &[
            "Final Fantasy",
            "Dragon Quest",
            "Kingdom Hearts",
            "Chrono Trigger",
            "Chrono Cross",
            "Chrono",
            "NieR",
            "Drakengard",
            "Secret of Mana",
            "Trials of Mana",
            "Visions of Mana",
            "Mana",
            "SaGa",
            "Romancing SaGa",
            "Deus Ex",
            "Tomb Raider",
            "Star Ocean",
            "Octopath Traveler",
            "Bravely Default",
            "Bravely Second",
            "Tactics Ogre",
            "Ogre Battle",
            "Life is Strange",
            "Valkyrie Profile",
            "Valkyrie Elysium",
            "Front Mission",
            "The World Ends with You",
            "Parasite Eve",
            "Just Cause",
            "Dungeon Siege",
            "Sleeping Dogs",
            "Harvestella",
            "Forspoken",
            "Balan Wonderworld",
            "Brave Fencer Musashi",
            "Vagrant Story",
            "Xenogears",
            "Live A Live",
            "Triangle Strategy",
        ],
        primary_genres: &["Role-playing (RPG)", "Action RPG", "JRPG", "Adventure"],
    },
    KnownDeveloper {
        name: "Nihon Falcom",
        aliases: &["Nihon Falcom", "Falcom", "NIS America", "XSEED Games"],
        franchises: &[
            "Ys",
            "The Legend of Heroes",
            "Trails in the Sky",
            "Trails of Cold Steel",
            "Trails from Zero",
            "Trails to Azure",
            "Trails through Daybreak",
            "Trails into Reverie",
            "Tokyo Xanadu",
            "Xanadu Next",
            "Xanadu",
            "Zwei",
            "Brandish",
            "Gurumin",
            "Vantage Master",
            "Sorcerian",
            "Dragon Slayer",
        ],
        primary_genres: &["Action RPG", "JRPG", "Role-playing (RPG)"],
    },
    KnownDeveloper {
        name: "Nintendo",
        aliases: &[
            "Nintendo",
            "Nintendo EPD",
            "HAL Laboratory",
            "Monolith Soft",
            "Game Freak",
            "Intelligent Systems",
            "Retro Studios",
            "Next Level Games",
            "Camelot Software Planning",
            "AlphaDream",
        ],
        franchises: &[
            "Super Mario",
            "Mario Bros",
            "Mario Kart",
            "Mario Party",
            "Paper Mario",
            "Mario & Luigi",
            "Luigi's Mansion",
            "The Legend of Zelda",
            "Zelda",
            "Pokemon",
            "Pokémon",
            "Metroid",
            "Metroid Prime",
            "Fire Emblem",
            "Kirby",
            "Donkey Kong",
            "Donkey Kong Country",
            "Xenoblade",
            "Xenoblade Chronicles",
            "Animal Crossing",
            "Super Smash Bros",
            "Smash Bros",
            "Pikmin",
            "Star Fox",
            "F-Zero",
            "Splatoon",
            "Golden Sun",
            "EarthBound",
            "Mother",
            "WarioWare",
            "Wario Land",
            "Rhythm Heaven",
            "Custom Robo",
            "Advance Wars",
            "Punch-Out!!",
            "Kid Icarus",
            "Captain Toad",
            "Yoshi's Island",
            "Yoshi",
            "Chibi-Robo!",
        ],
        primary_genres: &["Adventure", "Platformer", "Action-Adventure", "Role-playing (RPG)"],
    },
    KnownDeveloper {
        name: "Capcom",
        aliases: &[
            "Capcom",
            "Capcom Production Studio",
            "Clover Studio",
            "Capcom Vancouver",
        ],
        franchises: &[
            "Resident Evil",
            "Biohazard",
            "Monster Hunter",
            "Street Fighter",
            "Mega Man",
            "Rockman",
            "Devil May Cry",
            "Ace Attorney",
            "Phoenix Wright",
            "The Great Ace Attorney",
            "Dragon's Dogma",
            "Dead Rising",
            "Ghosts 'n Goblins",
            "Ghost Trick",
            "Onimusha",
            "Okami",
            "Breath of Fire",
            "Dino Crisis",
            "Marvel vs. Capcom",
            "Darkstalkers",
            "Viewtiful Joe",
            "Lost Planet",
            "Exoprimal",
            "Bionic Commando",
            "Strider",
            "Final Fight",
            "Kunitsu-Gami",
        ],
        primary_genres: &["Action", "Survival Horror", "Fighting", "Action RPG"],
    },
    KnownDeveloper {
        name: "Bandai Namco",
        aliases: &[
            "Bandai Namco",
            "Bandai",
            "Namco",
            "Bandai Namco Entertainment",
            "Bandai Namco Studios",
            "Namco Tales Studio",
            "CyberConnect2",
        ],
        franchises: &[
            "Tekken",
            "Tales of",
            "Tales of Arise",
            "Tales of Symphonia",
            "Tales of Vesperia",
            "Tales of Berseria",
            "Pac-Man",
            "Soulcalibur",
            "Soul Edge",
            "God Eater",
            "Digimon",
            "Dragon Ball",
            "Naruto",
            "Gundam",
            "Scarlet Nexus",
            "Code Vein",
            "Katamari Damacy",
            "We Love Katamari",
            "Klonoa",
            "Ace Combat",
            "Ridge Racer",
            "The Idolmaster",
            "Sword Art Online",
            "One Piece",
            "Little Nightmares",
            "Taiko no Tatsujin",
            "Blue Protocol",
            "Sand Land",
            "JoJo's Bizarre Adventure",
            "Ni no Kuni",
        ],
        primary_genres: &["Action RPG", "Fighting", "Anime", "Action"],
    },
    KnownDeveloper {
        name: "FromSoftware",
        aliases: &["FromSoftware", "From Software", "FromSoftware, Inc."],
        franchises: &[
            "Armored Core",
            "Dark Souls",
            "Bloodborne",
            "Sekiro",
            "Elden Ring",
            "Demon's Souls",
            "King's Field",
            "Tenchu",
            "Otogi",
            "Shadow Tower",
            "Echo Night",
        ],
        primary_genres: &["Action RPG", "Action", "Mecha", "Role-playing (RPG)"],
    },
    KnownDeveloper {
        name: "Sega",
        aliases: &[
            "Sega",
            "SEGA",
            "Atlus",
            "Ryu Ga Gotoku Studio",
            "Sonic Team",
            "Creative Assembly",
            "Relic Entertainment",
            "AM2",
            "Two Point Studios",
        ],
        franchises: &[
            "Sonic the Hedgehog",
            "Sonic",
            "Persona",
            "Shin Megami Tensei",
            "Yakuza",
            "Like a Dragon",
            "Judgment",
            "Lost Judgment",
            "Metaphor: ReFantazio",
            "Total War",
            "Phantasy Star",
            "Shenmue",
            "Bayonetta",
            "Super Monkey Ball",
            "Etrian Odyssey",
            "Virtua Fighter",
            "Crazy Taxi",
            "Jet Set Radio",
            "Valkyria Chronicles",
            "Football Manager",
            "Puyo Puyo",
            "Streets of Rage",
            "Golden Axe",
            "Soul Hackers",
            "Devil Survivor",
            "Catherine",
            "Two Point Hospital",
            "Two Point Campus",
            "Company of Heroes",
            "Warhammer 40,000: Dawn of War",
            "House of the Dead",
            "Panzer Dragoon",
            "Space Channel 5",
            "Shinobi",
            "Altered Beast",
        ],
        primary_genres: &["JRPG", "Action-Adventure", "Platformer", "Strategy"],
    },
    KnownDeveloper {
        name: "Konami",
        aliases: &[
            "Konami",
            "Konami Digital Entertainment",
            "Kojima Productions",
            "Team Silent",
        ],
        franchises: &[
            "Metal Gear",
            "Metal Gear Solid",
            "Silent Hill",
            "Castlevania",
            "Contra",
            "Suikoden",
            "Yu-Gi-Oh!",
            "Bomberman",
            "Gradius",
            "Zone of the Enders",
            "eFootball",
            "Pro Evolution Soccer",
            "Dance Dance Revolution",
            "Beatmania",
            "Frogger",
            "Mystical Ninja",
            "Goemon",
            "Boktai",
        ],
        primary_genres: &["Action", "Stealth", "Survival Horror", "JRPG"],
    },
    KnownDeveloper {
        name: "Valve",
        aliases: &["Valve", "Valve Corporation"],
        franchises: &[
            "Half-Life",
            "Portal",
            "Counter-Strike",
            "Left 4 Dead",
            "Team Fortress",
            "Dota",
            "Day of Defeat",
            "Ricochet",
            "Alien Swarm",
            "Deadlock",
            "Artifact",
            "Underlords",
        ],
        primary_genres: &["First-Person Shooter", "Puzzle", "Action"],
    },
    KnownDeveloper {
        name: "Bethesda",
        aliases: &[
            "Bethesda Softworks",
            "Bethesda Game Studios",
            "id Software",
            "Arkane Studios",
            "MachineGames",
            "Tango Gameworks",
            "ZeniMax Media",
        ],
        franchises: &[
            "The Elder Scrolls",
            "Skyrim",
            "Morrowind",
            "Oblivion",
            "Daggerfall",
            "Arena",
            "Fallout",
            "Doom",
            "Wolfenstein",
            "Dishonored",
            "Prey",
            "Quake",
            "Starfield",
            "Rage",
            "Deathloop",
            "The Evil Within",
            "Ghostwire: Tokyo",
            "Hi-Fi Rush",
            "Redfall",
            "Commander Keen",
        ],
        primary_genres: &["Role-playing (RPG)", "First-Person Shooter", "Action-Adventure"],
    },
    KnownDeveloper {
        name: "Rockstar Games",
        aliases: &["Rockstar Games", "Rockstar North", "Take-Two Interactive"],
        franchises: &[
            "Grand Theft Auto",
            "GTA",
            "Red Dead Redemption",
            "Red Dead",
            "Max Payne",
            "Bully",
            "Canis Canem Edit",
            "Midnight Club",
            "L.A. Noire",
            "Manhunt",
            "The Warriors",
        ],
        primary_genres: &["Open World", "Action-Adventure", "Shooter"],
    },
    KnownDeveloper {
        name: "Electronic Arts",
        aliases: &[
            "Electronic Arts",
            "EA",
            "EA Sports",
            "BioWare",
            "Respawn Entertainment",
            "DICE",
            "Criterion Games",
            "Maxis",
            "Motive Studio",
            "Codemasters",
        ],
        franchises: &[
            "Battlefield",
            "Mass Effect",
            "Dragon Age",
            "Dead Space",
            "Need for Speed",
            "The Sims",
            "Titanfall",
            "Apex Legends",
            "Star Wars Jedi",
            "Star Wars Battlefront",
            "Command & Conquer",
            "Mirror's Edge",
            "Skate",
            "Burnout",
            "Plants vs. Zombies",
            "Medal of Honor",
            "Crysis",
            "SimCity",
            "Spore",
            "Wild Hearts",
            "It Takes Two",
            "Unravel",
            "A Way Out",
            "Anthem",
            "F1",
            "GRID",
            "Dirt",
            "WRC",
        ],
        primary_genres: &["Action", "Shooter", "Role-playing (RPG)", "Racing"],
    },
    KnownDeveloper {
        name: "Ubisoft",
        aliases: &[
            "Ubisoft",
            "Ubisoft Montreal",
            "Ubisoft Quebec",
            "Ubisoft Toronto",
            "Massive Entertainment",
            "Ubisoft Montpellier",
            "Nadeo",
        ],
        franchises: &[
            "Assassin's Creed",
            "Far Cry",
            "Watch Dogs",
            "Tom Clancy's Splinter Cell",
            "Splinter Cell",
            "Tom Clancy's Ghost Recon",
            "Ghost Recon",
            "Tom Clancy's Rainbow Six",
            "Rainbow Six",
            "Tom Clancy's The Division",
            "The Division",
            "Rayman",
            "Prince of Persia",
            "Beyond Good & Evil",
            "Just Dance",
            "The Crew",
            "For Honor",
            "Rabbids",
            "Immortals Fenyx Rising",
            "Skull and Bones",
            "Star Wars Outlaws",
            "Avatar: Frontiers of Pandora",
            "Steep",
            "Riders Republic",
            "Trackmania",
            "Child of Light",
            "Valiant Hearts",
            "Mario + Rabbids",
            "Trials",
            "Grow Home",
        ],
        primary_genres: &["Action-Adventure", "Open World", "Shooter"],
    },
    KnownDeveloper {
        name: "Blizzard Entertainment",
        aliases: &[
            "Blizzard Entertainment",
            "Blizzard",
            "Activision",
            "Activision Blizzard",
            "Infinity Ward",
            "Treyarch",
            "Sledgehammer Games",
            "Toys for Bob",
            "Raven Software",
        ],
        franchises: &[
            "Warcraft",
            "World of Warcraft",
            "StarCraft",
            "Diablo",
            "Overwatch",
            "Hearthstone",
            "Heroes of the Storm",
            "Call of Duty",
            "Crash Bandicoot",
            "Spyro the Dragon",
            "Tony Hawk's",
            "Guitar Hero",
            "Pitfall",
        ],
        primary_genres: &["Action RPG", "MMORPG", "Shooter", "Strategy"],
    },
    KnownDeveloper {
        name: "Sony Interactive Entertainment",
        aliases: &[
            "Sony Interactive Entertainment",
            "PlayStation Studios",
            "Naughty Dog",
            "Insomniac Games",
            "Santa Monica Studio",
            "Sucker Punch Productions",
            "Guerrilla Games",
            "Polyphony Digital",
            "Housemarque",
            "Firesprite",
            "Bend Studio",
        ],
        franchises: &[
            "The Last of Us",
            "Uncharted",
            "God of War",
            "Marvel's Spider-Man",
            "Spider-Man",
            "Horizon Zero Dawn",
            "Horizon Forbidden West",
            "Ghost of Tsushima",
            "Ghost of Yotei",
            "Ratchet & Clank",
            "Gran Turismo",
            "inFamous",
            "Killzone",
            "Resistance",
            "Returnal",
            "Jak and Daxter",
            "Sly Cooper",
            "Days Gone",
            "Bloodborne",
            "Death Stranding",
            "Astro Bot",
            "Astro's Playroom",
            "Gravity Rush",
            "LittleBigPlanet",
            "Sackboy",
            "MediEvil",
            "Twisted Metal",
            "The Order: 1886",
            "Until Dawn",
            "Blood & Truth",
        ],
        primary_genres: &["Action-Adventure", "Narrative", "Action"],
    },
    KnownDeveloper {
        name: "Xbox Game Studios",
        aliases: &[
            "Xbox Game Studios",
            "Microsoft Studios",
            "343 Industries",
            "Halo Studios",
            "The Coalition",
            "Playground Games",
            "Turn 10 Studios",
            "Obsidian Entertainment",
            "Ninja Theory",
            "Rare",
            "Compulsion Games",
            "Double Fine Productions",
            "InXile Entertainment",
            "World's Edge",
        ],
        franchises: &[
            "Halo",
            "Gears of War",
            "Forza Horizon",
            "Forza Motorsport",
            "Fable",
            "Sea of Thieves",
            "Age of Empires",
            "Age of Mythology",
            "Avowed",
            "Grounded",
            "The Outer Worlds",
            "Pillars of Eternity",
            "Pentiment",
            "Psychonauts",
            "Hellblade",
            "Senua's Saga",
            "Perfect Dark",
            "Banjo-Kazooie",
            "Killer Instinct",
            "State of Decay",
            "Crackdown",
            "Battletoads",
            "Wasteland",
            "Clockwork Revolution",
            "South of Midnight",
            "Everwild",
            "Project Mara",
        ],
        primary_genres: &["Action-Adventure", "First-Person Shooter", "Racing", "Role-playing (RPG)"],
    },
    KnownDeveloper {
        name: "CD Projekt Red",
        aliases: &["CD Projekt Red", "CD PROJEKT RED", "CD Projekt"],
        franchises: &[
            "The Witcher",
            "Cyberpunk 2077",
            "Cyberpunk",
            "Gwent",
            "Thronebreaker",
        ],
        primary_genres: &["Role-playing (RPG)", "Open World", "Action RPG"],
    },
    KnownDeveloper {
        name: "Koei Tecmo",
        aliases: &["Koei Tecmo", "Koei", "Tecmo", "Team Ninja", "Gust", "Omega Force"],
        franchises: &[
            "Dynasty Warriors",
            "Samurai Warriors",
            "Warriors Orochi",
            "Ninja Gaiden",
            "Dead or Alive",
            "Nioh",
            "Wo Long: Fallen Dynasty",
            "Rise of the Ronin",
            "Atelier",
            "Fatal Frame",
            "Toukiden",
            "Nobunaga's Ambition",
            "Romance of the Three Kingdoms",
            "Bladestorm",
        ],
        primary_genres: &["Action", "Hack and Slash", "Action RPG", "JRPG"],
    },
    KnownDeveloper {
        name: "Spike Chunsoft",
        aliases: &["Spike Chunsoft", "Chunsoft", "Spike"],
        franchises: &[
            "Danganronpa",
            "Steins;Gate",
            "Zero Escape",
            "Nonary Games",
            "AI: The Somnium Files",
            "Shiren the Wanderer",
            "Mystery Dungeon",
            "Master Detective Archives: RAIN CODE",
            "Conception",
        ],
        primary_genres: &["Visual Novel", "Adventure", "Roguelike"],
    },
    KnownDeveloper {
        name: "Remedy Entertainment",
        aliases: &["Remedy Entertainment", "Remedy"],
        franchises: &[
            "Alan Wake",
            "Control",
            "Quantum Break",
            "Max Payne",
            "Death Rally",
        ],
        primary_genres: &["Action-Adventure", "Psychological Thriller", "Third-Person Shooter"],
    },
    KnownDeveloper {
        name: "IO Interactive",
        aliases: &["IO Interactive", "IOI"],
        franchises: &[
            "Hitman",
            "Kane & Lynch",
            "Freedom Fighters",
            "Mini Ninjas",
            "Project 007",
        ],
        primary_genres: &["Stealth", "Action"],
    },
    KnownDeveloper {
        name: "Proxxie",
        aliases: &["Proxxie", "ProxxieGames"],
        franchises: &["Relicts of Aeson", "Aeson"],
        primary_genres: &["Role-playing (RPG)", "Adult", "Adventure", "Fantasy"],
    },
    KnownDeveloper {
        name: "Sierra Lee",
        aliases: &["Sierra Lee", "SierraLee", "SierraLeeGames"],
        franchises: &["The Last Sovereign", "Third Crisis"],
        primary_genres: &["Role-playing (RPG)", "Adult", "Strategy", "Political", "Adventure"],
    },
    KnownDeveloper {
        name: "Neyor",
        aliases: &["Neyor", "Neyor Games", "NeyorGames"],
        franchises: &["Crossdressing in Camelot", "Camelot"],
        primary_genres: &["Visual Novel", "Adult", "Comedy", "Fantasy", "Romance"],
    },
    KnownDeveloper {
        name: "Kompas Productions",
        aliases: &["Kompas Productions", "Kompas"],
        franchises: &["My Futa Family"],
        primary_genres: &["Visual Novel", "Adult", "Comedy", "Slice of Life"],
    },
    KnownDeveloper {
        name: "NLT Media",
        aliases: &["NLT Media", "NLT", "NLT-Media"],
        franchises: &["The Genesis Order", "Treasure of Nadia", "Lust Epidemic"],
        primary_genres: &["Adventure", "Adult", "Puzzle", "Mystery"],
    },
    KnownDeveloper {
        name: "Caribdis",
        aliases: &["Caribdis", "CaribdisGames"],
        franchises: &["Eternum", "Once in a Lifetime"],
        primary_genres: &["Visual Novel", "Adult", "Cyberpunk", "Comedy", "Sci-Fi"],
    },
    KnownDeveloper {
        name: "Dr PinkCake",
        aliases: &["Dr PinkCake", "DrPinkCake", "Dr. PinkCake"],
        franchises: &["Being a DIK", "Acting Lessons"],
        primary_genres: &["Visual Novel", "Adult", "Comedy", "College", "Drama"],
    },
];

pub static HAND_ROLLED_GAMES: &[HandRolledGame] = &[
    HandRolledGame {
        title: "FINAL FANTASY XIV - A Realm Reborn",
        aliases: &[
            "Final Fantasy XIV",
            "FFXIV",
            "FF14",
            "Final Fantasy XIV Online",
            "FINAL FANTASY XIV: A Realm Reborn",
        ],
        franchise: "Final Fantasy",
        developers: &["Square Enix"],
        publishers: &["Square Enix"],
        genres: &["MMORPG", "Role-playing (RPG)", "Adventure", "JRPG"],
        tags: &["MMORPG", "Story Rich", "Great Soundtrack", "JRPG", "Multiplayer", "Fantasy"],
        release_date: Some("2013-08-27"),
        release_year: Some(2013),
        short_description: Some(
            "Take part in an epic and ever-changing FINAL FANTASY as you adventure and explore with friends from around the world. Experience all the hallmarks of the best-selling franchise - airships, chocobos, moogles, and more."
        ),
        capsule_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/39210/library_600x900_2x.jpg"),
        hero_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/39210/library_hero.jpg"),
        logo_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/39210/logo.png"),
        header_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/39210/header.jpg"),
        screenshots: &[
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/39210/ss_a70cbc9141d47a6e97f03da381a05bda44ceb847.1920x1080.jpg?t=1782870674",
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/39210/ss_2899439a93eb6a2997d79c9cc90ff4a647d0cd43.1920x1080.jpg?t=1782870674",
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/39210/ss_d9c35de6f0fcce01750553bbdeb5e2957ffa2f4a.1920x1080.jpg?t=1782870674",
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/39210/ss_7773904df857e65cc557a72d2026bafd5d1e1095.1920x1080.jpg?t=1782870674",
        ],
    },
    HandRolledGame {
        title: "Ys VIII: Lacrimosa of DANA",
        aliases: &["Ys VIII", "Ys 8", "Ys VIII: Lacrimosa of DANA"],
        franchise: "Ys",
        developers: &["Nihon Falcom"],
        publishers: &["NIS America, Inc."],
        genres: &["Action RPG", "JRPG", "Adventure", "Hack and Slash"],
        tags: &["Action RPG", "Great Soundtrack", "JRPG", "Story Rich", "Anime", "Exploration"],
        release_date: Some("2018-04-16"),
        release_year: Some(2018),
        short_description: Some(
            "Adol awakens shipwrecked and stranded on a cursed island. There, he and the other stranded passengers he rescues form a village to challenge fearsome beasts and mysterious ruins on the isolated island."
        ),
        capsule_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/579180/library_600x900_2x.jpg"),
        hero_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/579180/library_hero.jpg"),
        logo_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/579180/logo.png"),
        header_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/579180/header.jpg"),
        screenshots: &[
            "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/579180/ss_e0dc65fbcaec4dbf84f183ee28e360b64be87d85.1920x1080.jpg",
        ],
    },
    HandRolledGame {
        title: "Crossdressing in Camelot",
        aliases: &[
            "Crossdressing in Camelot",
            "Crossdressing in Camelot: The Tale of Arthur",
            "CiC",
        ],
        franchise: "Crossdressing in Camelot",
        developers: &["Neyor", "Stickyicky"],
        publishers: &["Neyor"],
        genres: &["Visual Novel", "Adult", "Comedy", "Fantasy", "Romance", "Role-playing (RPG)"],
        tags: &[
            "Visual Novel",
            "Adult",
            "Crossdressing",
            "Fantasy",
            "Comedy",
            "Medieval",
            "Arthurian Legend",
            "Story Rich",
            "RPG Maker",
        ],
        release_date: Some("2021-08-01"),
        release_year: Some(2021),
        short_description: Some(
            "An Arthurian fantasy romantic visual novel following young Arthur as he navigates court intrigue, knights, and humor through unexpected disguise in the kingdom of Camelot."
        ),
        capsule_image: Some("https://img.itch.zone/aW1nLzUyMDYzNzMucG5n/original/BFhdzF.png"),
        hero_image: Some("https://img.itch.zone/aW1nLzUyMDc4OTkucG5n/original/qsvL07.png"),
        logo_image: None,
        header_image: Some("https://img.itch.zone/aW1nLzUyMDYzNzMucG5n/508x254%23mb/ySPYtS.png"),
        screenshots: &[
            "https://img.itch.zone/aW1hZ2UvOTE5Nzk3LzEwMDI0MjA1LnBuZw==/original/kaDsdv.png",
            "https://img.itch.zone/aW1hZ2UvOTE5Nzk3LzEwMDI0MjA3LnBuZw==/original/JFVdMh.png",
            "https://img.itch.zone/aW1hZ2UvOTE5Nzk3LzEwMjg2ODEwLnBuZw==/original/4B2msJ.png",
            "https://img.itch.zone/aW1hZ2UvOTE5Nzk3LzEwMjg2ODEzLnBuZw==/original/Lx2N3l.png",
        ],
    },
    HandRolledGame {
        title: "Relicts of Aeson",
        aliases: &[
            "Relicts of Aeson",
            "Relicts of Aeson v0.8",
            "Relicts of Aeson v0.9",
            "Relicts of Aeson v1.0",
            "relicts-of-aeson",
        ],
        franchise: "Relicts of Aeson",
        developers: &["Proxxie", "Doianu"],
        publishers: &["Proxxie"],
        genres: &["Role-playing (RPG)", "Adult", "Stealth", "Adventure", "Fantasy"],
        tags: &["RPG", "Stealth", "Thief", "Fantasy", "Story Rich", "Adult", "Adventure"],
        release_date: Some("2021-05-01"),
        release_year: Some(2021),
        short_description: Some(
            "An expansive fantasy stealth RPG following young master thief Ariana as a mysterious desire-altering curse strikes the remote medieval village of Pylewood."
        ),
        capsule_image: None,
        hero_image: None,
        logo_image: None,
        header_image: None,
        screenshots: &[],
    },
    HandRolledGame {
        title: "The Last Sovereign",
        aliases: &["TLS", "The Last Sovereign"],
        franchise: "The Last Sovereign",
        developers: &["Sierra Lee"],
        publishers: &["Sierra Lee"],
        genres: &["Role-playing (RPG)", "Political", "Adult", "Turn-Based Strategy", "Adventure"],
        tags: &["RPG", "Political", "Story Rich", "Adult", "Turn-Based Combat", "Choices Matter", "Fantasy"],
        release_date: Some("2014-06-01"),
        release_year: Some(2014),
        short_description: Some(
            "An expansive political fantasy RPG chronicling the journey of the Last Sovereign, balancing economic investment, military strategy, and deep character development."
        ),
        capsule_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/951830/library_600x900_2x.jpg"),
        hero_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/951830/library_hero.jpg"),
        logo_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/951830/logo.png"),
        header_image: Some("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/951830/header.jpg?t=1773094439"),
        screenshots: &[
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/951830/ss_5886907db1c9d247eb159ef317c9a201ab98f557.1920x1080.jpg?t=1773094439",
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/951830/ss_8cbc2ae0f628c9909faa1b53fa09889f755f7a34.1920x1080.jpg?t=1773094439",
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/951830/ss_037788bbd565e87beac64988f278258bd0c09c27.1920x1080.jpg?t=1773094439",
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/951830/ss_1176bb7aac9bcb38f6e59ed9ad4e98226654c1c6.1920x1080.jpg?t=1773094439",
        ],
    },
    HandRolledGame {
        title: "Third Crisis",
        aliases: &["Third Crisis"],
        franchise: "Third Crisis",
        developers: &["Sierra Lee", "Anduo Games"],
        publishers: &["Sierra Lee", "Anduo Games", "Lewdonymous"],
        genres: &["Role-playing (RPG)", "Adult", "Cyberpunk", "Turn-Based Strategy", "Tactical"],
        tags: &["Cyberpunk", "Adult", "Tactical", "Story Rich", "Turn-Based Combat", "RPG"],
        release_date: Some("2024-05-19"),
        release_year: Some(2024),
        short_description: Some(
            "A cyberpunk adult RPG combining deep tactical combat, city management, and narrative-driven encounters."
        ),
        capsule_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1260820/library_600x900_2x.jpg"),
        hero_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1260820/library_hero.jpg"),
        logo_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1260820/logo.png"),
        header_image: Some("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1260820/header.jpg?t=1775119657"),
        screenshots: &[
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1260820/ss_567dd8093c0e11b01989f9c37d9733b8db680bab.1920x1080.jpg?t=1775119657",
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1260820/ss_3584cf0017339496558d61960f558293d09ae1f7.1920x1080.jpg?t=1775119657",
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1260820/ss_c51845011a42bb892ed934561226a1dbe0b64823.1920x1080.jpg?t=1775119657",
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1260820/ss_955641c40986f4d1a0fc260e7cbe228a6ac25084.1920x1080.jpg?t=1775119657",
        ],
    },
    HandRolledGame {
        title: "My Futa Family",
        aliases: &["My Futa Family", "MFF", "my-futa-family"],
        franchise: "My Futa Family",
        developers: &["Kompas Productions"],
        publishers: &["Kompas Productions"],
        genres: &["Visual Novel", "Adult", "Comedy", "Slice of Life", "Romance"],
        tags: &["Visual Novel", "Adult", "3D", "Comedy", "Slice of Life", "Story Rich", "Romance"],
        release_date: Some("2021-06-01"),
        release_year: Some(2021),
        short_description: Some(
            "A 3D renpy visual novel following the humorous daily lives, relationships, and romance of an unconventional household."
        ),
        capsule_image: None,
        hero_image: None,
        logo_image: None,
        header_image: None,
        screenshots: &[],
    },
    HandRolledGame {
        title: "The Genesis Order",
        aliases: &["The Genesis Order", "Genesis Order"],
        franchise: "The Genesis Order",
        developers: &["NLT Media"],
        publishers: &["NLT Media"],
        genres: &["Adventure", "Adult", "Puzzle", "Detective"],
        tags: &["Adventure", "Adult", "Puzzle", "Mystery", "Detective", "Point & Click", "Story Rich"],
        release_date: Some("2022-01-01"),
        release_year: Some(2022),
        short_description: Some(
            "The Genesis Order is the latest chapter in NLT's saga which started with Lust Epidemic and continued in Treasure of Nadia. Follow junior detective William through mystery and supernatural intrigue."
        ),
        capsule_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2553870/library_600x900_2x.jpg"),
        hero_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2553870/library_hero.jpg"),
        logo_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2553870/logo.png"),
        header_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/2553870/header.jpg"),
        screenshots: &[
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/2553870/ss_8ca2f326fb51df315ca4086ea76412e612cb7427.1920x1080.jpg",
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/2553870/ss_49392e6d62a9313ea595d7eb6b5dfa91bca7e8e5.1920x1080.jpg",
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/2553870/ss_eb859598ee40cf1d3d6dd129a0ebf9ee29b79878.1920x1080.jpg",
        ],
    },
    HandRolledGame {
        title: "Treasure of Nadia",
        aliases: &["Treasure of Nadia"],
        franchise: "Treasure of Nadia",
        developers: &["NLT Media"],
        publishers: &["NLT Media"],
        genres: &["Adventure", "Adult", "Puzzle"],
        tags: &["Adventure", "Adult", "Puzzle", "Exploration", "Crafting", "Story Rich"],
        release_date: Some("2020-02-14"),
        release_year: Some(2020),
        short_description: Some(
            "Treasure of Nadia is the direct follow up to Lust Epidemic. Take control of a young man looking to follow in his late father’s footsteps and become a famous treasure hunter."
        ),
        capsule_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1737100/library_600x900_2x.jpg"),
        hero_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1737100/library_hero.jpg"),
        logo_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1737100/logo.png"),
        header_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1737100/header.jpg"),
        screenshots: &[
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1737100/ss_81e8557b4458f40776b772ae841bcfae53610fb1.1920x1080.jpg",
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1737100/ss_752a78ae3e858db14b14dcae4d6d67cfca46e4d4.1920x1080.jpg",
        ],
    },
    HandRolledGame {
        title: "Eternum",
        aliases: &["Eternum", "Eternum: Season 1"],
        franchise: "Eternum",
        developers: &["Caribdis"],
        publishers: &["Caribdis"],
        genres: &["Visual Novel", "Adult", "Cyberpunk", "Sci-Fi", "Comedy", "Adventure"],
        tags: &["Visual Novel", "Adult", "Cyberpunk", "Comedy", "Sci-Fi", "Virtual Reality", "Choices Matter", "Story Rich"],
        release_date: Some("2022-04-01"),
        release_year: Some(2022),
        short_description: Some(
            "You move to the city of Kredon with your best friend, where you discover Eternum, the revolutionary and immersive virtual reality game."
        ),
        capsule_image: None,
        hero_image: None,
        logo_image: None,
        header_image: None,
        screenshots: &[],
    },
    HandRolledGame {
        title: "Being a DIK",
        aliases: &[
            "Being a DIK",
            "Being a DIK - Season 1",
            "Being a DIK - Season 2",
            "Being a DIK - Season 3",
            "Being a DIK: Season 1",
            "Being a DIK: Season 2",
            "Being a DIK: Season 3",
        ],
        franchise: "Being a DIK",
        developers: &["Dr PinkCake"],
        publishers: &["Dr PinkCake"],
        genres: &["Visual Novel", "Adult", "Comedy", "Story Rich", "Romance"],
        tags: &["Visual Novel", "Adult", "Choices Matter", "Comedy", "Story Rich", "Romance", "Dating Sim"],
        release_date: Some("2020-02-14"),
        release_year: Some(2020),
        short_description: Some(
            "Being a DIK is a choices matter, story-rich adult visual novel about a young man moving away from his hometown to attend Bresha University."
        ),
        capsule_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1126320/library_600x900_2x.jpg"),
        hero_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1126320/library_hero.jpg"),
        logo_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1126320/logo.png"),
        header_image: Some("https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/1126320/header.jpg"),
        screenshots: &[
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1126320/ss_8e59ec5e8105d15cb1d2448373b5d2ba1d09e3e7.1920x1080.jpg",
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1126320/ss_e1be5f31e3d640fa886ef664a75323bb6ebaf795.1920x1080.jpg",
        ],
    },
];

/// Detect if a given game title belongs to any known major franchise or developer.
pub fn detect_known_developer(title: &str) -> Option<&'static KnownDeveloper> {
    let lower = title.to_ascii_lowercase();
    let norm_key = canonical_title_key(title);

    for dev in KNOWN_DEVELOPERS {
        for fran in dev.franchises {
            let fran_lower = fran.to_ascii_lowercase();
            let fran_key = canonical_title_key(fran);

            // Exact or word-boundary match for the franchise
            if lower.starts_with(&fran_lower)
                || lower.contains(&format!(" {} ", fran_lower))
                || lower.ends_with(&format!(" {}", fran_lower))
                || lower.starts_with(&format!("{}:", fran_lower))
                || lower.starts_with(&format!("{} -", fran_lower))
                || (norm_key.starts_with(&fran_key) && fran_key.len() >= 3)
            {
                return Some(dev);
            }
        }
    }
    None
}

/// Check if developer/publisher strings from candidate metadata match a known developer.
pub fn matches_known_developer(dev: &KnownDeveloper, candidate_name: &str) -> bool {
    let c_lower = candidate_name.trim().to_ascii_lowercase();
    for alias in dev.aliases {
        let a_lower = alias.to_ascii_lowercase();
        if c_lower == a_lower || c_lower.contains(&a_lower) || a_lower.contains(&c_lower) {
            return true;
        }
    }
    false
}

/// Validates whether a candidate metadata result is consistent with the target title and its known developer.
///
/// Prevents cross-studio mismatches (e.g. Ys VIII from Nihon Falcom being matched to Final Fantasy XIV from Square Enix).
pub fn validate_metadata_candidate(
    target_title: &str,
    candidate_title: &str,
    candidate_developers: &[String],
    candidate_publishers: &[String],
) -> bool {
    let target_dev = detect_known_developer(target_title);
    let candidate_dev = detect_known_developer(candidate_title);

    // If both titles are identified with known major developers and they DIFFER, reject immediately!
    if let (Some(t_dev), Some(c_dev)) = (target_dev, candidate_dev) {
        if t_dev.name != c_dev.name {
            return false;
        }
    }

    // If target belongs to a known developer, ensure candidate developers/publishers do not belong to another major studio
    if let Some(t_dev) = target_dev {
        for cand_dev_str in candidate_developers.iter().chain(candidate_publishers.iter()) {
            for other_dev in KNOWN_DEVELOPERS {
                if other_dev.name != t_dev.name && matches_known_developer(other_dev, cand_dev_str) {
                    return false;
                }
            }
        }
    }

    true
}

/// Finds a specific hand-rolled game from the authoritative catalog.
pub fn find_hand_rolled_game(title: &str) -> Option<&'static HandRolledGame> {
    let t_key = canonical_title_key(title);
    if t_key.is_empty() {
        return None;
    }
    for game in HAND_ROLLED_GAMES {
        if canonical_title_key(game.title) == t_key {
            return Some(game);
        }
        for alias in game.aliases {
            if canonical_title_key(alias) == t_key {
                return Some(game);
            }
        }
    }
    None
}

/// Provides authoritative backup metadata (and curated artwork where available) for a title
/// using the hand-rolled games database and known developer definitions.
pub fn backup_metadata_for_title(title: &str) -> Option<AppMeta> {
    if let Some(game) = find_hand_rolled_game(title) {
        return Some(AppMeta {
            name: Some(game.title.to_string()),
            developers: game.developers.iter().map(|&s| s.to_string()).collect(),
            publishers: game.publishers.iter().map(|&s| s.to_string()).collect(),
            genres: game.genres.iter().map(|&s| s.to_string()).collect(),
            tags: game.tags.iter().map(|&s| s.to_string()).collect(),
            release_date: game.release_date.map(str::to_string),
            release_year: game.release_year,
            short_description: game.short_description.map(str::to_string),
            capsule_image: game.capsule_image.map(str::to_string),
            hero_image: game.hero_image.map(str::to_string),
            logo_image: game.logo_image.map(str::to_string),
            header_image: game.header_image.map(str::to_string),
            screenshots: game.screenshots.iter().map(|&s| s.to_string()).collect(),
            user_enhanced: Some(true),
            enhanced_by: vec!["Developer Hand-Rolled Database".to_string()],
            ..Default::default()
        });
    }

    let dev = detect_known_developer(title)?;
    Some(AppMeta {
        name: Some(title.to_string()),
        developers: vec![dev.name.to_string()],
        publishers: vec![dev.name.to_string()],
        genres: dev.primary_genres.iter().map(|&s| s.to_string()).collect(),
        user_enhanced: Some(true),
        enhanced_by: vec!["Developer Hand-Rolled Database".to_string()],
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_developer_detection() {
        let ffxiv = detect_known_developer("FINAL FANTASY XIV - A Realm Reborn");
        assert!(ffxiv.is_some());
        assert_eq!(ffxiv.unwrap().name, "Square Enix");

        let ys = detect_known_developer("Ys VIII: Lacrimosa of DANA");
        assert!(ys.is_some());
        assert_eq!(ys.unwrap().name, "Nihon Falcom");

        let mario = detect_known_developer("Super Mario Sunshine");
        assert!(mario.is_some());
        assert_eq!(mario.unwrap().name, "Nintendo");
    }

    #[test]
    fn test_metadata_candidate_validation() {
        // Final Fantasy XIV target cannot match Ys VIII candidate
        assert!(!validate_metadata_candidate(
            "FINAL FANTASY XIV - A Realm Reborn",
            "Ys VIII: Lacrimosa of DANA",
            &["Nihon Falcom".to_string()],
            &["NIS America".to_string()]
        ));

        // Final Fantasy XIV target can match genuine Final Fantasy candidate
        assert!(validate_metadata_candidate(
            "FINAL FANTASY XIV - A Realm Reborn",
            "Final Fantasy XIV Online",
            &["Square Enix".to_string()],
            &["Square Enix".to_string()]
        ));

        // Unknown indie game candidate is allowed if no conflict
        assert!(validate_metadata_candidate(
            "My Custom Indie Game",
            "My Custom Indie Game",
            &["Indie Dev".to_string()],
            &["Indie Publisher".to_string()]
        ));
    }

    #[test]
    fn test_backup_metadata() {
        let backup = backup_metadata_for_title("FINAL FANTASY XIV - A Realm Reborn");
        assert!(backup.is_some());
        let meta = backup.unwrap();
        assert_eq!(meta.developers, vec!["Square Enix"]);
        assert_eq!(meta.publishers, vec!["Square Enix"]);
        assert!(meta.genres.contains(&"Role-playing (RPG)".to_string()));
        assert!(meta.capsule_image.is_some());
    }

    #[test]
    fn test_hand_rolled_games_and_indie_developers() {
        // Crossdressing in Camelot (Neyor / Stickyicky)
        let cic = find_hand_rolled_game("Crossdressing in Camelot");
        assert!(cic.is_some());
        assert_eq!(cic.unwrap().developers, &["Neyor", "Stickyicky"]);
        assert_eq!(cic.unwrap().franchise, "Crossdressing in Camelot");

        let cic_backup = backup_metadata_for_title("Crossdressing in Camelot");
        assert!(cic_backup.is_some());
        let cic_meta = cic_backup.unwrap();
        assert!(cic_meta.developers.contains(&"Neyor".to_string()));
        assert!(cic_meta.developers.contains(&"Stickyicky".to_string()));
        assert!(cic_meta.genres.contains(&"Visual Novel".to_string()));

        // Relicts of Aeson (Proxxie / Doianu)
        let roa = find_hand_rolled_game("Relicts of Aeson");
        assert!(roa.is_some());
        assert_eq!(roa.unwrap().developers, &["Proxxie", "Doianu"]);

        // The Last Sovereign (Sierra Lee)
        let tls = find_hand_rolled_game("The Last Sovereign");
        assert!(tls.is_some());
        assert_eq!(tls.unwrap().developers, &["Sierra Lee"]);

        // Ys VIII (Nihon Falcom)
        let ys = find_hand_rolled_game("Ys VIII: Lacrimosa of DANA");
        assert!(ys.is_some());
        assert_eq!(ys.unwrap().developers, &["Nihon Falcom"]);
        assert!(ys.unwrap().capsule_image.is_some());
    }
}
