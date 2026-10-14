---
layout: wiki
title: Development & Contributing
summary: Toolchain, quality gate, and workflow
parent: Steam Vault Technical Wiki
nav_order: 6
---

# Development & Contributing

Thanks for helping improve Steam Vault. This page covers the toolchain and the checks your change
must pass.

## 🧰 Toolchain

- **Rust** (stable) and Cargo
- **Node.js** 18+ and npm
- The [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS

## 🚦 Quality gate

Before opening a pull request, all of the following must pass with zero errors:

```bash
npm run check                                            # svelte-check: 0 errors, 0 warnings
cargo check --manifest-path src-tauri/Cargo.toml         # 0 Rust errors
cargo test  --manifest-path src-tauri/Cargo.toml         # all tests pass
```

> **Build order matters.** The Tauri build embeds the compiled frontend, so on a fresh clone run
> `npm install && npm run build` before a bare `cargo check`. The npm scripts (`npm run tauri:dev`,
> `npm run tauri:build`) handle the ordering for you.

## 🧾 Conventions

- Keep Rust formatted (`cargo fmt`) and warning-free where practical.
- Prefer documented, stable data sources over scraping.
- Never commit API keys, tokens, or personal file paths.
- Keep Steam knowledge inside `src-tauri/src/steam/` and filesystem work out of the frontend.

## 🔢 Versioning

Steam Vault follows [Semantic Versioning](https://semver.org/). A version bump must move all three
in lockstep:

- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`

## ✍️ Commit style

Use concise, imperative subject lines, for example:

```plaintext
add SteamGridDB artwork client
fix collection write guard when Steam is running
```

## 🔐 Reporting

- **Bugs and features** — open an issue on the repository tracker.
- **Security** — follow [SECURITY.md](https://github.com/HELIX-Origin/Steam-Vault/blob/main/SECURITY.md)
  and report privately.

## ➡️ Next

- [Architecture](Architecture.md)
- [Security](Security.md)
