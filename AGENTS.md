# AGENTS.md — TAniCraft development instructions

Read this file before editing code. This project is an open-source desktop animation editor inspired by Adobe Animate. Work incrementally with OpenCode and a free DeepSeek model; never require paid APIs, hosted inference, or proprietary development services.

## Project baseline (verify against repository before changes)
- Rust 2024; egui/eframe 0.33.x; glow rendering; rfd dialogs; serde/serde_json; png.
- Raster frames currently use premultiplied RGBA8; convert to straight alpha for PNG.
- Bitmap is copy-on-write `Arc<Vec<u8>>`; all edits go through `Bitmap::data_mut()` and increment `rev`.
- Undo: structural snapshots or before/after pixel snapshots, 40-entry limit. Every mutation must participate in undo.
- Texture cache keyed by layer/keyframe and revision; invalidate epoch after structural edits and undo/redo.
- Timeline is manually rendered for synchronized scrolling; do not replace with ScrollArea.
- Preserve `.tcraft` compatibility; document versioned changes.
- UI text English; technical comments and documentation Russian.
- Consult existing `docs/DECISIONS.md`, `docs/TASKS.md`, `docs/FILE_FORMAT.md` if present. Never assume missing files exist.

## Main product goal: Adobe Animate project interoperability
The editor must ultimately open and edit modern Adobe Animate `.fla` files and uncompressed `.xfl` projects, retaining as much editable structure as possible. Prioritize *faithful project reconstruction*, not flattening frames into images. Support must be implemented in stages and tested with real, legally obtained sample files.

### Format handling principles
1. Inspect file signature, not just extension. Modern FLA may contain a ZIP-based XFL project; do not assume every FLA is ZIP. Older binary FLA is a separate compatibility task.
2. Uncompressed XFL is a project directory, typically containing `DOMDocument.xml`, `LIBRARY/`, `bin/`, and a `.xfl` marker. Resolve relative paths safely.
3. Read project metadata, document dimensions, frame rate, background, scenes, layer order, layer types, frame spans, keyframes, symbols, instances, transformations, and assets as available.
4. Preserve vector shapes as editable vector geometry when implemented. Do not silently rasterize them and claim full compatibility.
5. Preserve unsupported elements as opaque source data where practical, and warn users about any feature that cannot be rendered or round-tripped.
6. Opening must never overwrite original FLA/XFL. Default to Save As `.tcraft` until export has independently passed round-trip tests.
7. Treat input archives/XML as untrusted: enforce file size and decompression limits; reject path traversal, unsafe symlinks, XML external entities and malformed data; avoid crashes.
8. No Adobe runtime or proprietary SDK dependency. Respect licensing of any sample files and third-party code.

### Implementation sequence
**Phase 0 — research and fixtures:** Inspect the repository and existing document model. Add `docs/ADOBE_COMPAT.md` with a compatibility matrix, sample inventory, format observations and unsupported features. Collect small fixtures: blank project, 2-layer raster animation, vector shape, symbol instance, tween, nested symbol. No unsupported compatibility claims.

**Phase 1 — read-only XFL parser:** Add an isolated importer module (suggest `src/import/xfl.rs`, adjusted to actual architecture). Parse `DOMDocument.xml`, layers, frame indices/durations, basic metadata, library references and supported bitmap assets. Return a structured intermediate representation with warnings, not a partially mutated live document.

**Phase 2 — document model expansion:** Design editable shapes, symbols, instances, and animation properties independently of Adobe XML. Use explicit conversion to internal document types; preserve old `.tcraft` files and undo behavior. Add tests for timeline semantics and frame spans.

**Phase 3 — FLA import:** Detect ZIP-based FLA and reuse the XFL parser after safe in-memory/archive access. Unsupported legacy binary FLA must report a clear message; do not pretend it is supported.

**Phase 4 — rendering and editing:** Implement vector paths/fills/strokes, transforms, symbols, nested timelines, motion/classic/shape tween features incrementally. Each feature requires editable representation and rendering tests.

**Phase 5 — export:** Add XFL export only after import is stable. Add FLA export after round-trip fixtures pass. Preserve unknown metadata where feasible and warn about loss. Test export in Adobe Animate where legally available; otherwise state that verification is incomplete.

### Required compatibility tests
- XFL parse -> intermediate representation: expected scenes, layers, frames, FPS, dimensions and assets.
- FLA archive -> same representation as corresponding uncompressed XFL.
- Frame duration and hold/keyframe semantics match source.
- Nested symbols and transform matrices retain their relationships.
- Unsupported elements generate explicit diagnostics.
- Malicious ZIP entries, oversized decompression, broken XML and missing assets fail safely.
- Open -> save as `.tcraft` -> reopen preserves supported structure.
- Future export -> reimport compares semantic structure, not byte equality.

## Efficient agent workflow for free DeepSeek
1. For each request inspect only relevant files; do not dump the entire repository into context.
2. State a short plan of at most five steps, then implement ONE bounded feature or bug fix.
3. Prefer targeted edits and small modules; avoid wholesale rewrites, speculative dependencies and unrelated refactors.
4. After each edit run `cargo fmt --check`, `cargo check`, and relevant `cargo test` (or explain why unavailable). Use `cargo build` when feasible.
5. For UI changes launch the app and inspect screenshots if screenshot tooling is available; never claim visual verification without it.
6. Report changed files, tests actually executed, remaining issues, and one concrete next step.
7. If context is running low, update `docs/TASKS.md` with exact progress and pending work rather than inventing a completion claim.
8. Never introduce paid APIs, cloud-only services or secret credentials.

## Architecture guardrails
- Keep UI, document model, file import/export and rendering separated.
- Parse external formats into a validated intermediate representation before conversion.
- Maintain bitmap alpha invariants, revision tracking, undo/redo and texture invalidation.
- Keep keyboard shortcuts and UI behavior consistent with existing project conventions.
- Do not silently modify original user files.
- Avoid adding heavyweight engines (bevy/wgpu) or upgrading egui without a separately approved task.

## Definition of done
A task is done only when code compiles, relevant tests pass, existing raster editing/undo remains functional, file safety checks are covered, compatibility claims match fixture evidence, and documentation reflects new support and limitations. If any check cannot be performed, report it explicitly.
