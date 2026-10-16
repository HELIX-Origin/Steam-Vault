import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

const USAGE = [
	'Usage: npm run sign [-- options] [-- file ...]',
	'',
	'Opt-in signing step for Steam Vault. It never runs as part of a build:',
	'run it deliberately (after `npm run tauri:build` or `tauri build`) when you want signed artifacts.',
	'',
	'With no file arguments, signs the artifacts discovered for this platform:',
	'  windows   steam-vault.exe, the portable exe, and every .exe/.msi under src-tauri/target/release/bundle',
	'  macOS     every .app and .dmg under src-tauri/target/release/bundle/macos (plus notarization when credentials are set)',
	'  linux     every package under bundle/appimage, bundle/deb and bundle/rpm (gpg detached signature)',
	'',
	'Options:',
	'  --dry-run           print targets and commands without signing anything',
	'  --identity <name>   codesign identity (default: $APPLE_SIGNING_IDENTITY, else the only',
	'                      codesigning identity found in the keychain)',
	'  --cert <path>       code-signing PFX on Windows (default: certificates/*.pfx or $WINDOWS_CERTIFICATE_BASE64)',
	'  --cert-pass <pass>  PFX password (default: $WINDOWS_CERTIFICATE_PASSWORD, else "steam-vault")',
	'  -h, --help          show this help',
	'',
	'Environment:',
	'  APPLE_ID, APPLE_APP_SPECIFIC_PASSWORD, APPLE_TEAM_ID   notarize and staple the .app when all three are set',
	'  APPLE_SIGNING_IDENTITY                                default codesign identity',
	'  WINDOWS_CERTIFICATE_BASE64, WINDOWS_CERTIFICATE_PASSWORD   Windows certificate materialization',
	'',
	'Examples:',
	'  npm run sign',
	'  npm run sign -- --dry-run',
	'  npm run sign -- "src-tauri/target/release/bundle/nsis/Steam Vault_1.2.0_x64-setup.exe"',
	'  npm run sign -- --identity "Developer ID Application: Name (TEAMID)"',
].join('\n')

const SECRETS = new Set()

function fail(message) {
	console.error(`sign: ${message}`)
	process.exit(1)
}

function log(message) {
	console.log(`sign: ${message}`)
}

function usageError(message) {
	console.error(`sign: ${message}`)
	console.error('run `npm run sign -- --help` for usage')
	process.exit(2)
}

function parseArgs(argv) {
	const opts = {
		dry: false,
		help: false,
		identity: null,
		cert: null,
		certPass: null,
		targets: [],
	}
	const value = (flag, index) => {
		if (index + 1 >= argv.length) usageError(`missing value for ${flag}`)
		return argv[index + 1]
	}
	for (let i = 0; i < argv.length; i++) {
		const arg = argv[i]
		if (arg === '-h' || arg === '--help') opts.help = true
		else if (arg === '--dry-run') opts.dry = true
		else if (arg === '--') continue
		else if (arg === '--identity') {
			opts.identity = value(arg, i)
			i++
		} else if (arg.startsWith('--identity=')) opts.identity = arg.slice('--identity='.length)
		else if (arg === '--cert') {
			opts.cert = value(arg, i)
			i++
		} else if (arg.startsWith('--cert=')) opts.cert = arg.slice('--cert='.length)
		else if (arg === '--cert-pass') {
			opts.certPass = value(arg, i)
			i++
		} else if (arg.startsWith('--cert-pass=')) opts.certPass = arg.slice('--cert-pass='.length)
		else if (arg.startsWith('--')) usageError(`unknown option ${arg}`)
		else opts.targets.push(arg)
	}
	return opts
}

function echoCommand(cmd, args) {
	const parts = [cmd, ...args].map((a) => {
		const s = String(a)
		if (SECRETS.has(s)) return '***'
		return s.includes(' ') ? JSON.stringify(s) : s
	})
	console.log(`sign: $ ${parts.join(' ')}`)
}

function run(cmd, args, opts = {}) {
	echoCommand(cmd, args)
	if (opts.dry) return 0
	const res = spawnSync(cmd, args, {
		stdio: 'inherit',
		cwd: ROOT,
		env: { ...process.env, ...(opts.env ?? {}) },
	})
	if (res.error) {
		if (res.error.code === 'ENOENT') fail(opts.missing ?? `${cmd} not found`)
		fail(`${cmd}: ${res.error.message}`)
	}
	return res.status ?? 1
}

