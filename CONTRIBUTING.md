# Contributing to Steam Vault

Thanks for your interest in improving Steam Vault!

## Development setup

1. Install [Rust](https://rustup.rs/) (stable), [Node.js](https://nodejs.org/) 18+, and the
   [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform.
2. `npm install`
3. `npm run tauri:dev`

## Quality gate

Before opening a pull request, everything below must pass with **zero** errors:

```bash
npm install
npm run build                                          # required before cargo: Tauri reads build/
npm test                                               # cargo check + cargo test + svelte-check
```

For the stricter gate — formatting, Clippy, and a fresh production build as well:

```bash
npm run test:all
```

> The Rust build embeds the compiled frontend, so `npm run build` must run before the `cargo`
> commands on a fresh checkout (otherwise Tauri's `frontendDist` (`build/`) does not exist yet).

## Guidelines

- Keep Rust code formatted (`cargo fmt`) and warning-free where practical.
- Never commit API keys, tokens, or personal file paths.
- Version bumps must move `package.json`, `src-tauri/tauri.conf.json`, and
  `src-tauri/Cargo.toml` together.
- Prefer documented, stable data sources over scraping.

## Commit style

Concise, imperative subject lines (e.g. `add SteamGridDB artwork client`).
