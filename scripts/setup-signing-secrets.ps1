param(
  [string]$CertPath = "certificates/steam-vault-codesign.pfx",
  [string]$Password = "steam-vault"
)

if (-not (Test-Path $CertPath)) {
  Write-Error "Certificate not found at $CertPath"
  exit 1
}

$bytes = [System.IO.File]::ReadAllBytes((Resolve-Path $CertPath).Path)
$base64 = [System.Convert]::ToBase64String($bytes)

Write-Host "Uploading WINDOWS_CERTIFICATE_BASE64 to GitHub Secrets..."
$base64 | gh secret set WINDOWS_CERTIFICATE_BASE64 --repo HELIX-Origin/Steam-Vault

Write-Host "Uploading WINDOWS_CERTIFICATE_PASSWORD to GitHub Secrets..."
$Password | gh secret set WINDOWS_CERTIFICATE_PASSWORD --repo HELIX-Origin/Steam-Vault

Write-Host "Successfully configured signing secrets on HELIX-Origin/Steam-Vault!"