function walk(dir) {
	const out = []
	let entries
	try {
		entries = fs.readdirSync(dir, { withFileTypes: true })
	} catch {
		return out
	}
	for (const entry of entries) {
		const full = path.join(dir, entry.name)
		if (entry.isDirectory()) out.push(...walk(full))
		else out.push(full)
	}
	return out
}

function discover() {
	const bundle = path.join(ROOT, 'src-tauri', 'target', 'release', 'bundle')
	const found = []
	const push = (p) => {
		if (fs.existsSync(p)) found.push(p)
	}
	if (process.platform === 'win32') {
		push(path.join(ROOT, 'src-tauri', 'target', 'release', 'steam-vault.exe'))
		push(path.join(ROOT, 'src-tauri', 'target', 'release', 'portable', 'Steam Vault.exe'))
		for (const f of walk(bundle)) {
			if (/\.(exe|msi)$/i.test(f)) found.push(f)
		}
	}
	if (process.platform === 'darwin') {
		const macDir = path.join(bundle, 'macos')
		try {
			for (const e of fs.readdirSync(macDir, { withFileTypes: true })) {
				if (e.isDirectory() && e.name.endsWith('.app')) found.push(path.join(macDir, e.name))
			}
		} catch {}
		const dmgDir = path.join(bundle, 'dmg')
		try {
			for (const e of fs.readdirSync(dmgDir, { withFileTypes: true })) {
				if (!e.isDirectory() && e.name.endsWith('.dmg')) found.push(path.join(dmgDir, e.name))
			}
		} catch {}
	}
	if (process.platform === 'linux') {
		for (const sub of ['appimage', 'deb', 'rpm']) {
			for (const f of walk(path.join(bundle, sub))) {
				if (!f.endsWith('.asc')) found.push(f)
			}
		}
	}
	return [...new Set(found)]
}

function route(targets) {
	const groups = { windows: [], apple: [], gpg: [] }
	for (const t of targets) {
		const lower = t.toLowerCase()
		if (lower.endsWith('.exe') || lower.endsWith('.msi')) {
			if (process.platform !== 'win32') fail(`${path.basename(t)} can only be signed on Windows (signtool)`)
			groups.windows.push(t)
		} else if (lower.endsWith('.app') || lower.endsWith('.dmg')) {
			if (process.platform !== 'darwin') fail(`${path.basename(t)} can only be signed on macOS (codesign)`)
			groups.apple.push(t)
		} else {
			groups.gpg.push(t)
		}
	}
	return groups
}

function inspectSelfSigned(file, pass) {
	const script = [
		'try { $cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($env:STEAM_VAULT_SIGN_PFX_FILE, $env:STEAM_VAULT_SIGN_PFX_PASS) -ErrorAction Stop } catch { exit 10 }',
		'if ($cert.Issuer -eq $cert.Subject) { exit 0 } else { exit 1 }',
	].join('; ')
	const res = spawnSync('powershell', ['-NoProfile', '-Command', script], {
		stdio: 'ignore',
		cwd: ROOT,
		env: { ...process.env, STEAM_VAULT_SIGN_PFX_FILE: file, STEAM_VAULT_SIGN_PFX_PASS: pass },
	})
	if (res.error) fail('powershell not found — needed to inspect the code-signing certificate')
	if (res.status === 0) return true
	if (res.status === 1) return false
	fail(`cannot read the code-signing certificate at ${file} — check --cert-pass or WINDOWS_CERTIFICATE_PASSWORD`)
}

function resolveCert(opts) {
	const pass = opts.certPass ?? process.env.WINDOWS_CERTIFICATE_PASSWORD ?? 'steam-vault'
	if (opts.cert) {
		const p = path.resolve(ROOT, opts.cert)
		if (!fs.existsSync(p)) fail(`certificate not found: ${p}`)
		return { file: p, pass, selfSigned: inspectSelfSigned(p, pass) }
	}
	for (const candidate of [
		'certificates/steam-vault-codesign.pfx',
		'certificates/steam-vault.pfx',
		'certificates/steam-desktop-codesign.pfx',
	]) {
		const p = path.join(ROOT, candidate)
		if (fs.existsSync(p)) return { file: p, pass, selfSigned: inspectSelfSigned(p, pass) }
	}
	if (process.env.WINDOWS_CERTIFICATE_BASE64) {
		const p = path.join(ROOT, 'certificates', 'steam-vault-codesign.pfx')
		fs.mkdirSync(path.dirname(p), { recursive: true })
		fs.writeFileSync(p, Buffer.from(process.env.WINDOWS_CERTIFICATE_BASE64, 'base64'))
		log(`materialized certificate from WINDOWS_CERTIFICATE_BASE64 -> ${p}`)
		return { file: p, pass, selfSigned: inspectSelfSigned(p, pass) }
	}
	return { file: path.join(ROOT, 'certificates', 'steam-vault-codesign.pfx'), pass, selfSigned: true }
}

