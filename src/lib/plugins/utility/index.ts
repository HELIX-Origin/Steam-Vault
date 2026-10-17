import type { UniversalPlugin } from "../framework";
import { gamerpowerPlugin } from "./gamerpower";
import { storageAnalyzerPlugin } from "./storage-analyzer";

export * from "./gamerpower";
export * from "./storage-analyzer";

export const UTILITY_PLUGINS: UniversalPlugin[] = [
  gamerpowerPlugin,
  storageAnalyzerPlugin
];
