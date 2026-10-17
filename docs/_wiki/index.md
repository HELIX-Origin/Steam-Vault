---
layout: wiki
title: Universal Game Launcher Technical Wiki
summary: Architecture, in-app modular plugin system, developer guides, Rust backend, SvelteKit frontend, and security model.
permalink: /wiki/
nav_order: 1
---

# Universal Game Launcher Technical Wiki

This technical wiki documents how Universal Game Launcher is built: the decoupled core architecture, the in-app modular plugin system, the developer guides for authoring plugins, the Rust backend with generic SQLite persistence and fast-tier in-memory KV caching, the SvelteKit frontend, and the non-invasive security and privacy model.

---

<section class="doc-section">
  <div class="section-heading">
    <span class="section-number">01</span>
    <div>
      <p class="eyebrow">Architecture &amp; Core Engine</p>
      <h2>Core Internals</h2>
    </div>
  </div>
  <p>
    How the core application is structured and decoupled from features:
  </p>
  <ul>
    <li><a href="{{ '/wiki/Architecture/' | relative_url }}">Architecture</a> — core launcher responsibilities, Universal Core plugins boundaries, and module map.</li>
    <li><a href="{{ '/wiki/Plugins-Architecture/' | relative_url }}">Plugins Architecture</a> — Universal Core plugins framework, global helper modules, automatic multi-source metadata cascade, and tabbed settings dialog.</li>
    <li><a href="{{ '/wiki/Plugin-Developer-Guide/' | relative_url }}">Plugin Developer Guide</a> — comprehensive step-by-step developer tutorial with TypeScript templates for Universal Core Platform, Metadata, Utility, and Media plugins.</li>
  </ul>
</section>

<section class="doc-section">
  <div class="section-heading">
    <span class="section-number">02</span>
    <div>
      <p class="eyebrow">Implementation</p>
      <h2>Backend &amp; Frontend</h2>
    </div>
  </div>
  <ul>
    <li><a href="{{ '/wiki/Backend-Rust/' | relative_url }}">Backend (Rust)</a> — the Tauri v2 command surface, multi-source metadata engine, title normalizer backend, in-memory KV tier (`mem_kv`), and SQLite persistence (`database.sqlite`).</li>
    <li><a href="{{ '/wiki/Frontend-SvelteKit/' | relative_url }}">Frontend (SvelteKit)</a> — frameless titlebar navigation, Library-only sidebar games list with authentic vector SVG platform icons, unified media showcase, per-game customization modal, and `PluginSettingsDialog`.</li>
  </ul>
</section>

<section class="doc-section">
  <div class="section-heading">
    <span class="section-number">03</span>
    <div>
      <p class="eyebrow">Project</p>
      <h2>Development &amp; Assurance</h2>
    </div>
  </div>
  <ul>
    <li><a href="{{ '/wiki/Development-and-Contributing/' | relative_url }}">Development &amp; Contributing</a> — building from source, quality gates, authoring plugins, and verification workflows.</li>
    <li><a href="{{ '/wiki/Security/' | relative_url }}">Security</a> — non-invasive invariants, zero credential harvesting, and secrets handling.</li>
    <li><a href="{{ '/wiki/Privacy/' | relative_url }}">Privacy</a> — 100% on-device local operation and zero telemetry.</li>
  </ul>
</section>
