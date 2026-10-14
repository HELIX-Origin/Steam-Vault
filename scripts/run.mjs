#!/usr/bin/env node
// Generic logged command runner.
//
// Runs any command with its output recorded to a timestamped transcript under
// `logs/`. Use it for the commands that benefit from a persistent debug log.
//
// Usage:
//   node scripts/run.mjs <log-type> <command> [args...]
//
// Examples:
//   node scripts/run.mjs check npm run check
//   node scripts/run.mjs rust cargo check --manifest-path src-tauri/Cargo.toml

import { runLogged } from "./lib/runner.mjs";

const [type, command, ...args] = process.argv.slice(2);

if (!type || !command) {
  console.error("usage: node scripts/run.mjs <log-type> <command> [args...]");
  process.exit(2);
}

process.exitCode = await runLogged({ type, command, args, shell: true });
