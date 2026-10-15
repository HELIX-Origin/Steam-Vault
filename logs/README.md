# logs/

Timestamped transcripts written by the command scripts in [`../scripts/`](../scripts/).

- `npm run tauri:dev` and `npm run tauri:build` write
  `<log-type>-<YYYY-MM-DD_HH-MM-SS>.log` here through `scripts/dev.mjs` and
  `scripts/build.mjs`.
- The test & verification runner (`npm test` / `node scripts/test.mjs`) writes
  one transcript per step — `test-fmt-*`, `test-check-*`, `test-rust-*`,
  `test-svelte-check-*`, `test-build-*`, and so on.
- Anything run through `node scripts/run.mjs <log-type> <command…>` is recorded
  the same way.
- The log type plus timestamp in the filename means consecutive runs never
  overwrite each other.
- **Log cleanup & retention**: `scripts/lib/logger.mjs` automatically prunes stale
  log files on every run, keeping only the most recent 25 `.log` files (configurable
  via `MAX_LOG_FILES` environment variable). This ensures logs never consume excessive
  storage over time.
- This directory is git-ignored apart from this README, so transcripts stay
  local to your machine.

