#!/usr/bin/env node

/**
 * WinGet Manifest Generator for Steam Vault
 *
 * Generates official Microsoft WinGet Package Manager manifests for HELIXOrigin.SteamVault.
 * Standard output structure:
 *   winget/manifests/h/HELIXOrigin/SteamVault/<version>/HELIXOrigin.SteamVault.yaml
 *   winget/manifests/h/HELIXOrigin/SteamVault/<version>/HELIXOrigin.SteamVault.installer.yaml
 *   winget/manifests/h/HELIXOrigin/SteamVault/<version>/HELIXOrigin.SteamVault.locale.en-US.yaml
 *
 * Usage:
 *   node scripts/generate-winget-manifest.mjs [--version 1.2.0] [--sha256 <hash>] [--local-file <path>]
 */

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const log = (tag, msg) => console.log(`\x1b[36m[${tag}]\x1b[0m ${msg}`);
const err = (tag, msg) => console.error(`\x1b[31m[${tag}]\x1b[0m ${msg}`);

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, '..');

function parseArgs() {
  const args = process.argv.slice(2);
  const options = {};
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--version' && args[i + 1]) {
      options.version = args[++i].replace(/^v/, '');
    } else if (args[i] === '--sha256' && args[i + 1]) {
      options.sha256 = args[++i].toUpperCase();
    } else if (args[i] === '--local-file' && args[i + 1]) {
      options.localFile = args[++i];
    }
  }
  return options;
}

function computeSha256(filePath) {
  const fileBuffer = fs.readFileSync(filePath);
  const hashSum = crypto.createHash('sha256');
  hashSum.update(fileBuffer);
  return hashSum.digest('hex').toUpperCase();
}

async function main() {
  log('WinGet', 'Generating Microsoft WinGet package manifests...');

  const options = parseArgs();
  const pkg = JSON.parse(fs.readFileSync(path.join(rootDir, 'package.json'), 'utf8'));
  const version = options.version || pkg.version;
  const packageIdentifier = 'HELIXOrigin.SteamVault';

  log('WinGet', `Target package: ${packageIdentifier} v${version}`);

  let installerSha256 = options.sha256;
  if (!installerSha256 && options.localFile && fs.existsSync(options.localFile)) {
    log('WinGet', `Calculating SHA256 from local file: ${options.localFile}`);
    installerSha256 = computeSha256(options.localFile);
  }

  // Look for built NSIS installer locally if not provided
  if (!installerSha256) {
    const candidatePaths = [
      path.join(rootDir, 'src-tauri', 'target', 'release', 'bundle', 'nsis', `Steam Vault_${version}_x64-setup.exe`),
      path.join(rootDir, 'src-tauri', 'target', 'release', 'bundle', 'nsis', `Steam.Vault_${version}_x64-setup.exe`),
      path.join(rootDir, 'src-tauri', 'target', 'release', 'bundle', 'nsis', `steam-vault_${version}_x64-setup.exe`)
    ];
    for (const p of candidatePaths) {
      if (fs.existsSync(p)) {
        log('WinGet', `Found local installer: ${p}`);
        installerSha256 = computeSha256(p);
        break;
      }
    }
  }

  if (!installerSha256) {
    installerSha256 = '0000000000000000000000000000000000000000000000000000000000000000';
    log('WinGet', 'Note: No local installer or --sha256 supplied; placeholder hash inserted.');
  }

  const manifestDir = path.join(rootDir, 'winget', 'manifests', 'h', 'HELIXOrigin', 'SteamVault', version);
  fs.mkdirSync(manifestDir, { recursive: true });

  const installerUrl = `https://github.com/HELIX-Origin/Steam-Vault/releases/download/v${version}/Steam.Vault_${version}_x64-setup.exe`;

  // 1. Version manifest
  const versionYaml = `# Created using wingetcreate 1.6.0.0
# yaml-language-server: $schema=https://aka.ms/winget-manifest.version.1.6.0.schema.json

PackageIdentifier: ${packageIdentifier}
PackageVersion: ${version}
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
`;

  // 2. Installer manifest
  const installerYaml = `# Created using wingetcreate 1.6.0.0
# yaml-language-server: $schema=https://aka.ms/winget-manifest.installer.1.6.0.schema.json

PackageIdentifier: ${packageIdentifier}
PackageVersion: ${version}
MinimumOSVersion: 10.0.0.0
InstallerType: nullsoft
InstallModes:
  - interactive
  - silent
Installers:
  - Architecture: x64
    InstallerUrl: ${installerUrl}
    InstallerSha256: ${installerSha256}
    Scope: user
    InstallerLocale: en-US
    UpgradeBehavior: install
ManifestType: installer
ManifestVersion: 1.6.0
`;

  // 3. Locale manifest
  const localeYaml = `# Created using wingetcreate 1.6.0.0
# yaml-language-server: $schema=https://aka.ms/winget-manifest.defaultLocale.1.6.0.schema.json

PackageIdentifier: ${packageIdentifier}
PackageVersion: ${version}
PackageLocale: en-US
Publisher: HELIX Origin
PublisherUrl: https://github.com/HELIX-Origin
PublisherSupportUrl: https://github.com/HELIX-Origin/Steam-Vault/issues
Author: HELIX Origin
PackageName: Steam Vault
PackageUrl: https://github.com/HELIX-Origin/Steam-Vault
License: BSD-3-Clause
LicenseUrl: https://github.com/HELIX-Origin/Steam-Vault/blob/main/LICENSE
Copyright: Copyright (c) 2026 HELIX Origin
ShortDescription: Cross-platform Steam library organizer and manager built with Tauri v2.
Description: Steam Vault revitalizes Depressurizer for modern Steam ecosystems, giving players full control over multi-thousand game libraries, category management, automated tagging, and non-Steam shortcut integrations across GOG, Epic, itch.io, Ubisoft, EA, and emulators.
Tags:
  - steam
  - organizer
  - categories
  - library
  - depressurizer
  - tauri
  - gaming
ManifestType: defaultLocale
ManifestVersion: 1.6.0
`;

  const vFile = path.join(manifestDir, `${packageIdentifier}.yaml`);
  const iFile = path.join(manifestDir, `${packageIdentifier}.installer.yaml`);
  const lFile = path.join(manifestDir, `${packageIdentifier}.locale.en-US.yaml`);

  fs.writeFileSync(vFile, versionYaml, { encoding: 'utf8' });
  fs.writeFileSync(iFile, installerYaml, { encoding: 'utf8' });
  fs.writeFileSync(lFile, localeYaml, { encoding: 'utf8' });

  log('WinGet', `Generated manifests in: ${manifestDir}`);
  log('WinGet', `  -> ${path.basename(vFile)}`);
  log('WinGet', `  -> ${path.basename(iFile)}`);
  log('WinGet', `  -> ${path.basename(lFile)}`);
}

main().catch((e) => {
  err('WinGet', e.message);
  process.exit(1);
});
