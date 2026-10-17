---
layout: default
title: Universal Game Launcher
eyebrow: Universal Game Launcher & Hub
summary: A fast, lightweight, privacy-first universal game launcher and library organizer across Steam, Epic, GOG, Xbox, EA, Ubisoft, and local games.
nav_order: 1
header_actions:
  - title: Read the docs
    url: /docs/
    class: button--primary
  - title: Source on GitHub ↗
    url: https://github.com/HELIX-Origin/Universal-Game-Launcher
    class: button--secondary
image: assets/images/universal-game-launcher.svg
---

<div markdown="0" class="site-banner-container" style="margin-bottom: 2.2rem; border-radius: 14px; overflow: hidden; border: 1px solid rgba(135, 195, 250, 0.25); box-shadow: 0 16px 48px rgba(0,0,0,0.5); line-height: 0; font-size: 0;"><img src="{{ '/assets/images/universal-game-launcher.svg' | relative_url }}" alt="Universal Game Launcher Banner" style="width: 100%; height: auto; display: block; margin: 0; padding: 0; border: none;" loading="lazy" decoding="async"></div>

<section id="about" class="doc-section">
<div class="section-heading">
<span class="section-number">01</span>
<div>
<p class="eyebrow">Overview</p>
<h2>What is Universal Game Launcher?</h2>
</div>
</div>
<p>
Universal Game Launcher brings all your PC games together into a single, unified cockpit.
It automatically discovers games across all installed official clients — including <strong>Steam</strong>,
<strong>Epic Games Store</strong>, <strong>GOG Galaxy</strong>, <strong>Ubisoft Connect</strong>,
<strong>EA App</strong>, <strong>Xbox / Microsoft Store</strong>, <strong>Battle.net</strong>,
<strong>itch.io</strong>, and <strong>Amazon Games</strong> — with full support for manually adding
standalone local executables, DRM-free games, and emulation / ROM libraries via drop-in plugins.
</p>
<p>
Designed for speed, privacy, and ergonomics, Universal Game Launcher adopts a proven <strong>Steam and GOG Galaxy-inspired layout</strong>:
top titlebar navigation, an interactive left game list sidebar with live filtering and platform accordions, and a dedicated showcase
view featuring hero artwork, media screenshot galleries, and one-click launch controls.
</p>
<p>
Built with <strong>Tauri v2</strong> (Rust backend and SvelteKit frontend), Universal Game Launcher operates completely non-invasively.
It never alters external client files, never harvests credentials, and persists all library overrides and artwork mappings locally
in a high-performance SQLite database (<code>database.sqlite</code>) backed by an in-memory serverless KV tier for sub-millisecond responsiveness.
</p>
</section>

