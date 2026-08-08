# Publishing to itch.io

Two things can go up, and they are independent.

---

## 1. The web app — ready to upload now

`jazz-lab-web-itch.zip` is finished. It is the complete browser app (Learn, Theory
Lab, Drills, Ear Training, Play-Along, Masters & Standards) in a single 34 KB zip,
with `index.html` at the root, exactly as itch.io wants.

**Upload steps**

1. itch.io → *Upload new project*
2. **Kind of project:** HTML
3. Upload `jazz-lab-web-itch.zip`, then tick **“This file will be played in the browser.”**
4. Viewport: **1024 × 800** is a good fit; tick **Mobile friendly** (the layout is
   responsive) and **Fullscreen button**.
5. Publish.

**Worth knowing**

- It runs entirely client-side and works offline once loaded — nothing to host.
- Audio starts on the first click, which itch.io's own “click to run” button
  satisfies automatically.
- The optional *High-quality sounds* toggle streams samples from a CDN. It fails
  over to the built-in synth if the player is offline or the request is blocked, so
  it degrades quietly rather than breaking.

---

## 2. The Bevy app — build then upload

`build-wasm.ps1` (Windows) and `build-wasm.sh` (macOS/Linux) do the whole job:
compile to `wasm32-unknown-unknown`, run `wasm-bindgen`, optimise with `wasm-opt`
if present, drop in the HTML shell, and zip the result as
`jazz-lab-bevy-itch.zip`.

**One-time setup**

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli    # version must match the wasm-bindgen crate
cargo install wasm-opt            # optional, but roughly halves the download
```

The `wasm-bindgen-cli` version has to match the `wasm-bindgen` crate that Bevy
resolves, or you get a cryptic schema-mismatch error. Check with:

```bash
cargo tree -p wasm-bindgen --depth 0
```

**Build and test locally**

```powershell
.\build-wasm.ps1 -Serve     # then open http://localhost:8080
```

```bash
./build-wasm.sh --serve
```

Test it locally before uploading — `file://` will not work, the module needs to be
served over http.

**Then upload** the same way as above, ticking *played in the browser*.

**Worth knowing**

- **Size.** A Bevy wasm build is large — expect roughly 15–25 MB before `wasm-opt`
  and 8–15 MB after. itch.io's default limit is 1 GB, so it fits comfortably, but
  the first load is not instant. The shell shows a progress bar so players know
  something is happening.
- **Audio and the autoplay policy.** Browsers refuse to start audio before a user
  gesture. The app opens its output stream on the first click, key press or touch,
  and the shell's **Start** button provides that gesture.
- **No threads.** The build is single-threaded, so it needs no `SharedArrayBuffer`
  and no COOP/COEP headers — which matters, because you cannot set headers on
  itch.io.
- **Canvas.** Bevy attaches to `#jazz-canvas` and resizes with its parent, so the
  itch viewport and the fullscreen button both behave.

---

## Which to publish

They are not competing. The web app is complete today and loads instantly; the
Bevy build is the native-quality version and currently covers Play-Along, Theory
and Drills. Publishing the web app first and adding the Bevy build as a second
downloadable (or a separate project) costs nothing and gets something playable up
immediately.
