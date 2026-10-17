/**
 * Developer definition and hand-rolled game metadata schema.
 *
 * Each developer has its own data file (modeled after hand-rolled localization files)
 * providing authoritative studio information, aliases, flagship franchises, primary genres,
 * and curated hand-rolled metadata and artwork for major games.
 */

export interface HandRolledGame {
  /** The primary canonical title of the game */
  title: string;
  /** Alternate titles, acronyms, or edition names */
  aliases?: string[];
  /** Flagship franchise name (e.g. "Final Fantasy", "Ys", "Zelda") */
  franchise?: string;
  /** Primary developer names */
  developers?: string[];
  /** Primary publisher names */
  publishers?: string[];
  /** Primary genres */
  genres?: string[];
  /** Primary tags */
  tags?: string[];
  /** Release date (YYYY-MM-DD) */
  releaseDate?: string;
  /** Release year */
  releaseYear?: number;
  /** Brief synopsis */
  shortDescription?: string;
  /** Detailed markdown or text description */
  detailedDescription?: string;
  /** Age rating (e.g. "Teen", "Mature 17+", "PEGI 16", "Adults Only 18+") */
  ageRating?: string;
  /** Gameplay categories (e.g. "Single-player", "Multi-player", "Full controller support") */
  categories?: string[];
  /** Metascore or user rating out of 100 */
  userScore?: number;
  /** High-resolution 2:3 vertical capsule / boxart cover image URL */
  capsuleImage?: string;
  /** High-resolution 16:9 hero background banner image URL */
  heroImage?: string;
  /** Transparent PNG logo image URL */
  logoImage?: string;
  /** Landscape header image URL (460x215 or 16:9) */
  headerImage?: string;
  /** Square icon image URL */
  iconImage?: string;
  /** Curated high-resolution in-game screenshots */
  screenshots?: string[];
  /** Trailer MP4 / WebM video URLs */
  movies?: string[];
  /** Theme / soundtrack audio URLs */
  soundtracks?: string[];
}

export interface DeveloperDefinition {
  /** Unique developer identifier slug (e.g. "square-enix") */
  id: string;
  /** Canonical developer display name */
  name: string;
  /** Official studio aliases, historical names, and internal subsidiaries */
  aliases: string[];
  /** Major flagship franchises created or owned by this studio */
  franchises: string[];
  /** Standard genres associated with this developer's catalog */
  primaryGenres: string[];
  /** Official corporate or developer website */
  website?: string;
  /** Optional hand-rolled game metadata catalog (deprecated in favor of HELIX Gaming DB REST API) */
  games?: HandRolledGame[];
}