function ensureCert(cert, dry) {
	if (!cert.selfSigned || fs.existsSync(cert.file)) return
	if (dry) {
		log(`would generate a self-signed certificate at ${cert.file} (not a trusted publisher identity)`)
		return
	}
	log('no certificate found — generating a local self-signed code-signing certificate')
	fs.mkdirSync(path.dirname(cert.file), { recursive: true })
	const script = [
		"$cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject 'CN=HELIX Origin, O=HELIX Origin, OU=Steam Vault' -CertStoreLocation 'Cert:\\CurrentUser\\My'",
		'$pwd = ConvertTo-SecureString -String $env:STEAM_VAULT_SIGN_PFX_PASS -Force -AsPlainText',
		`Export-PfxCertificate -Cert $cert -FilePath '${cert.file.replace(/'/g, "''")}' -Password $pwd | Out-Null`,
	].join('; ')
	const st = run('powershell', ['-NoProfile', '-Command', script], {
		env: { STEAM_VAULT_SIGN_PFX_PASS: cert.pass },
		missing: 'powershell not found — needed to generate a self-signed certificate',
	})
	if (st !== 0) fail('failed to generate the self-signed certificate')
	if (!fs.existsSync(cert.file)) fail(`certificate was not created at ${cert.file}`)
}

function findSigntool() {
	const base = 'C:\\Program Files (x86)\\Windows Kits\\10\\bin'
	try {
		const versions = fs
			.readdirSync(base)
			.filter((v) => /^\d/.test(v))
			.sort()
			.reverse()
		for (const v of versions) {
			const p = path.join(base, v, 'x64', 'signtool.exe')
			if (fs.existsSync(p)) return p
		}
	} catch {}
	return 'signtool.exe'
}

function signWindows(files, opts, dry) {
	const cert = resolveCert(opts)
	SECRETS.add(cert.pass)
	ensureCert(cert, dry)
	const signtool = findSigntool()
	const missing = 'signtool.exe not found — install the Windows 10/11 SDK'
	for (const f of files) {
		log(`windows: signing ${path.relative(ROOT, f)}`)
		const signed = run(
			signtool,
			['sign', '/f', cert.file, '/p', cert.pass, '/fd', 'sha256', '/tr', 'http://timestamp.digicert.com', '/td', 'sha256', f],
			{ dry, missing },
		)
		if (signed !== 0) fail(`signtool sign failed for ${f}`)
		if (cert.selfSigned) {
			const probe = [
				`$s = Get-AuthenticodeSignature -LiteralPath '${f.replace(/'/g, "''")}'`,
				"if ($s.SignerCertificate -and $s.Status -ne 'HashMismatch' -and $s.Status -ne 'NotSigned') { exit 0 }",
				"Write-Host \"signature status: $($s.Status) - $($s.StatusMessage)\"",
				'exit 1',
			].join('; ')
			const verified = run('powershell', ['-NoProfile', '-Command', probe], {
				dry,
				missing: 'powershell not found — needed to verify the signature',
			})
			if (verified !== 0) fail(`signature verification failed for ${f}`)
		} else if (run(signtool, ['verify', '/pa', f], { dry, missing }) !== 0) {
			fail(`signature verification failed for ${f}`)
		}
		if (dry) continue
		if (cert.selfSigned) log(`windows: verified signature on ${path.basename(f)} (self-signed, not trust-chain verified)`)
		else log(`windows: verified signature on ${path.basename(f)}`)
	}
}

function macIdentity(opts) {
	if (opts.identity) return opts.identity
	if (process.env.APPLE_SIGNING_IDENTITY) return process.env.APPLE_SIGNING_IDENTITY
	const res = spawnSync('security', ['find-identity', '-v', '-p', 'codesigning'], { encoding: 'utf8', cwd: ROOT })
	if (res.error) fail('security not found — cannot determine a codesign identity, pass --identity')
	const ids = [...(res.stdout ?? '').matchAll(/\d+\)\s+[0-9A-F]+\s+"(.+)"/g)].map((m) => m[1])
	if (ids.length === 1) return ids[0]
	if (ids.length === 0) fail('no codesigning identity found — create one in Keychain Access or pass --identity')
	fail(`multiple codesigning identities found — pass --identity or set APPLE_SIGNING_IDENTITY:\n  ${ids.join('\n  ')}`)
}

