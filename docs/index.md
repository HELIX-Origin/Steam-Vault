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
top titlebar navigation, a dedicated left game list sidebar in the Library with live filtering and platform accordions, and an immersive showcase
view featuring hero artwork, media screenshot galleries, and one-click launch controls.
</p>
<p>
Built with <strong>Tauri v2</strong> (Rust backend and SvelteKit frontend), Universal Game Launcher operates completely non-invasively.
It never alters external client files, never harvests credentials, and persists all library overrides, custom tags, and artwork mappings locally
in a high-performance SQLite database (<code>database.sqlite</code>) backed by an in-memory serverless KV tier for sub-millisecond responsiveness.
</p>
</section>

<section id="features" class="doc-section">
<div class="section-heading">
<span class="section-number">02</span>
<div>
<p class="eyebrow">Core Engine</p>
<h2>Primary Application Features</h2>
</div>
</div>
<p>
The core launcher provides the native user interface, window ergonomics, high-performance data persistence, and protocol execution:
</p>
<div class="features-grid">
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🧭</span>
<h3>Frameless Title Bar Navigation</h3>
<p>Clean top-level navigation seamlessly integrated into the custom titlebar: Dashboard, Library, Store, Social, Storage, Plugins, and Settings.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎮</span>
<h3>Steam &amp; GOG Left Games List Sidebar</h3>
<p>Dedicated to the Library view, featuring instant name search, quick filter pills (All, Installed, ★ Favorites), background platform scan trigger (<code>↻ Scan</code>), and collapsible platform accordions.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🖼️</span>
<h3>Dedicated Game Showcase View</h3>
<p>Rich showcase for selected games with 16:9 hero wallpaper banners, title typography, prominent PLAY NOW launch buttons, playtime metrics, and descriptions.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">📸</span>
<h3>10-Image Screenshot Media Gallery</h3>
<p>Interactive media carousel featuring up to 10 high-resolution screenshots with a fullscreen lightbox viewer.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">⚡</span>
<h3>Zero-Lag Obsidian Purple Theme</h3>
<p>High-contrast dark purple visual identity (<code>#080511</code> / <code>#140d28</code> / <code>#a855f7</code>) with hardware-accelerated rendering and zero <code>backdrop-filter: blur</code> performance overhead.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🔒</span>
<h3>Native Launcher Delegation</h3>
<p>Launches, updates, and uninstalls delegate directly to official client protocols or native binaries, preserving overlays, cloud saves, friend lists, and achievements without storing credentials.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🗄️</span>
<h3>Generic SQLite &amp; In-Memory KV Persistence</h3>
<p>All collections, custom tags, favorite flags, and artwork assignments persist locally in <code>database.sqlite</code> backed by an in-memory Redis KV tier (<code>mem_kv</code>) for sub-millisecond responsiveness.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎨</span>
<h3>Custom Artwork Studio</h3>
<p>Inspect, fetch, and set portrait capsule covers, hero banners, transparent logos, and square icons via community providers and local image imports.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">📁</span>
<h3>Custom &amp; Dynamic Smart Collections</h3>
<p>Create custom categories, tag favorites, hide titles, and build dynamic smart collections evaluated instantaneously against your library.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🧩</span>
<h3>Native Modular Plugin Engine</h3>
<p>The core runtime loads and orchestrates native modular plugins (<code>src/lib/plugins/</code>) with full category isolation, typed schemas, and connection verification.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">⚙️</span>
<h3>Tabbed Plugin Settings Dialog</h3>
<p>Dedicated sub-window dialog for configuring each enabled plugin, featuring live password visibility eye toggles, direct help links, and real-time in-memory connection tests.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🛡️</span>
<h3>100% Non-Invasive Safety</h3>
<p>Never writes to Steam files (no <code>shortcuts.vdf</code> tampering), never requires official launchers to be closed, and stores zero external credentials.</p>
</div>
</div>
</section>

<section id="plugins" class="doc-section">
<div class="section-heading">
<span class="section-number">03</span>
<div>
<p class="eyebrow">Modularity</p>
<h2>Modular Plugin Ecosystem</h2>
</div>
</div>
<p>
External features are decoupled into native modular plugins built directly into the application under <code>src/lib/plugins/</code>. This eliminates Cloudflare network blocks, runtime script evaluation, and fragile external JSON manifests:
</p>
<div class="features-grid">
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🕹️</span>
<h3>12 Platform Storefront Integrations</h3>
<p>Native discovery, launch delegation, social friend rosters, and achievement hooks for Steam, Epic Games (official or Legendary CLI), GOG Galaxy, Xbox, itch.io, EA App, Ubisoft Connect, Battle.net, Amazon Games, Rockstar Games, Standalone executables, and EmulationStation.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">📚</span>
<h3>11 Rich Metadata &amp; Artwork Providers</h3>
<p>Authoritative native storefront details, community artwork, and game telemetry via Steam Store, GOG Catalog, Epic Games Store, itch.io, SteamGridDB, IGDB, TheGamesDB, ScreenScraper, HowLongToBeat, PCGamingWiki, and ProtonDB.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎁</span>
<h3>GamerPower Universal Feeds &amp; Alerts</h3>
<p>Plugin-powered notifications for 100% free store giveaways across major storefronts, weekly sales &amp; bargains, and developer patch notes for installed games.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">💾</span>
<h3>Storage &amp; Drive Footprint Analyzer</h3>
<p>Utility plugin performing non-blocking drive capacity analysis and platform game storage footprint calculations using background thread workers.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎞️</span>
<h3>Trailers &amp; Ambient Media</h3>
<p>Media plugins providing official streaming video trailer previews and ambient game soundtrack playback in the game showcase view.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🕹️</span>
<h3>EmulationStation ROM Normalization</h3>
<p>Organize ROM libraries conforming to the ES-DE directory standard with automated folder scaffolding and intelligent title normalization stripping dump tags.</p>
</div>
</div>
<p>
Explore the complete <a href="{{ '/docs/Available-Plugins/' | relative_url }}">Available Plugins Reference</a> for exhaustive details on every plugin.
</p>
</section>

<section id="providers" class="doc-section">
<div class="section-heading">
<span class="section-number">04</span>
<div>
<p class="eyebrow">Architecture</p>
<h2>Data Providers &amp; Metadata Hierarchy</h2>
</div>
</div>
<p>
Universal Game Launcher enforces an authoritative role hierarchy between game platforms and metadata providers:
</p>
<ul>
<li><strong>Native Game Providers (Authoritative Default):</strong> When games are scanned or loaded, the official game provider plugin (Steam, Epic, GOG, itch, EA, EmulationStation, Standalone) fetches its own authoritative game details and artwork directly from its own source.</li>
<li><strong>Metadata Providers (Manual On-Demand Enhancement):</strong> Third-party metadata providers (TheGamesDB, ScreenScraper, IGDB, SteamGridDB, HowLongToBeat, PCGamingWiki, ProtonDB) are run on-demand by the user via the <strong>✨ Enhance Metadata</strong> action in the showcase view and the Manage Game dialog to enrich details and fill missing fields.</li>
<li><strong>Cache Reset Invariant:</strong> Game providers are <em>strictly prohibited</em> from overwriting metadata contributed by metadata providers unless the user explicitly chooses to reset the cache via <strong>↺ Reset Cache to Provider Defaults</strong>. Enhanced records are flagged with <code>userEnhanced: true</code> and protected during normal background syncs.</li>
</ul>

<table>
<thead>
<tr><th>Provider Plugin</th><th>Category</th><th>Key Required</th><th>Primary Information &amp; Assets Provided</th></tr>
</thead>
<tbody>
<tr><td><strong>Steam Store</strong> (<code>metadata-steam</code>)</td><td>Metadata</td><td>No</td><td>Genres, developers, publishers, release dates, reviews, 1080p screenshots</td></tr>
<tr><td><strong>GOG Catalog</strong> (<code>metadata-gog</code>)</td><td>Metadata</td><td>No</td><td>Official vertical covers, hero wallpapers, summaries, DRM-free releases</td></tr>
<tr><td><strong>Epic Games Store</strong> (<code>metadata-epic</code>)</td><td>Metadata</td><td>No</td><td>Catalog details, promotional keyart covers, release dates</td></tr>
<tr><td><strong>itch.io Store</strong> (<code>metadata-itch</code>)</td><td>Metadata</td><td>No</td><td>Indie titles, developer profiles, screenshots, library sync</td></tr>
<tr><td><strong>SteamGridDB</strong> (<code>metadata-steamgriddb</code>)</td><td>Metadata</td><td>Yes (Optional)</td><td>Community capsule covers (600×900), hero banners, logos, square icons</td></tr>
<tr><td><strong>IGDB (Twitch)</strong> (<code>metadata-igdb</code>)</td><td>Metadata</td><td>Yes (Optional)</td><td>Global taxonomy, developer studios, franchise links, critic scores</td></tr>
<tr><td><strong>TheGamesDB</strong> (<code>metadata-thegamesdb</code>)</td><td>Metadata</td><td>No</td><td>Retro console metadata, authentic box art, fanart heroes, clear logos</td></tr>
<tr><td><strong>ScreenScraper</strong> (<code>metadata-screenscraper</code>)</td><td>Metadata</td><td>No</td><td>Multi-console ROM metadata, 2D/3D box art, wheel logos, video snaps</td></tr>
<tr><td><strong>HowLongToBeat</strong> (<code>metadata-howlongtobeat</code>)</td><td>Metadata</td><td>No</td><td>Main Story, Main + Extra, and Completionist playtime averages</td></tr>
<tr><td><strong>PCGamingWiki</strong> (<code>metadata-pcgamingwiki</code>)</td><td>Metadata</td><td>No</td><td>Engine info, FOV/ultrawide fixes, controller flags, save locations</td></tr>
<tr><td><strong>ProtonDB</strong> (<code>metadata-protondb</code>)</td><td>Metadata</td><td>No</td><td>Linux / Steam Deck compatibility tiers and recommended launch tweaks</td></tr>
</tbody>
</table>
<p>
See <a href="{{ '/docs/Metadata-Providers/' | relative_url }}">Metadata Providers Guide</a> and <a href="{{ '/docs/Available-Plugins/' | relative_url }}">Available Plugins</a> for the full reference.
</p>
</section>

<section id="docs" class="doc-section">
<div class="section-heading">
<span class="section-number">05</span>
<div>
<p class="eyebrow">Reference</p>
<h2>Documentation Directory</h2>
</div>
</div>
<p>
Explore the complete user guides and technical wiki articles on this site:
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
<span class="section-number">06</span>
<div>
<p class="eyebrow">Installation</p>
<h2>Get Universal Game Launcher</h2>
</div>
</div>
<p>
Universal Game Launcher is distributed as standalone desktop installers via GitHub Releases and can be built from source today with Rust and Node.js.
See <a href="{{ '/docs/Getting-Started/' | relative_url }}">Getting Started</a> for the complete walkthrough.
</p>
<div class="header-actions">
<a class="button button--primary" href="{{ '/docs/Getting-Started/' | relative_url }}">Getting Started</a>
<a class="button button--secondary" href="https://github.com/HELIX-Origin/Universal-Game-Launcher">Source on GitHub ↗</a>
</div>
</section>
