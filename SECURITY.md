# Security Policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately via GitHub Security Advisories
("Report a vulnerability") on the `HELIX-Origin/Steam-Vault` repository, or by opening a
minimal issue that does not disclose exploit details.

## Secrets & API keys

Steam Vault never hardcodes credentials. API keys you provide (Steam Web API, SteamGridDB,
IGDB/Twitch) are stored **only** in your local configuration directory
(`<config>/steam-vault/settings.json`) on your own machine and are never transmitted anywhere
except to the corresponding provider's API.

- Do not commit `settings.json`, `.env` files, or any credential material.
- The repository `.gitignore` excludes local settings and scratch files.

## Local writes

Steam Vault modifies local Steam files: collections
(`config/cloudstorage/cloud-storage-namespace-1.json`), non-Steam shortcuts (`config/shortcuts.vdf`),
the library-folder registry (`libraryfolders.vdf`, every copy), and an app's manifest
(`appmanifest_<appid>.acf`) when it is moved between libraries. Every write is preceded by a
timestamped backup under the app data directory, and writes are refused while Steam is running.
