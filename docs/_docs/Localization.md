---
layout: docs
title: Localization (i18n)
summary: Complete guide to Universal Game Launcher's hand-rolled multilingual architecture and supported languages
parent: Documentation Home
nav_order: 10
---

# Localization (i18n)

Universal Game Launcher features a hand-rolled, zero-dependency reactive internationalization engine designed specifically for multi-platform game library management. It provides full, native UI localization across all 30 officially supported gaming storefront languages.

---

## 🌐 Supported Languages

Universal Game Launcher provides comprehensive translation across the following languages:

| Language | Code | Storefront API Code | Native Name | Direction |
| :--- | :--- | :--- | :--- | :--- |
| **English** | `en` | `english` | English | Left-to-Right |
| **German** | `de` | `german` | Deutsch | Left-to-Right |
| **French** | `fr` | `french` | Français | Left-to-Right |
| **Italian** | `it` | `italian` | Italiano | Left-to-Right |
| **Korean** | `ko` | `koreana` | 한국어 | Left-to-Right |
| **Spanish (Spain)** | `es` | `spanish` | Español (España) | Left-to-Right |
| **Simplified Chinese** | `zh-CN` | `schinese` | 简体中文 | Left-to-Right |
| **Traditional Chinese** | `zh-TW` | `tchinese` | 繁體中文 | Left-to-Right |
| **Russian** | `ru` | `russian` | Русский | Left-to-Right |
| **Thai** | `th` | `thai` | ไทย | Left-to-Right |
| **Japanese** | `ja` | `japanese` | 日本語 | Left-to-Right |
| **Portuguese (Portugal)** | `pt` | `portuguese` | Português | Left-to-Right |
| **Polish** | `pl` | `polish` | Polski | Left-to-Right |
| **Danish** | `da` | `danish` | Dansk | Left-to-Right |
| **Dutch** | `nl` | `dutch` | Nederlands | Left-to-Right |
| **Finnish** | `fi` | `finnish` | Suomi | Left-to-Right |
| **Norwegian** | `no` | `norwegian` | Norsk | Left-to-Right |
| **Swedish** | `sv` | `swedish` | Svenska | Left-to-Right |
| **Hungarian** | `hu` | `hungarian` | Magyar | Left-to-Right |
| **Czech** | `cs` | `czech` | Čeština | Left-to-Right |
| **Romanian** | `ro` | `romanian` | Română | Left-to-Right |
| **Turkish** | `tr` | `turkish` | Türkçe | Left-to-Right |
| **Portuguese (Brazil)** | `pt-BR` | `brazilian` | Português (Brasil) | Left-to-Right |
| **Bulgarian** | `bg` | `bulgarian` | Български | Left-to-Right |
| **Greek** | `el` | `greek` | Ελληνικά | Left-to-Right |
| **Arabic** | `ar` | `arabic` | العربية | Right-to-Left (RTL) |
| **Ukrainian** | `uk` | `ukrainian` | Українська | Left-to-Right |
| **Spanish (Latin America)** | `es-419` | `latam` | Español (Latinoamérica) | Left-to-Right |
| **Vietnamese** | `vi` | `vietnamese` | Tiếng Việt | Left-to-Right |
| **Indonesian** | `id` | `indonesian` | Bahasa Indonesia | Left-to-Right |

---

## ⚙️ Architecture & Features

1. **Zero External Dependencies**: The localization system is hand-rolled in pure TypeScript and Svelte 5 reactive stores (`$t`), avoiding heavy external translation bundles or runtime overhead.
2. **Instant Reactive Switching**: Changing languages in **Settings** or via the language selector immediately updates the `$t` store across the entire interface without reloading the window.
3. **Bi-directional Layout Support (RTL)**: Dynamic detection applies `dir="rtl"` automatically when Arabic is selected.
4. **Fallback Safety**: If any translation key is missing or incomplete in a locale, the engine automatically falls back to the English dictionary, guaranteeing no broken keys or missing UI elements.
5. **Parameter Interpolation**: Handles parameterized templates such as `{count}` or `{games}` directly.
6. **Persistent Preference**: Stores the user's selected language in local configuration and syncs with `settings.json`.
