---
layout: wiki
title: Steam Vault Technical Wiki
summary: Architecture, the Steam data layer, the Rust backend, the SvelteKit frontend, and the security and privacy model.
permalink: /wiki/
nav_order: 1
---

# Steam Vault Technical Wiki

This wiki documents how Steam Vault is put together: where it reads and writes Steam data, how the
Rust backend is organized, how the frontend talks to it, and how the project handles safety,
security, and privacy.

<section class="doc-section">
  <div class="section-heading">
    <span class="section-number">01</span>
    <div>
      <p class="eyebrow">Architecture &amp; Data</p>
      <h2>Core Internals</h2>
    </div>
  </div>
  <p>
    How the application is structured and how it interacts with Steam's own files:
  </p>
  <ul>
    <li><a href="{{ '/wiki/Architecture/' | relative_url }}">Architecture</a> — the big picture and the module map.</li>
    <li><a href="{{ '/wiki/Steam-Data-Layer/' | relative_url }}">Steam Data Layer</a> — files read and written, formats, and write safety.</li>
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
    <li><a href="{{ '/wiki/Backend-Rust/' | relative_url }}">Backend (Rust)</a> — the Tauri command surface and modules.</li>
    <li><a href="{{ '/wiki/Frontend-SvelteKit/' | relative_url }}">Frontend (SvelteKit)</a> — the SPA, stores, and typed API layer.</li>
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
    <li><a href="{{ '/wiki/Development-and-Contributing/' | relative_url }}">Development &amp; Contributing</a> — the quality gate and workflow.</li>
    <li><a href="{{ '/wiki/Security/' | relative_url }}">Security</a> — secrets, local writes, and reporting.</li>
    <li><a href="{{ '/wiki/Privacy/' | relative_url }}">Privacy</a> — what leaves your machine (almost nothing).</li>
  </ul>
</section>

---

### 📚 Wiki Index
- **Core**: [Architecture](Architecture.md) · [Steam Data Layer](Steam-Data-Layer.md)
- **Implementation**: [Backend (Rust)](Backend-Rust.md) · [Frontend (SvelteKit)](Frontend-SvelteKit.md)
- **Project**: [Development & Contributing](Development-and-Contributing.md) · [Security](Security.md) · [Privacy](Privacy.md)

---

*[Steam Vault](https://github.com/HELIX-Origin/Steam-Vault) — a cross-platform Steam library organizer.*
