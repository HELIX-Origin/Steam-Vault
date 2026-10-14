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

Steam Vault modifies local Steam files (`config/cloudstorage/cloud-storage-namespace-1.json`
and `config/shortcuts.vdf`). Every write is preceded by a timestamped backup under the app data
directory, and writes are refused while Steam is running.
