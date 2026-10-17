import type { UniversalPlugin } from "../framework";
import { trailersPlayerPlugin } from "./trailers";
import { ambientSoundtracksPlugin } from "./ambient";

export * from "./trailers";
export * from "./ambient";

export const MEDIA_PLUGINS: UniversalPlugin[] = [
  trailersPlayerPlugin,
  ambientSoundtracksPlugin
];
