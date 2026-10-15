// Shared transcript writer for the command scripts in `scripts/`.
//
// Every logged run writes `<type>-<timestamp>.log` under `logs/` so consecutive
// runs never overwrite each other, while the run's output is still streamed
// live to the console. The on-disk copy is stripped of ANSI escape sequences so
// the transcript stays readable in any editor.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/** Repository root (this file lives in `<root>/scripts/lib`). */
export const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

/** Directory that holds the timestamped transcripts. */
export const LOG_DIR = path.join(ROOT, "logs");

// CSI and OSC escape sequences, used to keep the transcript colour-free.
const ANSI = /\u001B(?:\[[0-9;?]*[ -/]*[@-~]|\][^\u0007]*(?:\u0007|\u001B\\))/g;

/** Local, filesystem-safe timestamp: `2026-10-14_15-20-26`. */
export function timestamp(date = new Date()) {
  const pad = (value) => String(value).padStart(2, "0");
  return (
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}` +
    `_${pad(date.getHours())}-${pad(date.getMinutes())}-${pad(date.getSeconds())}`
  );
}

/** Remove ANSI escape sequences from a string. */
export function stripAnsi(text) {
  return String(text).replace(ANSI, "");
}

/** Default maximum number of `.log` files kept in `logs/`. */
export const DEFAULT_MAX_LOG_FILES = 25;

/**
 * Remove oldest `.log` files in `LOG_DIR` when the count exceeds `maxFiles`.
 *
 * `README.md` and any non-`.log` files are strictly preserved.
 *
 * @param {number} [maxFiles] Maximum log files to retain (default: 25, or `MAX_LOG_FILES` env).
 * @returns {number} Number of deleted stale log files.
 */
export function cleanupLogs(maxFiles = Number(process.env.MAX_LOG_FILES) || DEFAULT_MAX_LOG_FILES) {
  if (!fs.existsSync(LOG_DIR)) return 0;

  try {
    const entries = fs
      .readdirSync(LOG_DIR, { withFileTypes: true })
      .filter((entry) => entry.isFile() && entry.name.endsWith(".log"))
      .map((entry) => {
        const fullPath = path.join(LOG_DIR, entry.name);
        try {
          const stats = fs.statSync(fullPath);
          return { name: entry.name, path: fullPath, mtimeMs: stats.mtimeMs };
        } catch {
          return null;
        }
      })
      .filter(Boolean);

    // Newest first
    entries.sort((a, b) => b.mtimeMs - a.mtimeMs);

    let deleted = 0;
    if (entries.length > maxFiles) {
      const stale = entries.slice(maxFiles);
      for (const item of stale) {
        try {
          fs.unlinkSync(item.path);
          deleted++;
        } catch {
          // ignore concurrent unlink issues
        }
      }
    }
    return deleted;
  } catch {
    return 0;
  }
}

/**
 * Open a transcript writer for a single command run.
 *
 * @param {string} type short log category, e.g. `tauri-dev`
 */
export function createLogger(type) {
  fs.mkdirSync(LOG_DIR, { recursive: true });
  cleanupLogs();

  const started = new Date();
  const file = path.join(LOG_DIR, `${type}-${timestamp(started)}.log`);
  const stream = fs.createWriteStream(file, { flags: "a" });


  const note = (line) => stream.write(`${line}\n`);
  const label = `${type}:`;

  return {
    /** Absolute path of the transcript being written. */
    file,
    /** Transcript path relative to the repo root, for tidy console output. */
    relative: path.relative(ROOT, file),

    /** Write the run's metadata header to the transcript. */
    header({ command, args = [], cwd = ROOT }) {
      const field = (name, value) => note(`${name.padEnd(9)} ${value}`);
      note(`# ${type} @ ${started.toISOString()}`);
      field("command", [command, ...args].join(" "));
      field("cwd", cwd);
      field("node", process.version);
      field("platform", `${process.platform} ${process.arch}`);
      note("");
    },

    info(message) {
      console.log(`${label} ${message}`);
      note(`${message}`);
    },

    warn(message) {
      console.warn(`${label} ${message}`);
      note(`WARN ${message}`);
    },

    error(message) {
      console.error(`${label} ${message}`);
      note(`ERROR ${message}`);
    },

    /** Mirror a child process chunk to the console and the transcript. */
    chunk(data, isError = false) {
      const text = data.toString();
      if (isError) process.stderr.write(text);
      else process.stdout.write(text);
      stream.write(stripAnsi(text).replace(/\r\n/g, "\n"));
    },

    /** Write the closing block and flush the transcript. */
    async footer({ code = null, signal = null, error = null, timedOut = false } = {}) {
      const ended = new Date();
      const seconds = ((ended - started) / 1000).toFixed(1);
      note("");
      note(`# finished @ ${ended.toISOString()} (${seconds}s)`);
      note(
        `# exit: ${
          error
            ? `error: ${error.message}`
            : timedOut
              ? "timeout"
              : signal
                ? `signal ${signal}`
                : `code ${code}`
        }`
      );
      await new Promise((resolve) => stream.end(resolve));
      cleanupLogs();
    }
  };
}
