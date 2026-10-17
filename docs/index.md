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
<p>Clean top-level navigation seamlessly integrated into the custom titlebar: Dashboard, Library, Store, Social, Storage, and Settings.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎮</span>
<h3>Steam &amp; GOG Left Games List Sidebar</h3>
<p>Dedicated strictly to the Library view, featuring instant name search, quick filter pills (All, Installed, ★ Favorites), background platform scan trigger (<code>↻ Scan</code>), and collapsible platform and console accordions.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎯</span>
<h3>Authentic Vector SVG Platform &amp; Console Icons</h3>
<p>Replaces generic Unicode shapes with authentic vector SVG variants for all PC game providers (Steam, Epic, GOG, Xbox, EA, Ubisoft, Battle.net, itch.io, Amazon, Rockstar, Standalone) and retro emulation consoles.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🖼️</span>
<h3>Dedicated Game Showcase View</h3>
<p>Rich showcase for selected games featuring 16:9 hero wallpaper banners, title typography, prominent PLAY NOW launch buttons, playtime metrics, and descriptions.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎬</span>
<h3>Consolidated Unified Media Showcase</h3>
<p>Streamlined media section with dynamic tabs: <strong>🖼️ Screenshots &amp; Art</strong> (with fullscreen lightbox), <strong>🎬 Trailers</strong> (streaming modern Steam Akamai MP4 videos), and <strong>🎵 Soundtracks</strong> (ambient theme playback).</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">⚙️</span>
<h3>Per-Game Customization Modal</h3>
<p>The <code>⚙ Manage</code> modal in Library allows editing game Title, Short Description, Developers, Publishers, Release Date, Genres, and Custom Artwork (Capsule, Hero, Logo via URL or local file picker).</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🤖</span>
<h3>Automatic Multi-Source Metadata Engine</h3>
<p>Automatically enriches games in the background across Local Inspector, Libretro Thumbnails CDN, Steam Store, GOG Catalog v1, Wikipedia/Wikidata MediaWiki API, PCGamingWiki Cargo API, and ProtonDB with zero placeholder strings.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🗄️</span>
<h3>Generic SQLite &amp; In-Memory KV Persistence</h3>
<p>All collections, custom tags, favorite flags, and artwork assignments persist locally in <code>database.sqlite</code> backed by an in-memory Redis KV tier (<code>mem_kv</code>) for sub-millisecond responsiveness.</p>
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
<span class="feature-card__icon" aria-hidden="true">🧹</span>
<h3>Settings-Only Cache Refresh &amp; Purge</h3>
<p>Library views remain focused on playing and customization; cache refresh and purge actions live exclusively in <strong>Settings → Storage &amp; Cache</strong>.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">💻</span>
<h3>Headless CLI Foundation (<code>--cli</code>)</h3>
<p>Built-in command-line interface supporting <code>--cli scan</code>, <code>scan-roms</code>, <code>scaffold-roms</code>, <code>normalize-title</code>, and <code>launchers</code> for headless automation and continuous quality assurance.</p>
</div>
</div>
</section>

