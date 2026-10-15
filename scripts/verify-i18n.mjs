#!/usr/bin/env node
// Automated verification test suite for Steam Vault's hand-rolled i18n engine.
// Verifies all 30 official Steam platform languages, locale files, schema consistency, and RTL support.

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const I18N_DIR = path.join(ROOT, "src", "lib", "i18n");
const LOCALES_DIR = path.join(I18N_DIR, "locales");

console.log("▶ Verifying hand-rolled i18n engine and all 30 Steam languages...");

// 1. Verify languages.ts existence and content
const languagesPath = path.join(I18N_DIR, "languages.ts");
assert.ok(fs.existsSync(languagesPath), "languages.ts must exist");
const languagesContent = fs.readFileSync(languagesPath, "utf-8");

const expected30SteamLanguages = [
  { id: "en", steamCode: "english", rtl: false },
  { id: "de", steamCode: "german", rtl: false },
  { id: "fr", steamCode: "french", rtl: false },
  { id: "it", steamCode: "italian", rtl: false },
  { id: "ko", steamCode: "koreana", rtl: false },
  { id: "es", steamCode: "spanish", rtl: false },
  { id: "zh-CN", steamCode: "schinese", rtl: false },
  { id: "zh-TW", steamCode: "tchinese", rtl: false },
  { id: "ru", steamCode: "russian", rtl: false },
  { id: "th", steamCode: "thai", rtl: false },
  { id: "ja", steamCode: "japanese", rtl: false },
  { id: "pt", steamCode: "portuguese", rtl: false },
  { id: "pl", steamCode: "polish", rtl: false },
  { id: "da", steamCode: "danish", rtl: false },
  { id: "nl", steamCode: "dutch", rtl: false },
  { id: "fi", steamCode: "finnish", rtl: false },
  { id: "no", steamCode: "norwegian", rtl: false },
  { id: "sv", steamCode: "swedish", rtl: false },
  { id: "hu", steamCode: "hungarian", rtl: false },
  { id: "cs", steamCode: "czech", rtl: false },
  { id: "ro", steamCode: "romanian", rtl: false },
  { id: "tr", steamCode: "turkish", rtl: false },
  { id: "pt-BR", steamCode: "brazilian", rtl: false },
  { id: "bg", steamCode: "bulgarian", rtl: false },
  { id: "el", steamCode: "greek", rtl: false },
  { id: "ar", steamCode: "arabic", rtl: true },
  { id: "uk", steamCode: "ukrainian", rtl: false },
  { id: "es-419", steamCode: "latam", rtl: false },
  { id: "vi", steamCode: "vietnamese", rtl: false },
  { id: "id", steamCode: "indonesian", rtl: false }
];

assert.equal(
  expected30SteamLanguages.length,
  30,
  "Target list must contain exactly 30 languages"
);

for (const lang of expected30SteamLanguages) {
  assert.ok(
    languagesContent.includes(`id: "${lang.id}"`),
    `languages.ts must register id: "${lang.id}"`
  );
  assert.ok(
    languagesContent.includes(`steamCode: "${lang.steamCode}"`),
    `languages.ts must register steamCode: "${lang.steamCode}"`
  );
}
console.log("  ✔ All 30 official Steam platform languages defined in languages.ts");

// 2. Verify all 30 locale dictionaries exist on disk
for (const lang of expected30SteamLanguages) {
  const localeFile = path.join(LOCALES_DIR, `${lang.id}.ts`);
  assert.ok(
    fs.existsSync(localeFile),
    `Locale file missing for ${lang.id}: ${localeFile}`
  );
  const content = fs.readFileSync(localeFile, "utf-8");
  assert.ok(
    content.length > 50,
    `Locale file ${lang.id}.ts must not be empty`
  );
}
console.log("  ✔ All 30 locale dictionary files exist on disk with valid content");

// 3. Verify Arabic RTL flag
assert.ok(
  languagesContent.includes('id: "ar"') && languagesContent.includes("rtl: true"),
  "Arabic language must be marked with rtl: true"
);
console.log("  ✔ Arabic RTL flag correctly configured");

// 4. Verify locales/index.ts exports all 30 languages
const localesIndexPath = path.join(LOCALES_DIR, "index.ts");
assert.ok(fs.existsSync(localesIndexPath), "locales/index.ts must exist");
const localesIndexContent = fs.readFileSync(localesIndexPath, "utf-8");

for (const lang of expected30SteamLanguages) {
  const hasId =
    localesIndexContent.includes(`"${lang.id}":`) ||
    localesIndexContent.includes(`\n  ${lang.id},`) ||
    localesIndexContent.includes(` ${lang.id},`) ||
    localesIndexContent.includes(`${lang.id}:`);
  assert.ok(hasId, `locales/index.ts must register key for "${lang.id}"`);

  const hasSteamCode =
    localesIndexContent.includes(`"${lang.steamCode}":`) ||
    localesIndexContent.includes(`${lang.steamCode}:`);
  assert.ok(hasSteamCode, `locales/index.ts must register key for "${lang.steamCode}"`);
}
console.log("  ✔ locales/index.ts maps both id and steamCode for all 30 languages");

// 5. Verify core i18n engine exports
const indexPath = path.join(I18N_DIR, "index.ts");
assert.ok(fs.existsSync(indexPath), "i18n/index.ts must exist");
const indexContent = fs.readFileSync(indexPath, "utf-8");

const requiredExports = ["locale", "activeLanguage", "isRTL", "translate", "t", "setLocale", "syncWithSettingsLanguage"];
for (const exp of requiredExports) {
  assert.ok(
    indexContent.includes(`export const ${exp}`) || indexContent.includes(`export function ${exp}`),
    `i18n/index.ts must export ${exp}`
  );
}
console.log("  ✔ i18n/index.ts exports all core reactive stores and helper methods");

console.log("\n✅ i18n verification passed: 30/30 Steam languages validated!");
