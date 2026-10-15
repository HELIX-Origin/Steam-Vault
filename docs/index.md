---
layout: default
title: Steam Vault
eyebrow: Steam Library Organizer
summary: A clean-room, cross-platform revival of Depressurizer, built with Tauri v2. Organize, categorize, filter, and auto-manage your real Steam library.
nav_order: 1
header_actions:
  - title: Read the docs
    url: /docs/
    class: button--primary
  - title: Source on GitHub ↗
    url: https://github.com/HELIX-Origin/Steam-Vault
    class: button--secondary
image: assets/images/steam_banner.jpg
---

<div markdown="0" class="site-banner-container" style="margin-bottom: 2.2rem; border-radius: 14px; overflow: hidden; border: 1px solid rgba(135, 195, 250, 0.25); box-shadow: 0 16px 48px rgba(0,0,0,0.5); line-height: 0; font-size: 0;"><img src="{{ '/assets/images/steam_banner.jpg' | relative_url }}" alt="Steam Vault Banner" style="width: 100%; height: auto; display: block; margin: 0; padding: 0; border: none;" loading="lazy" decoding="async"></div>

<section id="about" class="doc-section">
<div class="section-heading">
<span class="section-number">01</span>
<div>
<p class="eyebrow">Overview</p>
<h2>What is Steam Vault?</h2>
</div>
</div>
<p>
Steam Vault reads your <strong>local Steam data</strong> and lets you organize your real Steam
library directly. It is a spiritual successor to Depressurizer: import your installed games,
detect titles installed by other launchers, sync your whole owned Steam library, build and edit
collections (categories), filter with an Allowed / Required / Excluded builder, and auto-categorize
by genre, developer, publisher, platform, tags, and feature flags.
</p>
<p>
Unlike its predecessor, Steam Vault is a modern, cross-platform <strong>Tauri v2</strong>
application (a Rust backend with a SvelteKit frontend) that targets <strong>stable, documented
data sources</strong> rather than fragile HTML scraping, so everyday Steam updates are far less
likely to break it.
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
<p>Browse your library with vertical capsule covers, platform badges, install pills, and instant one-click launch controls.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">📑</span>
<h3>Per-Game Details &amp; Management</h3>
<p>Extensive details sub-pages featuring a 10-image media gallery with fullscreen lightbox, meta links, playtime tracking, and management modal.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎛️</span>
<h3>Collapsible Filter Sidebar</h3>
<p>Toggle sort options, platform counts, install state, and collections instantly from a dedicated right sidebar.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🗂️</span>
<h3>Real Collections</h3>
<p>Read and write your actual Steam collections, with favourites and hidden state honoured.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🧩</span>
<h3>Multi-Scope Organize Lists</h3>
<p>Select multiple organization scopes simultaneously with Depressurizer-style custom rules to categorise your entire library.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🧠</span>
<h3>AutoCats</h3>
<p>Generate categories automatically from genre, developer, publisher, tags, and flags.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">💾</span>
<h3>Storage Statistics</h3>
<p>Dedicated drive capacity and usage gauges with Steam Library Folder management.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎮</span>
<h3>External Platforms</h3>
<p>Detect owned and installed games across GOG, Epic Games, itch.io, Xbox, and others, with shortcut launch integration.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🕹️</span>
<h3>Emulators &amp; ROMs</h3>
<p>Automatically discover ROMs across 11 popular emulators with pre-configured CLI launch arguments.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎨</span>
<h3>Artwork Manager</h3>
<p>Inspect, fetch, and set portrait grid, hero, logo, and icon artwork via SteamGridDB.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🌍</span>
<h3>30 Languages</h3>
<p>Hand-rolled reactive i18n supporting every official Steam language with Arabic RTL layout.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">⚡</span>
<h3>SQLite Cache</h3>
<p>High-performance embedded metadata cache with WAL mode and zero-touch migration.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🗄️</span>
<h3>Community Catalogue</h3>
<p>Precompiled offline database of curated multi-platform games with online sync and archive portability.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🏷️</span>
<h3>Console Tagging</h3>
<p>Determines console platforms for ROMs, renders UI badges, and writes binary shortcut tags for Steam grouping.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🛡️</span>
<h3>Safe writes</h3>
<p>Timestamped backups before every write, and changes are refused while Steam is running.</p>
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
Steam Vault combines multiple metadata sources. Only <strong>Steam Web API</strong>,
<strong>SteamGridDB</strong>, and <strong>IGDB</strong> require an API key; the rest are keyless.
</p>
<table>
<thead>
<tr><th>Provider</th><th>Key</th><th>Provides</th></tr>
</thead>
<tbody>
<tr><td>Community Catalogue</td><td>No</td><td>Curated offline seed with multi-platform OS and console metadata</td></tr>
<tr><td>Steam Store</td><td>No</td><td>Genres, developers, publishers, release dates, flags</td></tr>
<tr><td>Steam Store Search</td><td>No</td><td>Keyless title-to-appid matching for shortcuts &amp; indies</td></tr>
<tr><td>Steam Web API</td><td>Yes</td><td>Owned games, playtime, player summaries</td></tr>
<tr><td>SteamGridDB</td><td>Yes</td><td>Grids, heroes, logos, icons</td></tr>
<tr><td>IGDB (Twitch)</td><td>Yes</td><td>Summary, genres, companies, cover</td></tr>
<tr><td>VNDB</td><td>No</td><td>Visual novel metadata, developers, release dates, tags</td></tr>
<tr><td>PCGamingWiki</td><td>No</td><td>Engine, developers, publishers, release dates</td></tr>
<tr><td>Local Sniffer</td><td>No</td><td>itch manifests, Ren'Py, RPG Maker, PE VersionInfo headers</td></tr>
<tr><td>Wikipedia</td><td>No</td><td>Infobox parser: developers, publishers, genres, release year, console platforms, and summaries</td></tr>
</tbody>
</table>
<p>
See <a href="{{ '/docs/Metadata-Providers/' | relative_url }}">Metadata Providers</a> for the full
reference. RAWG is intentionally not supported.
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
<h2>Get Steam Vault</h2>
</div>
</div>
<p>
Steam Vault v1.0.0 is officially released! Build it from source today with Rust and Node.js.
See <a href="{{ '/docs/Getting-Started/' | relative_url }}">Getting Started</a> for the walkthrough.
</p>
<div class="header-actions">
<a class="button button--primary" href="{{ '/docs/Getting-Started/' | relative_url }}">Getting Started</a>
<a class="button button--secondary" href="https://github.com/HELIX-Origin/Steam-Vault">Source on GitHub ↗</a>
</div>
</section>
