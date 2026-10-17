---
layout: wiki
title: Development & Contributing
summary: Toolchain, quality gate, conventions, and contributing workflows
parent: Universal Game Launcher Technical Wiki
nav_order: 6
---

# Development & Contributing

Thank you for contributing to Universal Game Launcher! This guide details the toolchain, quality gate, coding conventions, and pull request workflows.

---

## 🧰 Toolchain

- **Rust** (stable toolchain) and Cargo
- **Node.js 18+** and npm
- Platform [Tauri system dependencies](https://tauri.app/start/prerequisites/) (WebView2 on Windows, WebKitGTK on Linux)
- [Microsoft Visual C++ 2015–2022 Redistributable (x64)](https://aka.ms/vs/17/release/vc_redist.x64.exe) (Windows)

---

## 🚦 Quality Gate

Before opening a pull request, all automated checks must pass with zero errors and zero warnings:

```bash
# 1. Typecheck & Svelte diagnostics (must report 0 errors, 0 warnings)
npm run check

# 2. Rust check & unit test suite
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml

# 3. Production static asset build
npm run build
```

> **Build Order Matters:** The Tauri desktop binary embeds the compiled SvelteKit frontend. On a fresh clone, always run `npm install && npm run build` before running a bare `cargo check`. The npm scripts (`npm run tauri:dev`, `npm run tauri:build`) orchestrate this ordering automatically.

---

## 🧩 Authoring Plugins & Modules

When implementing or improving a platform integration or metadata provider:

1. **Dedicated Files Only**: Every plugin must live in its own dedicated file under `src/lib/plugins/platforms/<id>.ts` or `src/lib/plugins/metadata/<id>.ts`. Never combine multiple providers into monolithic files.
2. **Implement `UniversalPlugin`**: Implement the `UniversalPlugin` interface from `src/lib/plugins/framework.ts`.
3. **Integrate Full Client Features**: Platform plugins should implement all available client capabilities:
   - `getLaunchSpec`: Protocol URIs and CLI arguments.
   - `getInstallUri` / `getUninstallUri`: Storefront client install/uninstall protocols.
   - `getFriendsListUri`: Direct client social roster links.
   - `getAchievementsUri`: Client achievements view links.
   - `getDefaultKnownFolders`: Default installation search directories.
4. **Decoupled Settings**: If your plugin accepts user configuration or API keys, export a `getSettingsSchema()` defining typed fields (`text`, `password`, `boolean`, `select`, `path`) and implement `testConnection()` for live verification.
5. **Register Your Plugin**: Add your exported plugin to `PLATFORM_PLUGINS` or `METADATA_PLUGINS` in `src/lib/plugins/index.ts`.

---

## 🧾 Conventions

- **Code Formatting**: Ensure Rust code adheres to `cargo fmt` and TypeScript adheres to Prettier.
- **Strict Typing**: All frontend code must use TypeScript with strict null checks. Avoid `any` where typed interfaces can be declared.
- **Air-Gapped Invariant**: Built-in plugins must never require external downloaders or runtime code generation. All core logic runs offline.
- **Non-Invasive Invariant**: Never write directly to external storefront proprietary databases or cloud sync manifests (e.g. `shortcuts.vdf`).
- **Secrets & Credentials**: Never commit personal API keys, tokens, or local absolute paths to the repository.

---

## 🔢 Versioning

Universal Game Launcher follows [Semantic Versioning (SemVer)](https://semver.org/). Version bumps must update all three configuration manifests in lockstep:

- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`

---

## ✍️ Commit Message Guidelines

Use clear, imperative commit subjects:

```plaintext
feat(plugins): add Rockstar Games platform integration
fix(storage): resolve volume calculation on secondary APFS drive
docs(wiki): update in-app modular plugin development guide
```

---

## 🔐 Security & Vulnerability Reporting

- **Bugs & Feature Requests**: Open an issue on the [GitHub Issue Tracker](https://github.com/HELIX-Origin/Universal-Game-Launcher/issues).
- **Security Vulnerabilities**: Report security issues privately per the instructions in [SECURITY.md](https://github.com/HELIX-Origin/Universal-Game-Launcher/blob/main/SECURITY.md).

---

## ➡️ Next Steps

- [Architecture Overview](Architecture.md)
- [Plugins Architecture](Plugins-Architecture.md)
- [Security Model](Security.md)
