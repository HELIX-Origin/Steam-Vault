$windowsDir = Join-Path $PSScriptRoot "..\src-tauri\windows"
$iconPath = Join-Path $PSScriptRoot "..\src-tauri\icons\icon.png"

if (-not (Test-Path $windowsDir)) {
    New-Item -ItemType Directory -Force -Path $windowsDir | Out-Null
}

if (-not (Test-Path $iconPath)) {
    Write-Error "Icon not found at $iconPath"
    exit 1
}

$sidebarPath = Join-Path $windowsDir "sidebar.bmp"
Write-Host "Generating NSIS sidebar bitmap at $sidebarPath..."

# Use ImageMagick to compose a 164x314 truecolor BMP3 with obsidian gradient (#0d1b2d to #07111f) and 112x112 centered squircle icon
& magick -size 164x314 "gradient:#0d1b2d-#07111f" ( $iconPath -resize 112x112 ) -geometry +26+50 -composite -type truecolor "BMP3:$sidebarPath"

if (Test-Path $sidebarPath) {
    Write-Host "Successfully generated $sidebarPath"
} else {
    Write-Error "Failed to generate $sidebarPath"
    exit 1
}
