<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { error, settings } from "$lib/stores";
  import { getAllPlatformMetadata, getPlatformMetadata } from "$lib/plugins";
  import type { FeaturedStoreItem, FreeGamePromotion, Platform, StoreDeal } from "$lib/types";
  import "$styles/store.scss";

  interface FeaturedItem {
    id: string;
    title: string;
    platform: Platform;
    platformName: string;
    platformColor: string;
    platformIcon: string;
    storeUrl: string;
    mainImage: string;
    thumbnails: string[];
    description: string;
    tags: string[];
    originalPrice?: string;
    finalPrice: string;
    discountPercent?: number;
    isFree?: boolean;
    tagLabel?: string;
  }

  interface CatalogItem {
    id: string;
    title: string;
    platform: Platform;
    platformName: string;
    platformColor: string;
    platformIcon: string;
    storeUrl: string;
    imageUrl?: string | null;
    originalPrice?: string | null;
    finalPrice: string;
    discountPercent?: number;
    isFree?: boolean;
    drmFree?: boolean;
    tags: string[];
    ratingScore?: number;
    description?: string;
  }

  const allPlatformMeta = getAllPlatformMetadata();

  const FALLBACK_IMAGE_URL = 'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="600" height="350" viewBox="0 0 600 350"><rect width="600" height="350" fill="%231e293b"/><text x="50%" y="50%" dominant-baseline="middle" text-anchor="middle" fill="%2364748b" font-family="sans-serif" font-size="20">Image Unavailable</text></svg>';

  function handleImageError(event: Event) {
    const target = event.currentTarget as HTMLImageElement;
    if (target && target.src !== FALLBACK_IMAGE_URL) {
      target.src = FALLBACK_IMAGE_URL;
    }
  }

  // State
  let searchQuery = $state("");
  let selectedPlatform = $state<string>("all");
  let selectedPriceFilter = $state<"all" | "free" | "10" | "20">("all");
  let selectedDiscountFilter = $state<"all" | "50" | "75">("all");
  let drmFreeOnly = $state(false);
  let viewMode = $state<"grid" | "list">("grid");
  let sortBy = $state<"featured" | "price-asc" | "discount-desc" | "title">("featured");

  // Steam-style Hero Carousel State
  let currentHeroIndex = $state(0);
  let activePreviewThumb = $state(0);
  let isCarouselPaused = $state(false);

  let featuredItems = $state<FeaturedItem[]>([]);
  let catalogItems = $state<CatalogItem[]>([]);
  let loadingCatalog = $state(false);

  function isPlatformPluginEnabled(platformId: string): boolean {
    const pluginId = `platform-${platformId}`;
    if ($settings?.enabledPlugins && $settings.enabledPlugins[pluginId] !== undefined) {
      return $settings.enabledPlugins[pluginId];
    }
    return true;
  }

  async function loadStoreData() {
    loadingCatalog = true;
    try {
      const allPlatforms = Object.keys(getAllPlatformMetadata());
      const enabledPlatforms = allPlatforms.filter((p) => isPlatformPluginEnabled(p));

      // 1. Fetch live featured items, promos, and deals concurrently
      const [liveFeatured, promos, deals] = await Promise.all([
        api.getFeaturedStoreItems(enabledPlatforms).catch(() => [] as FeaturedStoreItem[]),
        api.getFreeGamePromotions().catch(() => [] as FreeGamePromotion[]),
        api.getStoreDeals().catch(() => [] as StoreDeal[])
      ]);

      // 2. Map live featured items to featured hero carousel
      featuredItems = liveFeatured
        .filter((f) => isPlatformPluginEnabled(f.platform))
        .map((f) => {
          const meta = getPlatformMetadata(f.platform);
          return {
            id: f.id,
            title: f.title,
            platform: f.platform,
            platformName: meta.label,
            platformColor: meta.color,
            platformIcon: meta.icon,
            storeUrl: f.storeUrl,
            mainImage: f.mainImage,
            thumbnails: f.thumbnails && f.thumbnails.length > 0 ? f.thumbnails : [f.mainImage],
            description: f.description,
            tags: f.tags,
            originalPrice: f.originalPrice ?? undefined,
            finalPrice: f.finalPrice,
            discountPercent: f.discountPercent ?? undefined,
            isFree: f.isFree,
            tagLabel: f.tagLabel ?? undefined
          };
        });

      // 3. Assemble dynamic catalog from live feeds (promos, deals, and featured items)
      const items: CatalogItem[] = [];
      const seenIds = new Set<string>();

      // Promos -> Catalog
      for (const p of promos) {
        if (!isPlatformPluginEnabled(p.platform)) continue;
        const info = getPlatformMetadata(p.platform);
        seenIds.add(p.id);
        items.push({
          id: p.id,
          title: p.title,
          platform: p.platform,
          platformName: info.label,
          platformColor: info.color,
          platformIcon: info.icon,
          storeUrl: p.url,
          imageUrl: p.imageUrl,
          originalPrice: p.worth ?? "$29.99",
          finalPrice: "FREE",
          discountPercent: 100,
          isFree: true,
          drmFree: p.platform === "gog",
          tags: ["Giveaway", "100% Free", info.label],
          ratingScore: 94,
          description: p.description ?? undefined,
        });
      }

      // Deals -> Catalog
      for (const d of deals) {
        if (!isPlatformPluginEnabled(d.platform)) continue;
        const info = getPlatformMetadata(d.platform);
        if (seenIds.has(d.id)) continue;
        seenIds.add(d.id);
        items.push({
          id: d.id,
          title: d.title,
          platform: d.platform,
          platformName: info.label,
          platformColor: info.color,
          platformIcon: info.icon,
          storeUrl: d.url,
          imageUrl: d.imageUrl,
          originalPrice: d.originalPrice,
          finalPrice: d.finalPrice,
          discountPercent: d.discountPercent,
          isFree: false,
          drmFree: d.platform === "gog",
          tags: ["Special Offer", `-${d.discountPercent}%`, info.label],
          ratingScore: 88,
        });
      }

      // Featured items -> Catalog (if not already present)
      for (const f of liveFeatured) {
        if (!isPlatformPluginEnabled(f.platform)) continue;
        if (seenIds.has(f.id) || items.some((i) => i.title.toLowerCase() === f.title.toLowerCase())) continue;
        seenIds.add(f.id);
        const info = getPlatformMetadata(f.platform);
        items.push({
          id: f.id,
          title: f.title,
          platform: f.platform,
          platformName: info.label,
          platformColor: info.color,
          platformIcon: info.icon,
          storeUrl: f.storeUrl,
          imageUrl: f.mainImage,
          originalPrice: f.originalPrice ?? undefined,
          finalPrice: f.finalPrice,
          discountPercent: f.discountPercent ?? undefined,
          isFree: f.isFree,
          drmFree: f.platform === "gog",
          tags: f.tags,
          description: f.description,
          ratingScore: 90
        });
      }

      catalogItems = items;

      // Populate WebView2 / browser memory cache in background so cycling and hovering is instantaneous
      preloadStoreImages();
    } catch (e) {
      console.warn("Could not load store data:", e);
    } finally {
      loadingCatalog = false;
    }
  }

  function preloadStoreImages() {
    if (typeof window === "undefined") return;
    const urls: string[] = [];
    for (const f of featuredItems) {
      if (f.mainImage) urls.push(f.mainImage);
      if (f.thumbnails) urls.push(...f.thumbnails);
    }
    for (const c of catalogItems.slice(0, 20)) {
      if (c.imageUrl) urls.push(c.imageUrl);
    }
    // Background async preloader
    for (const url of urls) {
      const img = new Image();
      img.src = url;
    }
  }

  // Automatic Carousel Progression (Timer)
  onMount(() => {
    loadStoreData();

    const interval = setInterval(() => {
      if (!isCarouselPaused && featuredItems.length > 0) {
        currentHeroIndex = (currentHeroIndex + 1) % featuredItems.length;
        activePreviewThumb = 0;
      }
    }, 6000);

    return () => clearInterval(interval);
  });

  async function handleCheckout(item: { storeUrl: string; platformName: string }) {
    if (!item.storeUrl) return;
    try {
      // SECURITY & PRIVACY INVARIANT:
      // Orders and checkouts are NEVER processed inside the app.
      // Every checkout is safely sent to the official storefront in the user's default browser.
      await api.openExternalUrl(item.storeUrl);
    } catch (e) {
      error.set(errorMessage(e));
    }
  }

  async function handleOpenStorefront(platformKey: string) {
    const info = getPlatformMetadata(platformKey);
    if (info?.defaultUrl) {
      try {
        await api.openExternalUrl(info.defaultUrl);
      } catch (e) {
        error.set(errorMessage(e));
      }
    }
  }

  // Filtered & Sorted Catalog
  let filteredCatalog = $derived.by<CatalogItem[]>(() => {
    let list = catalogItems.filter((item) => {
      if (selectedPlatform !== "all" && item.platform !== selectedPlatform) return false;
      if (selectedPriceFilter === "free" && !item.isFree) return false;
      if (selectedPriceFilter === "10") {
        const num = parseFloat(item.finalPrice.replace(/[^0-9.]/g, ""));
        if (!isNaN(num) && num > 10) return false;
      }
      if (selectedPriceFilter === "20") {
        const num = parseFloat(item.finalPrice.replace(/[^0-9.]/g, ""));
        if (!isNaN(num) && num > 20) return false;
      }
      if (selectedDiscountFilter === "50" && (item.discountPercent ?? 0) < 50) return false;
      if (selectedDiscountFilter === "75" && (item.discountPercent ?? 0) < 75) return false;
      if (drmFreeOnly && !item.drmFree) return false;

      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase().trim();
        const matchTitle = item.title.toLowerCase().includes(q);
        const matchTags = item.tags.some((t) => t.toLowerCase().includes(q));
        return matchTitle || matchTags;
      }
      return true;
    });

    if (sortBy === "discount-desc") {
      list.sort((a, b) => (b.discountPercent ?? 0) - (a.discountPercent ?? 0));
    } else if (sortBy === "price-asc") {
      list.sort((a, b) => {
        const pa = a.isFree ? 0 : parseFloat(a.finalPrice.replace(/[^0-9.]/g, "")) || 999;
        const pb = b.isFree ? 0 : parseFloat(b.finalPrice.replace(/[^0-9.]/g, "")) || 999;
        return pa - pb;
      });
    } else if (sortBy === "title") {
      list.sort((a, b) => a.title.localeCompare(b.title));
    }

    return list;
  });

  let currentHero = $derived(featuredItems[currentHeroIndex]);
  let currentHeroImage = $derived(
    currentHero?.thumbnails[activePreviewThumb] || currentHero?.mainImage || ""
  );
