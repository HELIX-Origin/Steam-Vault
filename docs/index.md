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
---

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
<span class="feature-card__icon" aria-hidden="true">🗂️</span>
<h3>Real collections</h3>
<p>Read and write your actual Steam collections, with favorites and hidden state honored.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🧩</span>
<h3>Manual organizing</h3>
<p>Multi-select games and add or remove them from collections in a single pass.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🧠</span>
<h3>AutoCats</h3>
<p>Generate categories automatically from genre, developer, publisher, tags, and flags.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎛️</span>
<h3>Advanced filters</h3>
<p>Build Allowed (OR), Required (AND), and Excluded (NOT) rules over your whole library.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">👾</span>
<h3>Non-Steam games</h3>
<p>Import and manage non-Steam shortcuts, with artwork pulled from SteamGridDB.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🎮</span>
<h3>External platforms</h3>
<p>Detect games installed by GOG, Epic Games, itch.io, and others, then import them as shortcuts.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🌐</span>
<h3>Whole owned library</h3>
<p>Sync every game you own through the Steam Web API, not just the ones installed right now.</p>
</div>
<div class="feature-card">
<span class="feature-card__icon" aria-hidden="true">🧱</span>
<h3>Collections from any key</h3>
<p>Build one collection per genre, developer, platform, playtime band, or name — preview first.</p>
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
Steam Vault combines several metadata sources. Only <strong>Steam Web API</strong>,
<strong>SteamGridDB</strong>, and <strong>IGDB</strong> require an API key; the rest are keyless.
</p>
<table>
<thead>
<tr><th>Provider</th><th>Key</th><th>Provides</th></tr>
</thead>
<tbody>
<tr><td>Steam Store</td><td>No</td><td>Genres, developers, publishers, release dates, flags</td></tr>
<tr><td>Steam Web API</td><td>Yes</td><td>Owned games, playtime, player summaries</td></tr>
<tr><td>SteamGridDB</td><td>Yes</td><td>Grids, heroes, logos, icons</td></tr>
<tr><td>IGDB (Twitch)</td><td>Yes</td><td>Summary, genres, companies, cover</td></tr>
<tr><td>PCGamingWiki</td><td>No</td><td>Engine, developers, publishers, release dates</td></tr>
<tr><td>Wikipedia</td><td>No</td><td>Article summaries</td></tr>
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
Steam Vault is in active development (Milestone&nbsp;M2). Build it from source today; packaged
installers are planned. See
<a href="{{ '/docs/Getting-Started/' | relative_url }}">Getting Started</a> for the walkthrough.
</p>
<div class="header-actions">
<a class="button button--primary" href="{{ '/docs/Getting-Started/' | relative_url }}">Getting Started</a>
<a class="button button--secondary" href="https://github.com/HELIX-Origin/Steam-Vault">Source on GitHub ↗</a>
</div>
</section>
