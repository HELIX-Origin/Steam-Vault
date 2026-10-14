#!/usr/bin/env node
// Run `tauri build` with a timestamped transcript under `logs/`.
//
// Usage: npm run tauri:build [-- <tauri build args>]

import { runTauri } from "./lib/runner.mjs";

process.exitCode = await runTauri("build", process.argv.slice(2), "tauri-build");
