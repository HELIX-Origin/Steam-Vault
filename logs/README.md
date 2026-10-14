# logs/

Timestamped transcripts written by the command scripts in [`../scripts/`](../scripts/).

- `npm run tauri:dev` and `npm run tauri:build` write
  `<log-type>-<YYYY-MM-DD_HH-MM-SS>.log` here through `scripts/dev.mjs` and
  `scripts/build.mjs`.
- Anything run through `node scripts/run.mjs <log-type> <command…>` is recorded
  the same way.
- The log type plus timestamp in the filename means consecutive runs never
  overwrite each other.
- This directory is git-ignored apart from this README, so transcripts stay
  local to your machine.