<section id="plugins" class="doc-section">
<div class="section-heading">
<span class="section-number">03</span>
<div>
<p class="eyebrow">Modularity</p>
<h2>Universal Core Plugins Ecosystem</h2>
</div>
</div>
<p>
External features are organized into unified <strong>Universal Core Plugins</strong> paired with global helper modules under <code>src/lib/plugins/</code>, eliminating Cloudflare blocks and fragile external JSON manifests:
</p>
<div class="features-grid">
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🕹️</span>
<h3>Universal Platform Engine (<code>platform-universal</code>)</h3>
<p>Consolidated engine managing discovery, launch delegation, social rosters, and achievement hooks across all 12 platforms (Steam, Epic Games, GOG Galaxy, Xbox, itch.io, EA App, Ubisoft Connect, Battle.net, Amazon Games, Rockstar Games, Standalone, and EmulationStation) paired with global platform helpers.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">📚</span>
<h3>Universal Metadata Engine (<code>metadata-universal</code>)</h3>
<p>Automatic multi-source metadata resolution cascading across Local Inspector, Libretro CDN, Steam Store, GOG Catalog v1, Wikipedia + Wikidata MediaWiki API, PCGamingWiki Cargo API, ProtonDB, and optional authenticated APIs (SteamGridDB, IGDB).</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🔤</span>
<h3>Universal Title Normalizer (<code>utility-title-normalizer</code>)</h3>
<p>Dedicated title normalizer handling RPG Maker <code>System.json</code> parsing, itch.io slug separation, ROM dump tags, article inversion (<code>The Legend of Zelda</code>), and unified subtitle formatting.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎬</span>
<h3>Universal Media Engine (<code>media-universal</code>)</h3>
<p>Provides modern Steam CDN MP4 streaming video trailers and ambient soundtrack playback inside the consolidated media showcase.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎁</span>
<h3>Universal Feeds &amp; Alerts (<code>utility-deals</code> &amp; <code>utility-patchnotes</code>)</h3>
<p>Live alerts for 100% free store giveaways across major PC storefronts, weekly sales &amp; bargains, and developer patch notes for installed titles.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">💾</span>
<h3>Storage &amp; Drive Footprint Analyzer (<code>utility-storage</code>)</h3>
<p>Utility plugin performing non-blocking drive capacity analysis and platform game storage footprint calculations using background thread workers.</p>
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
<li><strong>Automatic Multi-Source Resolution:</strong> When games are imported, the Universal Metadata Engine cascades across Local Inspector, Libretro Thumbnails CDN, Steam Store, GOG Catalog v1, Wikipedia/Wikidata MediaWiki API, and PCGamingWiki Cargo API to automatically populate rich descriptions, release dates, and authentic artwork without requiring manual intervention.</li>
<li><strong>Modern Video Trailers &amp; Media:</strong> Steam Store metadata queries extract modern Akamai CDN MP4 video trailer streams (<code>https://cdn.akamai.steamstatic.com/steam/apps/{movie_id}/movie_max.mp4</code>) for playback in the Trailers tab.</li>
<li><strong>Per-Game Customization:</strong> Users can override any field or artwork slot directly via the <strong>⚙ Manage</strong> modal in the Library showcase view, persisting overrides locally in <code>database.sqlite</code>.</li>
<li><strong>Cache Reset Invariant:</strong> Background scans never overwrite user-enhanced or manually customized metadata. Cache refresh and cache purge actions are cleanly isolated in <strong>Settings → Storage &amp; Cache</strong>.</li>
</ul>

<table>
<thead>
<tr><th>Provider Source</th><th>Category</th><th>Key Required</th><th>Primary Information &amp; Assets Provided</th></tr>
</thead>
<tbody>
<tr><td><strong>Local Inspector</strong></td><td>Local Manifest</td><td>No</td><td>RPG Maker <code>System.json</code> titles, itch.io receipt manifests, <code>rom.ini</code> overrides, binary metadata</td></tr>
<tr><td><strong>Libretro CDN</strong></td><td>Metadata &amp; Art</td><td>No</td><td>Authentic console box art (Named_Boxarts), title screens (Named_Titles), snaps, clear logos</td></tr>
<tr><td><strong>Steam Store</strong></td><td>Metadata &amp; Media</td><td>No</td><td>Genres, developers, publishers, release dates, reviews, 1080p screenshots, modern CDN MP4 trailers</td></tr>
<tr><td><strong>GOG Catalog v1</strong></td><td>Metadata</td><td>No</td><td>Official vertical covers, hero wallpapers, summaries, DRM-free releases</td></tr>
<tr><td><strong>Wikipedia &amp; Wikidata</strong></td><td>Metadata</td><td>No</td><td>Deep synopses, developers (P178), publishers (P123), genres (P136), release dates (P577)</td></tr>
<tr><td><strong>PCGamingWiki Cargo</strong></td><td>Metadata</td><td>No</td><td>Game engine, architecture, save file locations, FOV/ultrawide fixes</td></tr>
<tr><td><strong>ProtonDB</strong></td><td>Metadata</td><td>No</td><td>Linux / Steam Deck compatibility ratings (Native, Platinum, Gold) and launch flags</td></tr>
<tr><td><strong>SteamGridDB</strong></td><td>Metadata &amp; Art</td><td>Yes (Optional)</td><td>Community capsule covers (600×900), hero banners, logos, square icons</td></tr>
<tr><td><strong>IGDB (Twitch)</strong></td><td>Metadata</td><td>Yes (Optional)</td><td>Global taxonomy, developer studios, franchise links, aggregated critic scores</td></tr>
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
