# TAniCraft

**TAniCraft** is a desktop 2D animation editor written in Rust, inspired by Adobe Animate workflows. The goal is to build a free, open-source alternative for frame-by-frame animation and, eventually, vector animation, with the ability to open and edit Adobe Animate projects.

> **Current status: early prototype (v0.1).** Raster drawing, timeline, layers, onion skin, undo/redo, and the native `.tcraft` format are functional. **FLA/XFL import is not implemented yet.**

## Project Goals

- Build a full-featured 2D animation editor, not merely a file viewer.
- Support editable layers, keyframes, symbol libraries, transformations, and tweens.
- Import Adobe Animate projects (`.xfl` and supported `.fla` variants), preserving their structure as faithfully as possible.
- Maintain an independent `.tcraft` project format and improve compatibility without silent data loss.
- Make the project free and open source under the MIT license, and open to community contributions.

## Adobe Animate Compatibility Roadmap

| Format / Feature | Status | Plan |
|---|---|---|
| `.tcraft` | Working | Open and save native projects |
| PNG | Working | Frame export, image-sequence export, and import to a keyframe |
| JPEG | Working | Import to a keyframe |
| GIF | Working | Export animations (animated GIF) |
| Video (MP4/WebM) | Working | Export via external ffmpeg |
| `.xfl` | Planned | Read XML project structure, timeline, layers, and assets |
| `.fla` | Planned | Investigate container variants and import supported versions |
| SVG | Planned | Import and edit vector objects |
| Symbols, tweens, library | Planned | Reconstruct editable Animate project objects |

**Important:** FLA/XFL projects may contain features that TAniCraft's current document model cannot represent. The importer should report unsupported elements instead of silently dropping them. Compatibility with particular Adobe Animate versions must be verified against test files. The first priority is reliable XFL import, followed by supported FLA variants. FLA/XFL export is not currently available.

## Quick Start

Requires Rust (2024 edition; built with Rust 1.99) and Windows, macOS, or Linux with OpenGL support.

```powershell
cargo run                      # debug build and run
cargo build --release          # optimized build
cargo run --release            # run the optimized build

# Minimal distributable: one exe (and a zip) in dist\
powershell -ExecutionPolicy Bypass -File scripts\package.ps1
```

## Building from Source

### Prerequisites

| Requirement | Notes |
|---|---|
| Rust toolchain | 1.99 or newer, edition 2024 — install from <https://rustup.rs> |
| C/C++ build tools | MSVC Build Tools on Windows, GCC/Clang on Linux, Xcode Command Line Tools on macOS |
| Graphics | OpenGL-capable GPU and driver (rendering goes through `glow`) |
| FFmpeg (optional) | Only for MP4/WebM export; put `ffmpeg` on `PATH` |

No paid services, accounts, API keys, or cloud dependencies are needed to build or run TAniCraft.

### Debug build

```powershell
cargo build
cargo run
```

Produces `target\debug\tanicraft.exe`. Compiles quickly and keeps a console window, so panic messages and warnings stay visible. It is much slower and larger than a release build — use it for development only.

### Release build

```powershell
cargo build --release
```

Produces `target\release\tanicraft.exe`, about **3.3 MB**. The `release` profile in `Cargo.toml` is tuned for size:

| Setting | Value | Effect |
|---|---|---|
| `opt-level` | `"z"` | Optimize for size instead of speed |
| `lto` | `true` | Fat link-time optimization, strips unused code across crates |
| `codegen-units` | `1` | Smaller, better-inlined output; slower builds |
| `panic` | `"abort"` | Drops unwind tables; a panic terminates immediately |
| `strip` | `"symbols"` | No debug symbols in the binary |
| `debug` / `incremental` | `false` | No debug info, no incremental artifacts |

A full release build takes roughly 1–2 minutes on a modern machine, mostly due to fat LTO. Because of `panic = "abort"`, release crashes are silent — reproduce problems with a debug build when you need a message or backtrace.

