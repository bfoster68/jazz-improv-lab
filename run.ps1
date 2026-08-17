# Launch the Jazz Improvisation Lab.
#
# The Cargo workspace lives in rust/, not the repo root, and it holds two
# binaries (the app and render-wav) - so a bare `cargo run` fails from here.
# This script is the one-step answer: right-click > Run with PowerShell, or
# just `.\run.ps1` from any prompt.

$ErrorActionPreference = 'Stop'
Push-Location (Join-Path $PSScriptRoot 'rust')
try {
    cargo run -p jazz-app --release
} finally {
    Pop-Location
}
