<script lang="ts">
  interface Props {
    platform?: string | null;
    consoleName?: string | null;
    size?: number | string;
    color?: string;
    class?: string;
  }

  let {
    platform = "standalone",
    consoleName = null,
    size = 15,
    color = "currentColor",
    class: className = "",
  }: Props = $props();

  let key = $derived.by(() => {
    const rawConsole = (consoleName || "").toLowerCase().trim();
    if (
      rawConsole.includes("gamecube") ||
      rawConsole.includes("nintendo") ||
      rawConsole.includes("switch") ||
      rawConsole.includes("wii") ||
      rawConsole.includes("snes") ||
      rawConsole.includes("nes") ||
      rawConsole.includes("game boy") ||
      rawConsole.includes("3ds") ||
      rawConsole.includes("nds")
    ) {
      return "nintendo";
    }
    if (rawConsole.includes("playstation") || rawConsole.includes("ps1") || rawConsole.includes("ps2") || rawConsole.includes("ps3") || rawConsole.includes("psp") || rawConsole.includes("vita")) {
      return "playstation";
    }
    if (rawConsole.includes("sega") || rawConsole.includes("dreamcast") || rawConsole.includes("genesis") || rawConsole.includes("saturn")) {
      return "sega";
    }

    const raw = (platform || "standalone").toLowerCase().trim();
    if (raw === "all" || raw === "library") return "all";
    if (raw === "favorite" || raw === "favorites" || raw === "★") return "favorite";
    if (raw.includes("steam") || raw === "♨") return "steam";
    if (raw.includes("gog") || raw === "◈") return "gog";
    if (raw.includes("epic") || raw === "▲") return "epic";
    if (raw.includes("itch") || raw === "⌗") return "itch";
    if (raw.includes("xbox") || raw === "⨂") return "xbox";
    if (raw === "ea" || raw.includes("ea app") || raw.includes("electronic arts") || raw === "●") return "ea";
    if (raw.includes("ubisoft") || raw.includes("uplay") || raw === "🌀") return "ubisoft";
    if (raw.includes("battle") || raw.includes("blizzard") || raw === "❄") return "battlenet";
    if (raw.includes("amazon") || raw === "📦") return "amazon";
    if (raw.includes("rockstar") || raw === "★") return "rockstar";
    if (
      raw.includes("gamecube") ||
      raw.includes("nintendo") ||
      raw.includes("switch") ||
      raw.includes("wii") ||
      raw.includes("snes") ||
      raw.includes("nes") ||
      raw.includes("game boy") ||
      raw.includes("3ds") ||
      raw.includes("nds")
    ) {
      return "nintendo";
    }
    if (raw.includes("playstation") || raw.includes("ps1") || raw.includes("ps2") || raw.includes("ps3") || raw.includes("psp") || raw.includes("vita")) {
      return "playstation";
    }
    if (raw.includes("sega") || raw.includes("dreamcast") || raw.includes("genesis") || raw.includes("saturn")) {
      return "sega";
    }
    if (raw.includes("emulation") || raw.includes("retro") || raw.includes("arcade") || raw === "🕹️") {
      return "emulationstation";
    }
    if (raw.includes("standalone") || raw.includes("other") || raw.includes("custom") || raw === "⚙" || raw === "🎲") {
      return "standalone";
    }
    return "standalone";
  });

  let dim = $derived(typeof size === "number" ? `${size}px` : size);
</script>

<svg
  class="platform-svg-icon {className}"
  width={dim}
  height={dim}
  viewBox="0 0 24 24"
  fill="none"
  style="color: {color}; flex-shrink: 0; display: inline-block; vertical-align: middle;"
  aria-hidden="true"
