#!/usr/bin/env node
// Deterministic, agent-friendly test & verification runner.
//
// Runs the project's suites directly (no shell, no nested `npm`), with stdin
// closed so no step can block waiting for input, a per-step timeout so a wedged
// tool is killed instead of hanging the session, live streaming plus a
// timestamped transcript in `logs/`, and a compact PASS/FAIL summary with a
// reliable exit code. Prefer this over invoking the tools by hand.
//
// Usage:
//   node scripts/test.mjs [suite ...] [--timeout <seconds>] [--list] [-- <cargo args>]
//
// Suites: fmt, clippy, check, rust, frontend, build, all
// Default: check, rust, frontend (see lib/suites.mjs)

import { ALL_SUITES, DEFAULT_SUITES, SUITES, runSuites } from "./lib/suites.mjs";

const argv = process.argv.slice(2);
const names = [];
let timeoutSeconds = 900;
let list = false;
let extraArgs = [];

for (let i = 0; i < argv.length; i += 1) {
  const arg = argv[i];
  if (arg === "--") {
    extraArgs = argv.slice(i + 1);
    break;
  }
  if (arg === "--list" || arg === "-l") {
    list = true;
    continue;
  }
  if (arg === "--timeout" || arg === "-t") {
    const value = Number(argv[i + 1]);
    if (!Number.isFinite(value) || value <= 0) {
      console.error("--timeout expects a number of seconds greater than 0");
      process.exit(2);
    }
    timeoutSeconds = value;
    i += 1;
    continue;
  }
  if (arg.startsWith("--")) {
    console.error(`unknown option: ${arg}`);
    process.exit(2);
  }
  names.push(arg);
}

if (list) {
  console.log("Available suites:");
  for (const [key, suite] of Object.entries(SUITES)) {
    console.log(`  ${key.padEnd(9)} ${suite.title}`);
  }
  console.log(`\nDefault: ${DEFAULT_SUITES.join(", ")}`);
  console.log(`All:     ${ALL_SUITES.join(", ")}`);
  process.exit(0);
}

const selected = [
  ...new Set(names.length === 0 ? DEFAULT_SUITES : names.flatMap((name) => (name === "all" ? ALL_SUITES : [name])))
];

const unknown = selected.filter((name) => !SUITES[name]);
if (unknown.length > 0) {
  console.error(`unknown suite(s): ${unknown.join(", ")} — run with --list`);
  process.exit(2);
}

const started = Date.now();
const { ok, results } = await runSuites(selected, { timeoutMs: timeoutSeconds * 1000, extraArgs });

console.log(`\n${"─".repeat(56)}`);
for (const result of results) {
  const status = result.ok ? "PASS" : "FAIL";
  const seconds = (result.ms / 1000).toFixed(1).padStart(6);
  console.log(` ${status}  ${result.name.padEnd(9)} ${seconds}s  ${result.title}`);
}
console.log("─".repeat(56));

const total = ((Date.now() - started) / 1000).toFixed(1);
if (results.length === 0) {
  console.log("No suites ran.");
  process.exitCode = 0;
} else {
  console.log(`${ok ? "✓" : "✗"} ${ok ? "all suites passed" : "one or more suites failed"} in ${total}s`);
  process.exitCode = ok ? 0 : 1;
}
