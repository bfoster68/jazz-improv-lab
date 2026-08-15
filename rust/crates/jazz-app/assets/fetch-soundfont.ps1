# Fetch the FluidR3 GM SoundFont next to this script.
#
# FluidR3 GM (by Frank Wen, MIT-licensed) is ~141 MB, so it is not committed to
# git. Run this once to install it; the app then plays every pitched instrument
# from real samples.

$ErrorActionPreference = 'Stop'
$dest = Join-Path $PSScriptRoot 'soundfont.sf2'
$url  = 'https://github.com/fhunleth/midi_synth/releases/download/v0.1.0/FluidR3_GM.sf2'

if (Test-Path $dest) {
    $mb = [math]::Round((Get-Item $dest).Length / 1MB)
    Write-Host "soundfont.sf2 already present ($mb MB). Delete it to re-fetch."
    return
}

Write-Host "Downloading FluidR3 GM (~141 MB) to $dest ..."
Invoke-WebRequest -Uri $url -OutFile $dest

# A valid SoundFont2 begins with the RIFF...sfbk magic.
$head = [System.IO.File]::ReadAllBytes($dest)[0..3] -join ','
if ($head -ne '82,73,70,70') {
    Remove-Item $dest -Force
    throw "Downloaded file is not a RIFF/SF2 file; removed it."
}
$mb = [math]::Round((Get-Item $dest).Length / 1MB)
Write-Host "Done: soundfont.sf2 ($mb MB)."