>
  {#if key === "steam"}
    <!-- Authentic Steam piston & crank silhouette -->
    <path
      fill="currentColor"
      d="M12 2a10 10 0 0 0-9.96 9.04l5.36 2.21a2.83 2.83 0 0 1 1.6-.5c.1 0 .2 0 .3.02l2.4-3.48v-.05a3.8 3.8 0 1 1 3.8 3.8h-.09l-3.42 2.44c0 .08.01.16.01.24a2.84 2.84 0 0 1-5.63.57L2.5 14.7A10 10 0 1 0 12 2zm3.5 5.34a2.53 2.53 0 1 0 0 5.06 2.53 2.53 0 0 0 0-5.06zm0 .84a1.69 1.69 0 1 1 0 3.38 1.69 1.69 0 0 1 0-3.38zM9 13.86c-.53 0-1.02.22-1.37.58l1.15.48a1.48 1.48 0 1 0 1.12-2.74l-1.15-.48c.08-.01.17-.02.25-.02a2.18 2.18 0 1 1 0 4.36 2.18 2.18 0 0 1-2.06-1.48l1.24.51c.22.18.5.29.82.29a1.3 1.3 0 0 0 0-2.5z"
    />
  {:else if key === "gog"}
    <!-- Authentic GOG Galaxy geometric cube/ring emblem -->
    <path
      fill="currentColor"
      d="M12 2L3.5 6.9v10.2L12 22l8.5-4.9V6.9L12 2zm5.8 13.5L12 18.8l-5.8-3.3V8.5L12 5.2l5.8 3.3v7zm-5.8-7.6L8.2 10v4l3.8 2.1 3.8-2.1v-4L12 7.9z"
    />
  {:else if key === "epic"}
    <!-- Authentic Epic Games shield crest -->
    <path
      fill="currentColor"
      d="M5 2.5h14A1.5 1.5 0 0 1 20.5 4v11.8c0 1.1-.6 2.1-1.56 2.63L12 22l-6.94-3.57A2.98 2.98 0 0 1 3.5 15.8V4A1.5 1.5 0 0 1 5 2.5zm2.5 3.5v8h2.2v-3h2.1v-1.8H9.7V7.8h2.6V6H7.5zm6 0v8h2.2V6h-2.2zM12 16.5l3.5 1.8 3.5-1.8H5l7 0z"
    />
  {:else if key === "itch"}
    <!-- Authentic itch.io arcade storefront roof & controller silhouette -->
    <path
      fill="currentColor"
      d="M4.2 3h15.6L22 7.2v1.3c0 1.4-1.1 2.5-2.5 2.5-.9 0-1.7-.5-2.1-1.2-.4.7-1.2 1.2-2.1 1.2-.9 0-1.7-.5-2.1-1.2-.4.7-1.2 1.2-2.1 1.2-.9 0-1.7-.5-2.1-1.2-.4.7-1.2 1.2-2.1 1.2C5.5 11 4.4 9.9 4.4 8.5V7.2L4.2 3zm-.2 9.2c.8.4 1.7.6 2.6.6.8 0 1.6-.2 2.2-.6.6.4 1.4.6 2.2.6s1.6-.2 2.2-.6c.6.4 1.4.6 2.2.6s1.6-.2 2.2-.6c.7.4 1.5.6 2.4.6V20a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-7.8zm4.5 2.3v1.5H7v1.5h1.5V19H10v-1.5h1.5V16H10v-1.5H8.5zm7.2.5a1.2 1.2 0 1 0 0 2.4 1.2 1.2 0 0 0 0-2.4zm2.3 1.8a1.2 1.2 0 1 0 0 2.4 1.2 1.2 0 0 0 0-2.4z"
    />
  {:else if key === "xbox"}
    <!-- Authentic Xbox sphere X emblem -->
    <path
      fill="currentColor"
      d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zm-4.9 3.2c1.4-.4 3.1.2 4.9 1.6 1.8-1.4 3.5-2 4.9-1.6-.9.9-2.1 2.2-3.3 3.7 1.7 2.2 3.4 5 4.2 7.8-1.1-1.5-3.2-3.8-5.8-5.9-2.6 2.1-4.7 4.4-5.8 5.9.8-2.8 2.5-5.6 4.2-7.8-1.2-1.5-2.4-2.8-3.3-3.7z"
    />
  {:else if key === "ea"}
    <!-- Authentic EA monogram roundel -->
    <path
      fill="currentColor"
      d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zm-4.8 6.5h5.6l-.9 1.8H6.3l.9-1.8zm8.1 0L19 15.5h-2.1l-.7-1.5h-2.7l-.7 1.5H5l2.1-4.2h4.2l-.8 1.6H8.3l-.5 1.1h3.6l2.9-5.5zm-.2 2.5l-.8 1.7h1.6l-.8-1.7z"
    />
  {:else if key === "ubisoft"}
    <!-- Authentic Ubisoft spiral emblem -->
    <path
      fill="currentColor"
      d="M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm0 17.6A7.6 7.6 0 0 1 4.4 12c0-3.2 2-5.9 4.8-7a6.2 6.2 0 0 0-2 4.6 6.2 6.2 0 0 0 6.2 6.2 4.8 4.8 0 0 0 4.8-4.8 3.6 3.6 0 0 0-3.6-3.6 2.5 2.5 0 0 0-2.5 2.5c0 .9.5 1.7 1.3 2.1a1.5 1.5 0 0 1-2.1-1.4 3.9 3.9 0 0 1 3.9-3.9 5.3 5.3 0 0 1 5.3 5.3A7.6 7.6 0 0 1 12 19.6z"
    />
  {:else if key === "battlenet"}
    <!-- Authentic Battle.net interlocking orbital rings -->
    <path
      fill="currentColor"
      d="M12 2c1.6 0 3 1.9 3.7 4.7 2.8-.6 5.1.1 5.9 1.5.8 1.4-.5 3.6-2.7 5.3 1.4 2.5 1.5 4.9.1 5.7-1.4.8-3.8-.2-5.8-2.1-1.9 1.9-4.3 2.9-5.7 2.1-1.4-.8-1.3-3.2.1-5.7-2.2-1.7-3.5-3.9-2.7-5.3.8-1.4 3.1-2.1 5.9-1.5C9 3.9 10.4 2 12 2zm0 6.8a3.2 3.2 0 1 0 0 6.4 3.2 3.2 0 0 0 0-6.4z"
    />
  {:else if key === "amazon"}
    <!-- Authentic Amazon Gaming crown & smile arc -->
    <path
      fill="currentColor"
      d="M3.5 6.5l4.2 3.2L12 5l4.3 4.7 4.2-3.2-1.6 8.5H5.1L3.5 6.5zm1.2 11c4.5 2.5 10.1 2.5 14.6 0l.9 1.5c-5.1 3-11.3 3-16.4 0l.9-1.5z"
    />
  {:else if key === "rockstar"}
    <!-- Authentic Rockstar bevel star emblem -->
    <path
      fill="currentColor"
      d="M4 3h16a1 1 0 0 1 1 1v16a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1zm11.5 7.5l-1.1 3.3-3.4.1 2.7 2-1 3.3 2.8-2 2.8 2-1-3.3 2.7-2-3.4-.1-1.1-3.3zM6 6.5v9h2.2v-3.2h1.6l1.5 3.2h2.4l-1.8-3.5c.9-.4 1.5-1.2 1.5-2.4 0-1.9-1.4-3.1-3.6-3.1H6zm2.2 1.8h1.5c.9 0 1.5.4 1.5 1.2s-.6 1.2-1.5 1.2H8.2V8.3z"
    />
  {:else if key === "nintendo"}
    <!-- Authentic GameCube / Console isometric cube & D-pad crest -->
    <path
      fill="currentColor"
      d="M12 2L3.5 6.8v10.4L12 22l8.5-4.8V6.8L12 2zm0 2.5l6.1 3.4-2.3 1.3-3.8-2.1-3.8 2.1v4.3l3.8 2.1 3.8-2.1v-1.4l-2.2 1.2v-2.1l4.4-2.5v6L12 18.2l-6.1-3.5V7.9L12 4.5z"
    />
  {:else if key === "playstation"}
    <!-- Authentic PlayStation controller symbols crest -->
    <path
      fill="currentColor"
      d="M10.5 3v14.8l-3.2-1V5.5L10.5 3zm2 .8l5.2 1.7c1.6.5 2.6 1.6 2.6 3.3 0 1.8-1.1 2.8-2.8 2.3l-2.8-.9V7.5l1.7.5c.5.2.9 0 .9-.5 0-.4-.3-.7-.8-.9L12.5 5.6V3.8zM3.2 15.6l5.3-1.7v2.1l-3.1 1c-.6.2-.9.5-.9.9 0 .4.4.6 1 .5l3-.9v2.1l-4.2 1.3c-1.8.5-3.2-.1-3.2-1.8 0-1.5.8-2.8 2.1-3.5zm11.8-.3l4.2-1.3c1.6-.5 2.8.2 2.8 1.6 0 1.3-.9 2.5-2.4 3l-4.6 1.4v-2.1l2.8-.9c.5-.2.8-.5.8-.9 0-.4-.4-.6-.9-.4L15 16.7v-1.4z"
    />
  {:else if key === "sega"}
    <!-- Authentic Dreamcast / Sega swirl emblem -->
    <path
      fill="currentColor"
      d="M12 3a9 9 0 1 0 8.5 11.9 1 1 0 0 0-1.9-.6A7 7 0 1 1 12 5a5.5 5.5 0 0 1 5.5 5.5 4 4 0 0 1-4 4 2.5 2.5 0 0 1-2.5-2.5c0-.8.7-1.5 1.5-1.5a1 1 0 1 0 0-2 3.5 3.5 0 0 0-3.5 3.5 4.5 4.5 0 0 0 4.5 4.5 6 6 0 0 0 6-6A7.5 7.5 0 0 0 12 3z"
    />
  {:else if key === "emulationstation"}
    <!-- Authentic EmulationStation / Arcade joystick & buttons -->
    <path
      fill="currentColor"
      d="M6.5 6h11A4.5 4.5 0 0 1 22 10.5v5a3.5 3.5 0 0 1-6.1 2.3L14 15.5h-4l-1.9 2.3A3.5 3.5 0 0 1 2 15.5v-5A4.5 4.5 0 0 1 6.5 6zm.5 2.5v1.8H5.2v1.4H7v1.8h1.4v-1.8h1.8v-1.4H8.4V8.5H7zm9.5.5a1.2 1.2 0 1 0 0 2.4 1.2 1.2 0 0 0 0-2.4zm2.2 2.2a1.2 1.2 0 1 0 0 2.4 1.2 1.2 0 0 0 0-2.4z"
    />
  {:else if key === "favorite"}
    <!-- Star crest -->
    <path
      fill="currentColor"
      d="M12 2.5l2.9 6 6.6.9-4.8 4.6 1.1 6.5L12 17.4l-5.8 3.1 1.1-6.5-4.8-4.6 6.6-.9L12 2.5z"
    />
  {:else if key === "all"}
    <!-- 4-tile grid library icon -->
    <path
      fill="currentColor"
      d="M3.5 3.5h7v7h-7v-7zm10 0h7v7h-7v-7zm-10 10h7v7h-7v-7zm10 0h7v7h-7v-7z"
    />
  {:else}
    <!-- Standalone PC Desktop Window & Gamepad crest -->
    <path
      fill="currentColor"
      d="M3 4a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2h-5l1.5 3H17v2H7v-2h1.5L10 17H5a2 2 0 0 1-2-2V4zm2 2v9h14V6H5zm3.5 2.5h1.5v1.5h1.5v1.5H10V13H8.5v-1.5H7V10h1.5V8.5zm6 .5a1 1 0 1 1 0 2 1 1 0 0 1 0-2zm2 1.5a1 1 0 1 1 0 2 1 1 0 0 1 0-2z"
    />
  {/if}
</svg>
