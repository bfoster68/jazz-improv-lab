# Sounds: the FluidR3 GM SoundFont

The app plays its pitched instruments — piano, guitar, upright bass, tenor sax,
trombone, trumpet and vibraphone — from the **FluidR3 GM** SoundFont, a real
recorded General MIDI set. Only the drums stay synthesised (the inharmonic ride
here is better than most GM kits and costs no samples).

The file lives here as **`soundfont.sf2`**. It is loaded from local disk at
startup, so the native app never downloads anything at runtime. If the file is
missing the app falls back to the built-in synthesis rather than losing audio,
printing which engine it chose:

```
[audio] using soundfont crates/jazz-app/assets/soundfont.sf2
[audio] built-in synth (no soundfont found)
```

## Getting the file

`soundfont.sf2` is ~141 MB, so it is **not** committed to git (it exceeds
GitHub's 100 MB limit and does not belong in history). Fetch it with:

```powershell
crates/jazz-app/assets/fetch-soundfont.ps1
```

That downloads FluidR3 GM (MIT-licensed, by Frank Wen) to `soundfont.sf2`. Any
other General MIDI **`.sf2`** (uncompressed SoundFont2 — not `.sf3`/`.sfz`) works
too; just save it here under that name.

## Where the app looks

At startup it checks, in order:

1. next to the executable — `soundfont.sf2` and `assets/soundfont.sf2`
2. `assets/soundfont.sf2` relative to the working directory
3. `crates/jazz-app/assets/soundfont.sf2` (the dev tree, e.g. under `cargo run`)

To ship a build, copy `soundfont.sf2` next to the packaged executable so (1)
finds it wherever the app is launched from.