### Packaging a distributable

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package.ps1          # exe + zip
powershell -ExecutionPolicy Bypass -File scripts\package.ps1 -NoZip   # exe only
```

The script builds the release profile, copies the executable to `dist\TAniCraft.exe` plus `LICENSE.txt` (the MIT notice must travel with the binary), and — unless `-NoZip` is passed — creates `dist\TAniCraft-win64.zip` (~1.7 MB) containing both files. It can be started from any directory, because it resolves the repository root from its own location. `-ExecutionPolicy Bypass` is needed only when PowerShell blocks local scripts, and applies to that single invocation.

### Where the files are

| Path | What it is | Ship it? |
|---|---|---|
| `dist\TAniCraft.exe` | Packaged release build | **Yes** |
| `dist\TAniCraft-win64.zip` | The same exe, zipped, with `LICENSE.txt` | **Yes** |
| `dist\LICENSE.txt` | MIT notice shipped with the build | **Yes** |
| `target\release\tanicraft.exe` | The same binary before packaging | Yes, as a single file |
| `target\` | Cargo build cache — dependency rlibs, build scripts, PDB, incremental data (0.4–1 GB) | No |

`target\` is a cache, not part of the product, and it is ignored by git. `cargo clean` (or deleting the folder) frees the space; the next build recreates it. Removing `target\` never affects `dist\` or your `.tcraft` projects.

Release builds use the Windows GUI subsystem, so double-clicking `TAniCraft.exe` opens only the editor — no extra console window.

### Tests and checks

```powershell
cargo test        # document model, PNG/GIF round trips, colour sampling
cargo check       # fast type check without linking
cargo clippy      # optional lints
```

A successful build does not replace manual UI verification — see [`docs/DEBUGGING.md`](docs/DEBUGGING.md) and `scripts/screenshot.ps1` for visual checks on Windows.

## Implemented Features

| Area | Features |
|---|---|
| Document | New documents, dimensions, FPS, background, properties, resize without scaling |
| Drawing | Brush with size, hardness, opacity, and smoothing controls; eraser; eyedropper with `Alt`+click sampling |
| Canvas | Zoom, pan, Fit in Window, 100% view |
| Timeline | Layers, keyframes, blank keyframes, hold and empty frames; F5/F6/F7 operations |
| Playback | Play/Stop, frame stepping, looping, document FPS |
| Layers | Add, duplicate, delete, reorder, show/hide, lock, rename |
| Onion skin | Preview neighboring keyframes with fading |
| History | Undo/redo for pixel and structural edits; 40-step limit |
| Files | Open/save `.tcraft`; import PNG/JPEG to a keyframe; export current frame to PNG, PNG image sequence, animated GIF, and MP4/WebM video (via ffmpeg) |

## Keyboard Shortcuts

| Action | Shortcut |
|---|---|
| Brush / Eraser / Eyedropper / Hand / Zoom | `B` / `E` / `I` / `H` / `Z` |
| Brush size | `[` / `]` |
| Sample colour / quick sample | Eyedropper `I` / `Alt`+click with Brush or Eraser |
| Onion skin | `Alt+O` |
| Extend frame | `F5` |
| Insert keyframe with copied artwork | `F6` |
| Insert blank keyframe | `F7` |
| Remove frame / clear keyframe | `Shift+F5` / `Shift+F6` |
| Play / Stop | `Enter` |
| Previous / next frame | `←` / `→` |
| First / last frame | `Home` / `End` |
| Clear frame artwork | `Delete` |
| Undo / Redo | `Ctrl+Z` / `Ctrl+Shift+Z` or `Ctrl+Y` |
| New / Open / Save / Save As | `Ctrl+N` / `Ctrl+O` / `Ctrl+S` / `Ctrl+Shift+S` |
| Export PNG | `Ctrl+E` |
| Fit / 100% / zoom | `Ctrl+0` / `View → 100%` / `Ctrl++` / `Ctrl+-` |
| Pan canvas | `Space` + drag, middle mouse button, Hand tool |
| Zoom toward cursor | `Ctrl` + mouse wheel |
| Scroll timeline | Mouse wheel; `Shift` for horizontal scroll, `Alt` to scrub |

## Architecture

Current stack: **Rust 2024, egui/eframe 0.33.x, glow/OpenGL**, `serde`, `serde_json`, `png`, and `rfd`. Application UI is in English; existing internal project documentation is in Russian.

```text
src/
  main.rs       application entry point
  app.rs        state, menus, commands, history, playback
  canvas.rs     canvas, drawing, zoom, pan, onion skin
  timeline.rs   layers, frames, playhead, timeline
  dialogs.rs    dialogs and properties
  doc.rs        document model, Bitmap, .tcraft, PNG
  tools.rs      tools and settings
  theme.rs      palette and styling
  icons.rs      vector icons
  ui.rs         shared UI components
