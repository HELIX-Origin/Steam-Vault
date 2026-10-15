// Test & verification suites for Steam Vault.
//
// Each suite is a short, ordered list of steps. Steps run directly (never
// through `npm`), so there is no nested-package-manager overhead or shell
// quoting to go wrong, and every step is launched with stdin closed and a hard
// timeout so a wedged tool can never hang a session (see `runner.mjs`).

import path from "node:path";
import { ROOT } from "./logger.mjs";
import { runLogged } from "./runner.mjs";

/** Rust crate directory (where `Cargo.toml` lives). */
const RUST_DIR = path.join(ROOT, "src-tauri");

/** A `cargo` step, run inside the Rust crate. */
function cargo(label, args, type) {
  return { label, command: "cargo", args, cwd: RUST_DIR, type };
}

/** A Node-based step (SvelteKit / svelte-check / Vite), run from the repo root. */
function nodeStep(label, entryRel, args, type) {
  const entry = path.join(ROOT, "node_modules", ...entryRel.split("/"));
  return { label, command: process.execPath, args: [entry, ...args], cwd: ROOT, type };
}

/**
 * The suite catalogue. Keys are the names accepted on the command line.
 */
export const SUITES = {
  fmt: {
    title: "Rust formatting",
    steps: [cargo("cargo fmt --check", ["fmt", "--all", "--", "--check"], "test-fmt")]
  },
  clippy: {
    title: "Rust lints",
    steps: [cargo("cargo clippy", ["clippy", "--all-targets", "--", "-D", "warnings"], "test-clippy")]
  },
  check: {
    title: "Rust type-check",
    steps: [cargo("cargo check", ["check", "--all-targets"], "test-check")]
  },
  rust: {
    title: "Rust tests",
    steps: [cargo("cargo test", ["test"], "test-rust")]
  },
  frontend: {
    title: "Frontend type-check",
    steps: [
      nodeStep("svelte-kit sync", "@sveltejs/kit/svelte-kit.js", ["sync"], "test-svelte-sync"),
      nodeStep("svelte-check", "svelte-check/bin/svelte-check", ["--tsconfig", "./tsconfig.json"], "test-svelte-check")
    ]
  },
  build: {
    title: "Frontend build",
    steps: [nodeStep("vite build", "vite/bin/vite.js", ["build"], "test-build")]
  }
};

/** Suites run when no names are given — the fast, everyday verification set. */
export const DEFAULT_SUITES = ["check", "rust", "frontend"];

/** Everything, in a sensible order (format, lint, types, tests, build). */
export const ALL_SUITES = ["fmt", "clippy", "check", "rust", "frontend", "build"];

/** Environment that keeps the toolchain quiet and non-interactive. */
function suiteEnv() {
  return {
    ...process.env,
    CI: "1",
    NO_COLOR: "1",
    FORCE_COLOR: "0",
    CARGO_TERM_COLOR: "never",
    CARGO_TERM_PROGRESS_WHEN: "never"
  };
}

/**
 * Run the named suites in order.
 *
 * @param {string[]} names         suite keys from {@link SUITES}
 * @param {object} [options]
 * @param {number} [options.timeoutMs]  per-step timeout (0 disables)
 * @param {string[]} [options.extraArgs] extra args appended to `cargo` steps
 * @returns {Promise<{ ok: boolean, results: Array<object> }>}
 */
export async function runSuites(names, { timeoutMs = 0, extraArgs = [] } = {}) {
  const env = suiteEnv();
  const results = [];

  for (const name of names) {
    const suite = SUITES[name];
    if (!suite) {
      console.error(`unknown suite: ${name} — run with --list`);
      return { ok: false, results };
    }

    console.log(`\n▶ ${name} — ${suite.title}`);
    const started = Date.now();
    let ok = true;
    let code = 0;

    for (const step of suite.steps) {
      const stepArgs = step.command === "cargo" ? [...step.args, ...extraArgs] : step.args;
      console.log(`  · ${step.label}`);
      code = await runLogged({
        type: step.type,
        command: step.command,
        args: stepArgs,
        cwd: step.cwd,
        env,
        stdin: "ignore",
        timeoutMs
      });
      if (code !== 0) {
        ok = false;
        break;
      }
    }

    results.push({ name, title: suite.title, ok, code, ms: Date.now() - started });
  }

  return { ok: results.every((result) => result.ok), results };
}