function notarize(apps, dry) {
	const enabled = ['APPLE_ID', 'APPLE_APP_SPECIFIC_PASSWORD', 'APPLE_TEAM_ID'].every((k) => process.env[k])
	if (!enabled) {
		if (apps.length) log('notarization skipped — set APPLE_ID, APPLE_APP_SPECIFIC_PASSWORD and APPLE_TEAM_ID to enable it')
		return
	}
	SECRETS.add(process.env.APPLE_APP_SPECIFIC_PASSWORD)
	for (const app of apps) {
		const zip = `${app}.notarize.zip`
		log(`macos: notarizing ${path.basename(app)}`)
		if (!dry) fs.rmSync(zip, { force: true })
		if (run('ditto', ['-c', '-k', '--keepParent', app, zip], { dry, missing: 'ditto not found' }) !== 0)
			fail(`failed to zip ${app} for notarization`)
		const submitted = run(
			'xcrun',
			[
				'notarytool',
				'submit',
				zip,
				'--apple-id',
				process.env.APPLE_ID,
				'--password',
				process.env.APPLE_APP_SPECIFIC_PASSWORD,
				'--team-id',
				process.env.APPLE_TEAM_ID,
				'--wait',
			],
			{ dry, missing: 'xcrun notarytool not found' },
		)
		if (submitted !== 0) fail(`notarization failed for ${app}`)
		if (run('xcrun', ['stapler', 'staple', app], { dry, missing: 'xcrun stapler not found' }) !== 0)
			fail(`failed to staple ${app}`)
		if (run('xcrun', ['stapler', 'validate', app], { dry }) !== 0) fail(`staple validation failed for ${app}`)
		if (!dry) fs.rmSync(zip, { force: true })
		if (!dry) log(`macos: ${path.basename(app)} notarized and stapled`)
	}
}

function signApple(files, opts, dry) {
	const identity = macIdentity(opts)
	for (const f of files) {
		log(`macos: signing ${path.relative(ROOT, f)} (${identity})`)
		const signed = run('codesign', ['--force', '--deep', '--options', 'runtime', '--timestamp', '--sign', identity, f], {
			dry,
			missing: 'codesign not found — install Xcode Command Line Tools',
		})
		if (signed !== 0) fail(`codesign failed for ${f}`)
		const verified = run('codesign', ['--verify', '--deep', '--strict', '--verbose=2', f], {
			dry,
			missing: 'codesign not found — install Xcode Command Line Tools',
		})
		if (verified !== 0) fail(`codesign verification failed for ${f}`)
		if (!dry) log(`macos: verified signature on ${path.basename(f)}`)
	}
	notarize(
		files.filter((f) => f.toLowerCase().endsWith('.app')),
		dry,
	)
}

function signLinux(files, dry) {
	for (const f of files) {
		log(`linux: signing ${path.relative(ROOT, f)}`)
		const signed = run('gpg', ['--armor', '--detach-sign', f], { dry, missing: 'gpg not found — install GnuPG' })
		if (signed !== 0) fail(`gpg signing failed for ${f}`)
		const asc = `${f}.asc`
		if (run('gpg', ['--verify', asc, f], { dry, missing: 'gpg not found — install GnuPG' }) !== 0)
			fail(`gpg verification failed for ${asc}`)
		if (!dry) log(`linux: verified ${path.basename(asc)}`)
	}
}

function main() {
	const opts = parseArgs(process.argv.slice(2))
	if (opts.help) {
		console.log(USAGE)
		return
	}
	let targets = opts.targets.map((t) => path.resolve(ROOT, t))
	for (const t of targets) {
		if (!fs.existsSync(t)) fail(`target not found: ${t}`)
	}
	if (!targets.length) {
		targets = discover()
		if (!targets.length)
			fail('nothing to sign — build first (npm run tauri:build) or pass files explicitly')
	}
	log(`platform ${process.platform} — ${targets.length} target(s)`)
	const groups = route(targets)
	if (groups.windows.length) signWindows(groups.windows, opts, opts.dry)
	if (groups.apple.length) signApple(groups.apple, opts, opts.dry)
	if (groups.gpg.length) signLinux(groups.gpg, opts.dry)
	log(opts.dry ? 'dry run complete — nothing signed' : `ok — ${targets.length} target(s) signed and verified`)
}

try {
	main()
} catch (error) {
	fail(error?.message ?? String(error))
}