docs/           architecture, formats, tasks, decisions, roadmap
scripts/        supporting verification utilities
```

Adobe Animate import parsers should remain separate from the internal document model. Decide the precise layout of new modules after reviewing the existing code. Do not introduce incompatible `.tcraft` format changes without versioning and migration support.

## AI-Assisted Development

TAniCraft can be developed using **OpenCode with a free DeepSeek model**; no paid AI service is required. Before editing, an agent should read [`AGENTS.md`](AGENTS.md), inspect the relevant source files, and consult related documentation. Keep changes small and verify the build and behavior after each task.

```powershell
cargo build
cargo test
cargo run
```

A successful build does not replace manual UI verification. On Windows, `scripts/screenshot.ps1` is available for visual checks.

## Contributing

Contributions are welcome in areas such as bug fixes, legally shareable XFL/FLA test fixtures, format research, import tests, drawing tools, documentation, and UI improvements. Do not commit proprietary Adobe code, assets, or project files without the appropriate rights.

Before submitting changes, verify the build, tests, undo/redo behavior, and `.tcraft` save/load round trips. Update compatibility documentation whenever the project format changes.

By submitting a pull request you agree that your contribution is licensed under the same [MIT License](LICENSE) as the rest of the project.

## Documentation

- [`AGENTS.md`](AGENTS.md) — instructions for AI agents and contributors.
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — architecture.
- [`docs/UI_GUIDE.md`](docs/UI_GUIDE.md) — UI standards.
- [`docs/ROADMAP.md`](docs/ROADMAP.md) — development roadmap.
- [`docs/TASKS.md`](docs/TASKS.md) — tasks and known issues.
- [`docs/DECISIONS.md`](docs/DECISIONS.md) — architecture decisions.
- [`docs/DEBUGGING.md`](docs/DEBUGGING.md) — debugging and visual verification.
- [`docs/FILE_FORMAT.md`](docs/FILE_FORMAT.md) — `.tcraft` format specification.

## License

TAniCraft is **free and open source**, released under the [MIT License](LICENSE).
Copyright (c) 2026 TAniCraft contributors.

What that means in practice:

- **Free to use.** Download, run, and use it for personal, educational, or commercial work. No paid tiers, subscriptions, trials, accounts, telemetry, or activation.
- **Free to modify.** Change the code, fork the project, and build your own version. You are not required to publish your changes.
- **Free to share.** Redistribute the original or a modified build, including commercially, as long as the copyright notice and the MIT permission notice are kept — `scripts\package.ps1` puts that notice next to the executable as `LICENSE.txt`.
- **No warranty.** The software is provided "as is", without warranty of any kind; the authors are not liable for damages or data loss. Keep backups of your projects.
- **Your artwork stays yours.** The MIT license covers the editor's source code. Animation, images, and `.tcraft` files you create with TAniCraft belong to you, and the license claims no rights over them.
- **Contributions** are accepted under the same MIT terms, so the project stays free for everyone.

TAniCraft bundles no Adobe code or assets and requires no Adobe runtime, proprietary SDK, or cloud service. Building and running it costs nothing beyond your own hardware.

*TAniCraft is an independent project and is not affiliated with Adobe. Adobe Animate is a trademark of its respective owner.*
