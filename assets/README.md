# Assets

Source artwork and icon masters for Steam Vault.

## Contents

| File | Description |
| --- | --- |
| `Steam_icon_logo.svg` | Master vector logo (65×65 viewBox). Source: Wikimedia Commons. |
| `steam-icon-1024.png` | 1024×1024 8-bit RGBA raster derived from the SVG; input to `tauri icon`. |

## Regenerating the app icons

```bash
# 1. Rasterize the master SVG to a 1024×1024 RGBA PNG (requires ImageMagick)
magick -background none -density 1024 "assets/Steam_icon_logo.svg" \
    -resize 1024x1024 -depth 8 PNG32:"assets/steam-icon-1024.png"

# 2. Generate the full desktop/mobile icon set into src-tauri/icons/
npm run tauri icon "assets/steam-icon-1024.png"
```

## Attribution & trademark notice

`Steam_icon_logo.svg` is sourced from Wikimedia Commons
(<https://commons.wikimedia.org/wiki/File:Steam_icon_logo.svg>), which hosts it as a
simple-geometry logo. **The Steam logo is a registered trademark of Valve Corporation.**

It is included here solely to identify the application's subject matter (organizing a Steam
library). Steam Vault is an independent, unofficial project and is **not affiliated with,
endorsed by, or sponsored by Valve Corporation**.
