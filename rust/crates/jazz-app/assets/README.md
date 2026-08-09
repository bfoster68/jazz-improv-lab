# Optional: better sounds

Drop a General MIDI SoundFont here named **`soundfont.sf2`** and the app will use
real recorded samples for piano, bass and vibraphone instead of the built-in
synthesis. Drums stay synthesised — the ride cymbal here is better than most GM
drum kits, and it costs no download.

The app prints which engine it chose at startup:

```
[audio] using soundfont assets/soundfont.sf2
[audio] built-in synth (could not open soundfont: ...)
```

A missing or unreadable file is not an error; you simply keep the synth.

## Where to get one

Any GM-compatible `.sf2` works. Well-known free ones include **GeneralUser GS**
(~30 MB, permissive licence) and **FluidR3 GM** (~140 MB, the same set the browser
build streams). Smaller is usually better here: only three programs are used —
Electric Piano 1 (4), Acoustic Bass (32) and Vibraphone (11).

Nothing is bundled, because SoundFont licences vary and a large binary does not
belong in a git repo.
