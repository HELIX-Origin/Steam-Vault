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
    url: https://github.com/HELIX-Origin/Steam-Vault
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
standalone local executables and indie titles.
</p>
<p>
It serves as a clean-room, modernized revival of the library organization philosophies of Depressurizer:
build and edit custom collections, filter with an advanced Allowed / Required / Excluded builder,
auto-categorize by genre, developer, publisher, platform, and tags, and manage local storage across drives.
</p>
<p>
Built with <strong>Tauri v2</strong> (Rust backend and SvelteKit frontend), Universal Game Launcher
relies strictly on <strong>official client protocols and documented local data sources</strong> rather than
scraping or credential harvesting, delivering instant performance, zero-friction privacy, and full offline usability.
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
<span class="feature-card__icon" aria-hidden="true">🚀</span>
<h3>Library Grid Launcher</h3>
<p>Browse your multi-platform library with vertical capsule covers, platform badges, install pills, and instant one-click launch controls.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🔒</span>
<h3>Native Launcher Delegation</h3>
<p>Launches, updates, and uninstalls delegate directly to official clients, preserving overlays, cloud saves, friend lists, and achievements without storing credentials.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">📑</span>
<h3>Per-Game Details &amp; Management</h3>
<p>Extensive details sub-pages featuring a 10-image media gallery with fullscreen lightbox, meta links, playtime tracking, and management tools.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎛️</span>
<h3>Collapsible Filter Sidebar</h3>
<p>Toggle sort options, platform counts, install state, and collections instantly from a dedicated right sidebar.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎁</span>
<h3>Free Games, Deals &amp; Patch Notes</h3>
<p>Track verified 100% free giveaways, store specials across Steam/Epic/GOG, and official changelogs for installed games.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🗂️</span>
<h3>Custom Collections</h3>
<p>Read and write categorized collections with favorites and hidden states, plus native smart collections.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🧩</span>
<h3>Multi-Scope Organize Lists</h3>
<p>Select multiple organization scopes simultaneously with custom rule builders to categorise your entire library.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🧠</span>
<h3>AutoCats</h3>
<p>Generate categories automatically from genre, developer, publisher, tags, and storefront flags.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">💾</span>
<h3>Storage Statistics</h3>
<p>Dedicated drive capacity and usage gauges with Steam Library Folder management and safe cross-drive moves.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎮</span>
<h3>All Major Storefronts</h3>
<p>Automatic detection across Steam, GOG Galaxy, Epic Games, itch.io, Xbox, EA App, Ubisoft, and Battle.net.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">📁</span>
<h3>Manually Added Local Games</h3>
<p>Easily import and manage standalone executables, DRM-free games, and indie projects with custom artwork and tags.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎨</span>
<h3>Artwork Manager</h3>
<p>Inspect, fetch, and set portrait grid, hero, logo, and icon artwork via SteamGridDB and local file imports.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🌍</span>
<h3>Multi-Language Support</h3>
<p>Hand-rolled reactive i18n supporting 28+ languages with Arabic RTL layout and zero runtime overhead.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">⚡</span>
<h3>Generic SQLite Persistence</h3>
<p>High-performance embedded SQLite database (<code>database.sqlite</code>) in WAL mode decoupled from application branding.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🛡️</span>
<h3>Safe writes</h3>
<p>Timestamped backups before every write, and changes are refused while platform launchers are running.</p>
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
Universal Game Launcher is distributed on the official Windows Package Manager (WinGet) and can be built from source today with Rust and Node.js.
See <a href="{{ '/docs/Getting-Started/' | relative_url }}">Getting Started</a> for the walkthrough.
</p>
<div class="header-actions">
<a class="button button--primary" href="{{ '/docs/Getting-Started/' | relative_url }}">Getting Started</a>
<a class="button button--secondary" href="https://github.com/HELIX-Origin/Steam-Vault">Source on GitHub ↗</a>
</div>
</section>