</script>

<div class="store-container">
  <!-- Privacy & Zero-Payment Security Notice -->
  <div class="store-security-banner">
    <div class="banner-content">
      <span class="banner-icon">🛡️</span>
      <div>
        <strong>Privacy-First Universal Storefront</strong>
        <p>
          Browse sales, new releases, and free giveaways across all enabled platforms. All purchases and checkout flows are handled directly in your default browser on the official storefront — <strong>zero payment processing or credit card data is ever collected in this app.</strong>
        </p>
      </div>
    </div>
    <div class="storefront-badges">
      {#each Object.entries(allPlatformMeta) as [key, info]}
        {#if isPlatformPluginEnabled(key) && info.defaultUrl}
          <button
            class="btn mini btn-ghost"
            style="font-size:0.75rem;padding:0.25rem 0.5rem;border-color:{info.color}35;color:{info.color}"
            onclick={() => handleOpenStorefront(key)}
            title="Browse official {info.label} homepage in browser"
          >
            {info.icon} {info.label} ↗
          </button>
        {/if}
      {/each}
    </div>
  </div>

  <!-- Steam-Style Featured & Recommended Hero Carousel -->
  {#if currentHero}
    <div
      class="store-hero-carousel"
      role="region"
      aria-label="Featured Games Carousel"
      onmouseenter={() => (isCarouselPaused = true)}
      onmouseleave={() => (isCarouselPaused = false)}
    >
      <!-- Main Large Preview Stage -->
      <div
        class="carousel-main-stage"
        role="button"
        tabindex="0"
        onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") handleCheckout(currentHero); }}
        onclick={() => handleCheckout(currentHero)}
      >
        <img
          src={currentHeroImage}
          alt={currentHero.title}
          class="main-backdrop"
          loading="eager"
          onerror={handleImageError}
        />
        <div class="carousel-overlay">
          <span class="pill pill-accent" style="width:fit-content;margin-bottom:0.4rem;font-size:0.72rem">
            {currentHero.tagLabel ?? "Featured & Recommended"}
          </span>
          <h2 class="carousel-game-title">{currentHero.title}</h2>
          <div class="carousel-meta-row">
            <span class="pill pill-dim" style="border-color:{currentHero.platformColor}60;color:{currentHero.platformColor}">
              {currentHero.platformIcon} {currentHero.platformName}
            </span>
            <div class="row" style="gap:0.35rem">
              {#each currentHero.tags.slice(0, 3) as tag}
                <span class="pill pill-dim" style="font-size:0.65rem">{tag}</span>
              {/each}
            </div>
          </div>
        </div>
      </div>

      <!-- Right Column: Description, Thumbnails, Pricing, and Checkout -->
      <div class="carousel-sidebar">
        <div>
          <h3 style="margin:0 0 0.5rem;font-size:1.1rem;color:#fff">{currentHero.title}</h3>
          <p class="sidebar-desc">{currentHero.description}</p>

          <span class="muted-xs" style="display:block;margin-bottom:0.4rem;text-transform:uppercase;letter-spacing:0.04em">
            Screenshot Preview (Hover to view)
          </span>
          <div class="sidebar-thumbnails-grid">
            {#each currentHero.thumbnails as thumb, idx}
              <button
                type="button"
                class="thumb-preview-box"
                class:active={activePreviewThumb === idx}
                onmouseenter={() => (activePreviewThumb = idx)}
                onclick={() => (activePreviewThumb = idx)}
                title="Preview screenshot {idx + 1}"
              >
                <img src={thumb} alt="Preview thumbnail {idx + 1}" onerror={handleImageError} />
              </button>
            {/each}
          </div>

          <div class="sidebar-tags">
            {#each currentHero.tags as tag}
              <span class="pill pill-dim" style="font-size:0.62rem">{tag}</span>
            {/each}
          </div>
        </div>

        <div class="sidebar-checkout-bar">
          <div class="price-cluster">
            {#if currentHero.discountPercent && currentHero.discountPercent > 0}
              <span class="pill pill-accent" style="font-size:0.75rem;font-weight:bold">
                -{currentHero.discountPercent}%
              </span>
            {/if}
            <div>
              {#if currentHero.originalPrice && currentHero.originalPrice !== currentHero.finalPrice}
                <span class="orig-price">{currentHero.originalPrice}</span>
              {/if}
              <span class="final-price">{currentHero.finalPrice}</span>
            </div>
          </div>

          <button
            class="btn btn-primary"
            onclick={() => handleCheckout(currentHero)}
            title="Buy or view {currentHero.title} on {currentHero.platformName} in default browser"
          >
            {currentHero.isFree ? "Claim Now ↗" : `Buy on ${currentHero.platformName} ↗`}
          </button>
        </div>
      </div>

      <!-- Carousel Navigation Dots -->
      <div class="carousel-nav-dots">
        {#each featuredItems as _, idx}
          <button
            type="button"
            class="dot"
            class:active={currentHeroIndex === idx}
            onclick={() => { currentHeroIndex = idx; activePreviewThumb = 0; }}
            title="Slide {idx + 1}"
          ></button>
        {/each}
      </div>
    </div>
  {/if}

  <!-- GOG / Steam-Style Special Weekend Deals & Giveaways Ribbon -->
  {#if catalogItems.filter((i) => i.isFree || (i.discountPercent && i.discountPercent >= 50)).length > 0}
    <section class="deals-ribbon-section">
      <div class="section-head">
        <h2><span>⚡</span> Special Offers &amp; 100% Free Giveaways</h2>
        <span class="muted-sm">Limited time discounts verified across official storefronts</span>
      </div>

      <div class="deals-cards-scroll">
        {#each catalogItems.filter((i) => i.isFree || (i.discountPercent && i.discountPercent >= 50)).slice(0, 4) as deal (deal.id)}
          <div
            class="deal-card"
            role="button"
            tabindex="0"
            onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") handleCheckout(deal); }}
            onclick={() => handleCheckout(deal)}
            style="cursor:pointer"
          >
            <div class="card-media">
              {#if deal.imageUrl}
                <img src={deal.imageUrl} alt={deal.title} loading="lazy" onerror={handleImageError} />
              {:else}
                <div style="width:100%;height:100%;display:flex;align-items:center;justify-content:center;font-size:2.5rem">
                  {deal.platformIcon}
                </div>
              {/if}

              {#if deal.discountPercent && deal.discountPercent > 0}
                <span class="discount-badge">
                  {deal.isFree ? "100% FREE" : `-${deal.discountPercent}%`}
                </span>
              {/if}

              <span class="provider-tag" style="border-left:3px solid {deal.platformColor};color:#fff">
                {deal.platformIcon} {deal.platformName}
              </span>
            </div>

            <div class="card-info">
              <span class="card-title">{deal.title}</span>
              <div class="card-price-row">
                <div class="row" style="align-items:center;gap:0.4rem">
                  {#if deal.originalPrice && deal.originalPrice !== deal.finalPrice}
                    <span class="muted-xs" style="text-decoration:line-through">{deal.originalPrice}</span>
                  {/if}
                  <strong style="color:{deal.isFree ? 'var(--accent)' : '#fff'};font-size:0.95rem">
                    {deal.finalPrice}
                  </strong>
                </div>
                <button
                  class="btn mini btn-ghost"
                  onclick={(e) => { e.stopPropagation(); handleCheckout(deal); }}
                  title="Open {deal.platformName} in browser"
                >
                  ↗
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </section>
  {/if}

  <!-- Advanced Storefront Catalog Section with Filtering Sidebar -->
  <div class="catalog-main-layout">
    <!-- Left Sidebar: Multi-Dimensional Filter Controls (Steam & GOG Style) -->
    <aside class="catalog-filters-sidebar">
      <!-- Search Input -->
      <div class="filter-group">
        <h4 class="filter-group-title">Search Catalog</h4>
        <input
          type="text"
          placeholder="Filter by title or tag…"
          bind:value={searchQuery}
          style="width:100%;font-size:0.82rem;padding:0.4rem 0.6rem"
        />
      </div>

      <!-- Storefronts -->
      <div class="filter-group">
        <h4 class="filter-group-title">Storefront</h4>
        <label class="filter-option-row">
          <span>All Storefronts</span>
          <input
            type="radio"
            name="platform-filter"
            checked={selectedPlatform === "all"}
            onchange={() => (selectedPlatform = "all")}
          />
        </label>
        {#each Object.entries(allPlatformMeta) as [key, info]}
          {#if isPlatformPluginEnabled(key)}
            <label class="filter-option-row">
              <span style="color:{info.color}">{info.icon} {info.label}</span>
              <input
                type="radio"
                name="platform-filter"
                checked={selectedPlatform === key}
                onchange={() => (selectedPlatform = key)}
              />
            </label>
          {/if}
        {/each}
      </div>

      <!-- Price Range -->
      <div class="filter-group">
        <h4 class="filter-group-title">Price Filter</h4>
        <label class="filter-option-row">
          <span>Any Price</span>
          <input
            type="radio"
            name="price-filter"
            checked={selectedPriceFilter === "all"}
            onchange={() => (selectedPriceFilter = "all")}
          />
        </label>
        <label class="filter-option-row">
          <span>Free Only</span>
          <input
            type="radio"
            name="price-filter"
            checked={selectedPriceFilter === "free"}
            onchange={() => (selectedPriceFilter = "free")}
          />
        </label>
        <label class="filter-option-row">
          <span>Under $10.00</span>
          <input
            type="radio"
            name="price-filter"
            checked={selectedPriceFilter === "10"}
            onchange={() => (selectedPriceFilter = "10")}
          />
        </label>
        <label class="filter-option-row">
          <span>Under $20.00</span>
          <input
            type="radio"
            name="price-filter"
            checked={selectedPriceFilter === "20"}
            onchange={() => (selectedPriceFilter = "20")}
          />
        </label>
      </div>

      <!-- Discount Depth -->
      <div class="filter-group">
        <h4 class="filter-group-title">Discount Depth</h4>
        <label class="filter-option-row">
          <span>Any Discount</span>
          <input
            type="radio"
            name="discount-filter"
            checked={selectedDiscountFilter === "all"}
            onchange={() => (selectedDiscountFilter = "all")}
          />
        </label>
        <label class="filter-option-row">
          <span>50% Off or More</span>
          <input
            type="radio"
            name="discount-filter"
            checked={selectedDiscountFilter === "50"}
            onchange={() => (selectedDiscountFilter = "50")}
          />
        </label>
        <label class="filter-option-row">
          <span>75% Off or More</span>
          <input
            type="radio"
            name="discount-filter"
            checked={selectedDiscountFilter === "75"}
            onchange={() => (selectedDiscountFilter = "75")}
          />
        </label>
      </div>

      <!-- Features & Protections -->
      <div class="filter-group">
        <h4 class="filter-group-title">Features</h4>
        <label class="filter-option-row">
          <span>DRM-Free Only (GOG/itch)</span>
          <input type="checkbox" bind:checked={drmFreeOnly} />
        </label>
      </div>
    </aside>

    <!-- Right Items Area: Toolbar + Grid / List View -->
    <main class="catalog-items-area">
      <!-- Catalog Toolbar -->
      <div class="catalog-toolbar">
        <span class="muted-sm">
          Showing <strong>{filteredCatalog.length}</strong> games &amp; offers
        </span>

        <div class="row" style="align-items:center;gap:0.75rem">
          <!-- Sort Selector -->
          <div class="row" style="align-items:center;gap:0.4rem">
            <span class="muted-xs">Sort:</span>
            <select
              bind:value={sortBy}
              style="font-size:0.8rem;padding:0.25rem 0.5rem;background:rgba(15,23,42,0.9);color:#fff;border:1px solid rgba(255,255,255,0.1);border-radius:4px"
            >
              <option value="featured">Featured Deals</option>
              <option value="discount-desc">Highest Discount</option>
              <option value="price-asc">Lowest Price</option>
              <option value="title">Alphabetical (A-Z)</option>
            </select>
          </div>

          <!-- View Mode Toggle -->
          <div class="row" style="gap:0.2rem">
            <button
              class="btn mini"
              class:btn-accent={viewMode === "grid"}
              class:btn-ghost={viewMode !== "grid"}
              onclick={() => (viewMode = "grid")}
              title="Grid view"
            >
              ▦
            </button>
            <button
              class="btn mini"
              class:btn-accent={viewMode === "list"}
              class:btn-ghost={viewMode !== "list"}
              onclick={() => (viewMode = "list")}
              title="List view"
            >
              ☰
            </button>
          </div>
        </div>
      </div>

      <!-- Items Grid View -->
      {#if viewMode === "grid"}
        <div class="catalog-grid">
          {#each filteredCatalog as item (item.id)}
            <div
              class="deal-card"
              role="button"
              tabindex="0"
              onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") handleCheckout(item); }}
              onclick={() => handleCheckout(item)}
              style="cursor:pointer"
            >
              <div class="card-media">
                {#if item.imageUrl}
                  <img src={item.imageUrl} alt={item.title} loading="lazy" onerror={handleImageError} />
                {:else}
                  <div style="width:100%;height:100%;display:flex;align-items:center;justify-content:center;font-size:2.5rem">
                    {item.platformIcon}
                  </div>
                {/if}

                {#if item.discountPercent && item.discountPercent > 0}
                  <span class="discount-badge">
                    {item.isFree ? "FREE" : `-${item.discountPercent}%`}
                  </span>
                {/if}

                <span class="provider-tag" style="border-left:3px solid {item.platformColor};color:#fff">
                  {item.platformIcon} {item.platformName}
                </span>
              </div>

              <div class="card-info">
                <span class="card-title">{item.title}</span>

                <div class="row" style="gap:0.25rem;flex-wrap:wrap">
                  {#each item.tags.slice(0, 2) as tag}
                    <span class="pill pill-dim" style="font-size:0.6rem">{tag}</span>
                  {/each}
                </div>

                <div class="card-price-row">
                  <div>
                    {#if item.originalPrice && item.originalPrice !== item.finalPrice}
                      <span class="muted-xs" style="text-decoration:line-through;display:block;font-size:0.75rem">
                        {item.originalPrice}
                      </span>
                    {/if}
                    <strong style="color:{item.isFree ? 'var(--accent)' : '#fff'};font-size:1rem">
                      {item.finalPrice}
                    </strong>
                  </div>

                  <button
                    class="btn mini btn-primary"
                    onclick={(e) => { e.stopPropagation(); handleCheckout(item); }}
                    title="Open checkout securely on {item.platformName} in default browser"
                  >
                    {item.isFree ? "Claim ↗" : "Buy ↗"}
                  </button>
                </div>
              </div>
            </div>
          {/each}
        </div>

      <!-- Items List / Table View -->
      {:else}
        <div style="display:flex;flex-direction:column;gap:0.4rem">
          {#each filteredCatalog as item (item.id)}
            <div
              class="glass card"
              role="button"
              tabindex="0"
              onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") handleCheckout(item); }}
              onclick={() => handleCheckout(item)}
              style="padding:0.65rem 1rem;display:flex;align-items:center;justify-content:space-between;gap:1rem;cursor:pointer;border-radius:6px;transition:background 0.15s"
            >
              <div class="row" style="align-items:center;gap:0.85rem;flex:1;min-width:0">
                <div style="width:55px;height:35px;border-radius:4px;overflow:hidden;background:#0f172a;flex-shrink:0">
                  {#if item.imageUrl}
                    <img src={item.imageUrl} alt={item.title} style="width:100%;height:100%;object-fit:cover" onerror={handleImageError} />
                  {/if}
                </div>
                <div style="min-width:0;flex:1">
                  <strong style="font-size:0.92rem;color:#fff;display:block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">
                    {item.title}
                  </strong>
                  <div class="row" style="gap:0.35rem;align-items:center;margin-top:0.15rem">
                    <span class="muted-xs" style="color:{item.platformColor}">
                      {item.platformIcon} {item.platformName}
                    </span>
                    {#each item.tags.slice(0, 3) as tag}
                      <span class="pill pill-dim" style="font-size:0.58rem">{tag}</span>
                    {/each}
                  </div>
                </div>
              </div>

              <div class="row" style="align-items:center;gap:0.75rem">
                {#if item.discountPercent && item.discountPercent > 0}
                  <span class="pill pill-accent" style="font-size:0.68rem;font-weight:bold">
                    -{item.discountPercent}%
                  </span>
                {/if}
                <div style="text-align:right;min-width:65px">
                  {#if item.originalPrice && item.originalPrice !== item.finalPrice}
                    <span class="muted-xs" style="text-decoration:line-through;font-size:0.72rem;display:block">
                      {item.originalPrice}
                    </span>
                  {/if}
                  <strong style="color:{item.isFree ? 'var(--accent)' : '#fff'};font-size:0.92rem">
                    {item.finalPrice}
                  </strong>
                </div>

                <button
                  class="btn mini btn-primary"
                  onclick={(e) => { e.stopPropagation(); handleCheckout(item); }}
                  title="Checkout on {item.platformName}"
                >
                  ↗
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}

      {#if filteredCatalog.length === 0}
        <div class="glass card" style="padding:3rem;text-align:center">
          <p class="muted" style="margin:0 0 0.5rem;font-size:1rem">No items found matching the selected filters.</p>
          <button class="btn mini btn-ghost" onclick={() => { searchQuery = ""; selectedPlatform = "all"; selectedPriceFilter = "all"; selectedDiscountFilter = "all"; drmFreeOnly = false; }}>
            Reset Filters
          </button>
        </div>
      {/if}
    </main>
  </div>
</div>