<section id="features" class="doc-section">
<div class="section-heading">
<span class="section-number">02</span>
<div>
<p class="eyebrow">Highlights</p>
<h2>Features</h2>
</div>
</div>
<div class="features-grid">
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎮</span>
<h3>Steam &amp; GOG Left Games List Sidebar</h3>
<p>Interactive sidebar featuring instant name search, quick filter pills (All, Installed, ★ Favorites), background platform scan trigger, and collapsible platform and collection accordions.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🧭</span>
<h3>Frameless Title Bar Navigation</h3>
<p>Clean top-level navigation seamlessly integrated into the custom titlebar: Dashboard, Library, Storage, Plugins, and Settings.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🖼️</span>
<h3>Dedicated Game Showcase View</h3>
<p>Showcase selected games with high-resolution 16:9 hero wallpaper banners, title typography, prominent PLAY NOW launch buttons, playtime metrics, and descriptions.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">📸</span>
<h3>10-Image Screenshot Gallery</h3>
<p>Interactive media carousel featuring up to 10 high-resolution screenshots with a fullscreen lightbox viewer.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🧩</span>
<h3>Native In-App Modular Plugins Framework</h3>
<p>All game platforms, metadata scrapers, and utility tools are delivered via native modular plugins built directly into the app, eliminating Cloudflare network blocks and fragile external manifests.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">⚙️</span>
<h3>Tabbed Plugin Settings Sub-Window</h3>
<p>Dedicated sub-window dialog for configuring each enabled plugin, featuring live password visibility eye toggles, direct help links, and real-time in-memory connection tests.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🔒</span>
<h3>Native Launcher Delegation</h3>
<p>Launches, updates, and uninstalls delegate directly to official client protocols or native binaries, preserving overlays, cloud saves, friend lists, and achievements without storing credentials.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎁</span>
<h3>Free Games, Deals &amp; Patch Notes</h3>
<p>Track verified 100% free giveaways, store specials across major storefronts, and official changelogs for installed games on your Dashboard.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎨</span>
<h3>Custom Artwork Studio</h3>
<p>Inspect, fetch, and set portrait capsule covers, hero banners, transparent logos, and square icons via SteamGridDB and local image imports.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🗄️</span>
<h3>Custom &amp; Dynamic Collections</h3>
<p>Create custom game categories, tag favorites, hide titles, and build dynamic smart collections persisted safely in local SQLite.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">💾</span>
<h3>Storage Footprint Explorer</h3>
<p>Dedicated drive capacity and usage gauges showing disk space consumed across all connected volumes and platform libraries.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">⚡</span>
<h3>Zero-Lag Obsidian Purple Theme</h3>
<p>High-contrast obsidian purple visual identity (#080511 / #0f0920 / #a855f7) with hardware-accelerated rendering and zero <code>backdrop-filter: blur</code> performance overhead.</p>
</div>
</div>
</section>

<section id="providers" class="doc-section">
<div class="section-heading">
<span class="section-number">03</span>
<div>
<p class="eyebrow">Metadata</p>
<h2>Providers</h2>
</div>
</div>
<p>
Universal Game Launcher combines multiple metadata sources. Only <strong>Steam Web API</strong>,
<strong>SteamGridDB</strong>, and <strong>IGDB</strong> require optional API keys; all primary storefront integrations are keyless.
</p>
<table>
<thead>
<tr><th>Provider</th><th>Key Required</th><th>Provides</th></tr>
</thead>
<tbody>
<tr><td>Steam Store</td><td>No</td><td>Genres, developers, publishers, release dates, flags</td></tr>
<tr><td>Steam Store Search</td><td>No</td><td>Keyless title matching for standalone &amp; indie titles</td></tr>
<tr><td>GOG Store &amp; Galaxy DB</td><td>No</td><td>Official covers, heroes, and metadata via local SQLite galaxy DB</td></tr>
<tr><td>Epic Games Manifests</td><td>No</td><td>Title, install locations, app IDs, official launch parameters</td></tr>
<tr><td>Xbox AppModel</td><td>No</td><td>Registry and package manifest discovery with zero scraping</td></tr>
<tr><td>Steam Web API</td><td>Yes (Optional)</td><td>Owned games, playtime, player summaries</td></tr>
<tr><td>SteamGridDB</td><td>Yes (Optional)</td><td>Grids, heroes, logos, icons (especially for manually added local games)</td></tr>
<tr><td>IGDB (Twitch)</td><td>Yes (Optional)</td><td>Summary, genres, companies, cover</td></tr>
<tr><td>VNDB</td><td>No</td><td>Visual novel metadata, developers, release dates, tags</td></tr>
<tr><td>PCGamingWiki</td><td>No</td><td>Engine, developers, publishers, release dates</td></tr>
<tr><td>Local Sniffer</td><td>No</td><td>itch manifests, Ren'Py, RPG Maker, PE VersionInfo headers</td></tr>
<tr><td>Wikipedia</td><td>No</td><td>Infobox parser: developers, publishers, genres, release year, summaries</td></tr>
</tbody>
</table>
<p>
See <a href="{{ '/docs/Metadata-Providers/' | relative_url }}">Metadata Providers</a> for the full
reference.
</p>
</section>

<section id="docs" class="doc-section">
<div class="section-heading">
<span class="section-number">04</span>
<div>
<p class="eyebrow">Reference</p>
<h2>Documentation</h2>
</div>
</div>
<p>
Explore the complete guides and references on this site. The
<a href="{{ '/docs/' | relative_url }}">documentation directory</a> also links to every page.
</p>
<div class="documentation-directory">
<div>
<h3>User Guides</h3>
<ul>
{%- assign guide_pages = site.docs | sort: "nav_order" %}
{%- for doc in guide_pages %}
<li><a href="{{ doc.url | relative_url }}">{{ doc.title | escape }}</a></li>
{%- endfor %}
</ul>
</div>
<div>
<h3>Technical Wiki</h3>
<ul>
{%- assign wiki_pages = site.wiki | sort: "nav_order" %}
{%- for doc in wiki_pages %}
<li><a href="{{ doc.url | relative_url }}">{{ doc.title | escape }}</a></li>
{%- endfor %}
</ul>
</div>
</div>
</section>

<section id="build" class="doc-section">
<div class="section-heading">
<span class="section-number">05</span>
<div>
<p class="eyebrow">Install</p>
<h2>Get Universal Game Launcher</h2>
</div>
</div>
<p>
Universal Game Launcher is distributed as standalone installers via GitHub Releases and can be built from source today with Rust and Node.js.
See <a href="{{ '/docs/Getting-Started/' | relative_url }}">Getting Started</a> for the walkthrough.
</p>
<div class="header-actions">
<a class="button button--primary" href="{{ '/docs/Getting-Started/' | relative_url }}">Getting Started</a>
<a class="button button--secondary" href="https://github.com/HELIX-Origin/Universal-Game-Launcher">Source on GitHub ↗</a>
</div>
</section>
