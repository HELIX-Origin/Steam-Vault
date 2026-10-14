// Spawn helper shared by the command scripts.
//
// Runs a child process with its output streamed to the console and recorded to
// a timestamped transcript (see `logger.mjs`), forwarding termination signals
// so interactive commands such as `tauri dev` stop cleanly on Ctrl+C.

import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { ROOT, createLogger } from "./logger.mjs";

/** Local Tauri CLI entry point, run through `node` for portability. */
const TAURI_CLI = path.join(ROOT, "node_modules", "@tauri-apps", "cli", "tauri.js");

/** Signals forwarded from this wrapper to the child process. */
const FORWARDED_SIGNALS = ["SIGINT", "SIGTERM"];

/** Quote a single argument for a shell command line. */
function quoteArg(value) {
  const text = String(value);
  if (text === "") return '""';
  return /[\s"&|<>^()%!]/.test(text) ? `"${text.replace(/"/g, '\\"')}"` : text;
}

/**
 * Run a command, streaming output to the console and a timestamped transcript.
 *
 * @param {object} options
 * @param {string} options.type      transcript category, e.g. `tauri-dev`
 * @param {string} options.command   executable to run
 * @param {string[]} [options.args]  arguments passed to the executable
 * @param {string} [options.cwd]     working directory (defaults to the repo root)
 * @param {NodeJS.ProcessEnv} [options.env]
 * @param {boolean} [options.shell]  run through the shell (resolves `npm` on Windows)
 * @returns {Promise<number>} the child's exit code
 */
export async function runLogged({
  type,
  command,
  args = [],
  cwd = ROOT,
  env = process.env,
  shell = false
}) {
  const logger = createLogger(type);
  logger.header({ command, args, cwd });
  logger.info(`logging to ${logger.relative}`);

  return new Promise((resolve) => {
    const finish = async (code, signal, error = null) => {
      for (const signalName of FORWARDED_SIGNALS) process.removeAllListeners(signalName);
      await logger.footer({ code, signal, error });
      resolve(error ? 1 : (code ?? (signal ? 1 : 0)));
    };

    // With `shell` we pass one pre-quoted command line instead of an args array
    // (Node warns about, and does not escape, args combined with `shell: true`).
    const spawnCommand = shell ? [command, ...args].map(quoteArg).join(" ") : command;
    const spawnArgs = shell ? [] : args;

    let child;
    try {
      child = spawn(spawnCommand, spawnArgs, { cwd, env, shell, stdio: ["inherit", "pipe", "pipe"] });
    } catch (error) {
      logger.error(`failed to start ${command}: ${error.message}`);
      void finish(null, null, error);
      return;
    }

    child.stdout.on("data", (data) => logger.chunk(data, false));
    child.stderr.on("data", (data) => logger.chunk(data, true));

    for (const signalName of FORWARDED_SIGNALS) {
      process.on(signalName, () => {
        if (!child.killed) child.kill(signalName);
      });
    }

    child.on("error", (error) => {
      logger.error(`failed to start ${command}: ${error.message}`);
      void finish(null, null, error);
    });
    child.on("close", (code, signal) => void finish(code, signal));
  });
}

/**
 * Run a Tauri CLI subcommand (`dev`, `build`, …) with logging.
 *
 * @param {string} subcommand   e.g. `dev` or `build`
 * @param {string[]} [args]     extra arguments forwarded to the CLI
 * @param {string} [type]       transcript category (defaults to `tauri-<subcommand>`)
 */
export function runTauri(subcommand, args = [], type = `tauri-${subcommand}`) {
  if (!fs.existsSync(TAURI_CLI)) {
    console.error(
      `tauri: local CLI not found at ${path.relative(ROOT, TAURI_CLI)} — run \`npm install\` first`
    );
    return Promise.resolve(1);
  }
  return runLogged({ type, command: process.execPath, args: [TAURI_CLI, subcommand, ...args] });
}
