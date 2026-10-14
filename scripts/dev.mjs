#!/usr/bin/env node
// Run `tauri dev` with a timestamped transcript under `logs/`.
//
// Usage: npm run tauri:dev [-- <tauri dev args>]

import { runTauri } from "./lib/runner.mjs";

process.exitCode = await runTauri("dev", process.argv.slice(2), "tauri-dev");
