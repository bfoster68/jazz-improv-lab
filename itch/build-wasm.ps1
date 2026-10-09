# Build the Bevy app for the browser and zip it for itch.io.   Windows / PowerShell
#
#   .\build-wasm.ps1              build + package
#   .\build-wasm.ps1 -Serve       build, package, then serve locally to test
#
# One-time setup:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version <must match the wasm-bindgen crate version>
#   cargo install wasm-opt          # optional, roughly halves the download

param([switch]$Serve)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$dist = Join-Path $PSScriptRoot 'dist'

Write-Host '==> building wasm (release)' -ForegroundColor Cyan
Push-Location (Join-Path $root 'rust')
cargo build --release --target wasm32-unknown-unknown -p jazz-app
Pop-Location

# wasm-bindgen-cli must match the wasm-bindgen crate in Cargo.lock exactly, or
# it refuses the module. Read the locked version and (re)install on mismatch.
$lockHit = Select-String -Path (Join-Path $root 'rust\Cargo.lock') `
  -Pattern '^name = "wasm-bindgen"$' -Context 0,1
$wanted = ($lockHit.Context.PostContext[0] -replace 'version = "([^"]+)"', '$1').Trim()
$have = ''
if (Get-Command wasm-bindgen -ErrorAction SilentlyContinue) {
  $have = ((wasm-bindgen --version) -split ' ')[1]
}
if ($have -ne $wanted) {
  Write-Host "==> installing wasm-bindgen-cli $wanted (found: '$have')" -ForegroundColor Cyan
  cargo install wasm-bindgen-cli --version $wanted --force
}

Write-Host '==> running wasm-bindgen' -ForegroundColor Cyan
Remove-Item -Recurse -Force $dist -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $dist | Out-Null
wasm-bindgen --no-typescript --target web `
  --out-dir $dist `
  --out-name jazz-app `
  (Join-Path $root 'rust\target\wasm32-unknown-unknown\release\jazz-app.wasm')

if (Get-Command wasm-opt -ErrorAction SilentlyContinue) {
  Write-Host '==> optimising with wasm-opt' -ForegroundColor Cyan
  wasm-opt -Oz --output (Join-Path $dist 'jazz-app_bg.wasm') (Join-Path $dist 'jazz-app_bg.wasm')
} else {
  Write-Host '   (wasm-opt not found - skipping; the upload will be larger)' -ForegroundColor Yellow
}

Copy-Item (Join-Path $PSScriptRoot 'bevy-shell\index.html') $dist

# The band's instruments ride along: the page downloads soundfont.sf2 before
# the app starts (see the boot script in index.html).
$sf2 = Join-Path $root 'rust\crates\jazz-app\assets\soundfont.sf2'
if (-not (Test-Path $sf2)) {
  Write-Host '==> fetching the SoundFont (first time only)' -ForegroundColor Cyan
  & (Join-Path $root 'rust\crates\jazz-app\assets\fetch-soundfont.ps1')
}
Copy-Item $sf2 $dist

$zip = Join-Path $PSScriptRoot 'jazz-lab-bevy-itch.zip'
Remove-Item $zip -ErrorAction SilentlyContinue
Compress-Archive -Path (Join-Path $dist '*') -DestinationPath $zip
$size = [math]::Round((Get-Item $zip).Length / 1MB, 1)
Write-Host "==> wrote $zip ($size MB)" -ForegroundColor Green
Write-Host '    Upload to itch.io, tick "This file will be played in the browser".'

if ($Serve) {
  Write-Host '==> serving http://localhost:8080  (Ctrl+C to stop)' -ForegroundColor Cyan
  Push-Location $dist; python -m http.server 8080; Pop-Location
}
