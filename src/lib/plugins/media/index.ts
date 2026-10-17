import type { UniversalPlugin } from "../framework";
import { mediaUniversalPlugin } from "./universal";

export * from "./universal";

export const MEDIA_PLUGINS: UniversalPlugin[] = [
  mediaUniversalPlugin
];
