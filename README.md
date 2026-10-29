# TAniCraft

**TAniCraft** is a desktop 2D animation editor written in Rust, inspired by Adobe Animate workflows. The goal is to build a free, open-source alternative for frame-by-frame animation and, eventually, vector animation, with the ability to open and edit Adobe Animate projects.

> **Current status: early prototype (v0.1).** Raster drawing, timeline, layers, onion skin, undo/redo, and the native `.tcraft` format are functional. **FLA/XFL import is not implemented yet.**

## Project Goals

- Build a full-featured 2D animation editor, not merely a file viewer.
- Support editable layers, keyframes, symbol libraries, transformations, and tweens.
- Import Adobe Animate projects (`.xfl` and supported `.fla` variants), preserving their structure as faithfully as possible.
- Maintain an independent `.tcraft` project format and improve compatibility without silent data loss.
- Make the project free and open to community contributions once a license is selected and published.

## Adobe Animate Compatibility Roadmap

| Format / Feature | Status | Plan |
|---|---|---|
| `.tcraft` | Working | Open and save native projects |
| PNG | Partially working | Current-frame export; expand import/export |
| `.xfl` | Planned | Read XML project structure, timeline, layers, and assets |
| `.fla` | Planned | Investigate container variants and import supported versions |
| SVG | Planned | Import and edit vector objects |
| GIF / image sequences | Planned | Import and export animations |
| Symbols, tweens, library | Planned | Reconstruct editable Animate project objects |

**Important:** FLA/XFL projects may contain features that TAniCraft's current document model cannot represent. The importer should report unsupported elements instead of silently dropping them. Compatibility with particular Adobe Animate versions must be verified against test files. The first priority is reliable XFL import, followed by supported FLA variants. FLA/XFL export is not currently available.

## Quick Start

Requires Rust (2024 edition; the project was previously built with Rust 1.99) and Windows, macOS, or Linux with OpenGL support.

```powershell
cargo run
cargo build --release
cargo run --release
```

On Windows, the compiled executables are located at `target\debug\tanicraft.exe` and `target\release\tanicraft.exe`.

## Implemented Features

| Area | Features |
|---|---|
| Document | New documents, dimensions, FPS, background, properties, resize without scaling |
| Drawing | Brush with size, hardness, opacity, and smoothing controls; eraser |
| Canvas | Zoom, pan, Fit in Window, 100% view |
| Timeline | Layers, keyframes, blank keyframes, hold and empty frames; F5/F6/F7 operations |
| Playback | Play/Stop, frame stepping, looping, document FPS |
| Layers | Add, duplicate, delete, reorder, show/hide, lock, rename |
| Onion skin | Preview neighboring keyframes with fading |
| History | Undo/redo for pixel and structural edits; 40-step limit |
| Files | Open/save `.tcraft`; export current frame to PNG |

## Keyboard Shortcuts

| Action | Shortcut |
|---|---|
| Brush / Eraser / Hand / Zoom | `B` / `E` / `H` / `Z` |
| Brush size | `[` / `]` |
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

**A license has not been selected yet.** Publishing source code without a license does not automatically grant permission to copy, modify, or redistribute it. To make TAniCraft an open-source project, select and add a license (for example, MIT, Apache-2.0, or GPL-3.0) and clarify contribution terms.

*TAniCraft is an independent project and is not affiliated with Adobe. Adobe Animate is a trademark of its respective owner.*
