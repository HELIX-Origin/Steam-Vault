import type { UniversalPlugin } from "../framework";
import { gamerpowerPlugin } from "./gamerpower";
import { storageAnalyzerPlugin } from "./storage-analyzer";
import { titleNormalizerPlugin } from "./title-normalizer";

export * from "./gamerpower";
export * from "./storage-analyzer";
export * from "./title-normalizer";

export const UTILITY_PLUGINS: UniversalPlugin[] = [
  titleNormalizerPlugin,
  gamerpowerPlugin,
  storageAnalyzerPlugin
];
