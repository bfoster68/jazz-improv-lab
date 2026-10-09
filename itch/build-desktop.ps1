# Package the Windows build: the exe with the SoundFont beside it, zipped.
#
#   .\build-desktop.ps1
#
# find_soundfont looks next to the executable first, so a player who unzips
# this anywhere gets the full FluidR3 band with zero setup.

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$dist = Join-Path $PSScriptRoot 'dist-desktop'

Write-Host '==> building release' -ForegroundColor Cyan
Push-Location (Join-Path $root 'rust')
cargo build --release -p jazz-app
Pop-Location

$sf2 = Join-Path $root 'rust\crates\jazz-app\assets\soundfont.sf2'
if (-not (Test-Path $sf2)) {
  Write-Host '==> fetching the SoundFont (first time only)' -ForegroundColor Cyan
  & (Join-Path $root 'rust\crates\jazz-app\assets\fetch-soundfont.ps1')
}

Write-Host '==> staging' -ForegroundColor Cyan
Remove-Item -Recurse -Force $dist -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $dist | Out-Null
Copy-Item (Join-Path $root 'rust\target\release\jazz-app.exe') $dist
Copy-Item $sf2 $dist

$zip = Join-Path $PSScriptRoot 'jazz-lab-windows.zip'
Remove-Item $zip -ErrorAction SilentlyContinue
Compress-Archive -Path (Join-Path $dist '*') -DestinationPath $zip
$size = [math]::Round((Get-Item $zip).Length / 1MB, 1)
Write-Host "==> wrote $zip ($size MB)" -ForegroundColor Green
Write-Host '    Upload to itch.io as a Windows download (not browser-played).'
