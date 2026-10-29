//! Application state, menus, panels and commands.

use std::collections::HashMap;
use std::path::PathBuf;

use egui::{Color32, Context, Key, Rect, Vec2};

use crate::canvas::Stroke;
use crate::doc::{Bitmap, Document, FrameKind};
use crate::icons::Icon;
use crate::theme;
use crate::tools::{BrushSettings, EraserSettings, OnionSettings, Tool};
use crate::ui;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cmd {
    New,
    Open,
    Save,
    SaveAs,
    ExportPng,
    Exit,
    Undo,
    Redo,
    Cut,
    Copy,
    ClearFrame,
    FitView,
    ZoomIn,
    ZoomOut,
    ActualSize,
    InsertFrame,
    InsertKey,
    InsertBlankKey,
    RemoveFrame,
    ClearKey,
    NewLayer,
    DuplicateLayer,
    DeleteLayer,
    MoveLayerUp,
    MoveLayerDown,
    Play,
    LoopToggle,
    NextFrame,
    PrevFrame,
    FirstFrame,
    LastFrame,
    OnionToggle,
    DocProps,
    LayerProps,
    About,
}

pub struct TexEntry {
    pub handle: egui::TextureHandle,
    pub rev: u64,
    pub epoch: u64,
}

pub enum UndoAction {
    Pixels {
        layer_uid: u64,
        frame: usize,
        before: Bitmap,
        after: Bitmap,
    },
    Doc {
        before: Box<Document>,
        after: Box<Document>,
    },
}

#[derive(Default)]
pub struct ViewState {
    /// Offset of the document centre from the viewport centre, in screen px.
    pub offset: Vec2,
    pub zoom: f32,
    pub fitted: bool,
}

#[derive(Clone)]
pub struct DocSetup {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub bg: Color32,
}

impl DocSetup {
    pub fn from_doc(doc: &Document) -> Self {
        Self {
            name: doc.name.clone(),
            width: doc.width,
            height: doc.height,
            fps: doc.fps,
            bg: Color32::from_rgba_unmultiplied(doc.bg[0], doc.bg[1], doc.bg[2], doc.bg[3]),
        }
    }
}

pub struct App {
    pub doc: Document,
    pub path: Option<PathBuf>,
    pub dirty: bool,

    pub tool: Tool,
    pub brush: BrushSettings,
    pub eraser: EraserSettings,
    pub onion: OnionSettings,

    pub view: ViewState,
    pub stage_rect: Rect,

    pub current_frame: usize,
    pub sel_layer: usize,

    pub playing: bool,
    pub looping: bool,
    pub play_accum: f32,

    pub tl_scroll_x: f32,
    pub tl_scroll_y: f32,

    pub history: Vec<UndoAction>,
    pub redo_stack: Vec<UndoAction>,

    pub textures: HashMap<(u64, usize), TexEntry>,
    pub tex_epoch: u64,
    pub blank_cache: HashMap<(u64, usize), (u64, bool)>,

    pub stroke: Option<Stroke>,

    pub status: String,

    pub new_doc_open: bool,
    pub new_doc: DocSetup,
    pub doc_props: Option<DocSetup>,
    pub layer_props: Option<usize>,
    pub layer_name_buf: String,
    pub about_open: bool,
    pub unsaved_open: bool,
    pub pending: Option<Cmd>,

    pub menu_target: Option<(usize, usize)>,
    pub title_cache: String,
    pub untitled_counter: u32,
    pub show_guides: bool,
    pub show_fill_picker: bool,
    pub want_close: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        let doc = Document::new("Untitled-1", 1280, 720, 24.0, [255, 255, 255, 255]);
        let mut textures = HashMap::new();
        textures.reserve(16);
        Self {
            doc,
            path: None,
            dirty: false,
            tool: Tool::Brush,
            brush: BrushSettings::default(),
            eraser: EraserSettings::default(),
            onion: OnionSettings::default(),
            view: ViewState { offset: Vec2::ZERO, zoom: 1.0, fitted: false },
            stage_rect: Rect::from_min_size(egui::Pos2::ZERO, Vec2::splat(100.0)),
            current_frame: 0,
            sel_layer: 0,
            playing: false,
            looping: true,
            play_accum: 0.0,
            tl_scroll_x: 0.0,
            tl_scroll_y: 0.0,
            history: Vec::new(),
            redo_stack: Vec::new(),
            textures,
            tex_epoch: 0,
            blank_cache: HashMap::new(),
            stroke: None,
            status: "Brush (B) — draw on a keyframe. F6 adds a keyframe, F5 extends the timeline.".into(),
            new_doc_open: false,
            new_doc: DocSetup {
                name: "Untitled-1".into(),
                width: 1280,
                height: 720,
                fps: 24.0,
                bg: Color32::WHITE,
            },
            doc_props: None,
            layer_props: None,
            layer_name_buf: String::new(),
            about_open: false,
            unsaved_open: false,
            pending: None,
            menu_target: None,
            title_cache: String::new(),
            untitled_counter: 1,
            show_guides: true,
            show_fill_picker: false,
            want_close: false,
        }
    }

    // -- small helpers ------------------------------------------------------

    pub fn notify(&mut self, msg: impl Into<String>) {
        self.status = msg.into();
    }

    pub fn frame_count(&self) -> usize {
        self.doc.frame_count()
    }

    pub fn current_layer(&self) -> Option<&crate::doc::Layer> {
        self.doc.layers.get(self.sel_layer)
    }

    pub fn layer_kind_at_current(&self, layer: usize) -> FrameKind {
        self.doc
            .layers
            .get(layer)
            .map(|l| l.kind_at(self.current_frame))
            .unwrap_or(FrameKind::Empty)
    }

    pub fn clamp_selection(&mut self) {
        if self.doc.layers.is_empty() {
            return;
        }
        if self.sel_layer >= self.doc.layers.len() {
            self.sel_layer = self.doc.layers.len() - 1;
        }
        let fc = self.frame_count();
        if self.current_frame >= fc {
            self.current_frame = fc - 1;
        }
    }

    pub fn set_frame(&mut self, f: usize) {
        let max = self.frame_count() + 48;
        self.current_frame = f.min(max);
    }

    // -- undo ---------------------------------------------------------------

    pub fn commit(&mut self, before: Document) {
        let after = self.doc.clone();
        self.history.push(UndoAction::Doc { before: Box::new(before), after: Box::new(after) });
        if self.history.len() > 40 {
            self.history.remove(0);
        }
        self.redo_stack.clear();
        self.dirty = true;
        self.tex_epoch += 1;
        self.clamp_selection();
    }

    pub fn push_pixels(&mut self, layer_uid: u64, frame: usize, before: Bitmap, after: Bitmap) {
        self.history.push(UndoAction::Pixels { layer_uid, frame, before, after });
        if self.history.len() > 40 {
            self.history.remove(0);
        }
        self.redo_stack.clear();
        self.dirty = true;
    }

    pub fn undo(&mut self) {
        if let Some(a) = self.history.pop() {
            self.apply_undo(&a, true);
            self.redo_stack.push(a);
            self.dirty = true;
            self.tex_epoch += 1;
            self.clamp_selection();
            self.notify("Undo");
        }
    }

    pub fn redo(&mut self) {
        if let Some(a) = self.redo_stack.pop() {
            self.apply_undo(&a, false);
            self.history.push(a);
            self.dirty = true;
            self.tex_epoch += 1;
            self.clamp_selection();
            self.notify("Redo");
        }
    }

    fn apply_undo(&mut self, a: &UndoAction, undo: bool) {
        match a {
            UndoAction::Pixels { layer_uid, frame, before, after } => {
                let bmp = if undo { before } else { after };
                if let Some(li) = self.doc.layer_index(*layer_uid) {
                    if let Some(f) = self.doc.layers[li].frames.get_mut(*frame) {
                        if f.kind == FrameKind::Key {
                            f.bitmap = Some(bmp.clone());
                        }
                    }
                }
            }
            UndoAction::Doc { before, after } => {
                let d = if undo { before } else { after };
                self.doc = (**d).clone();
            }
        }
    }

    // -- textures -----------------------------------------------------------

    pub fn texture_for(
        &mut self,
        ctx: &Context,
        uid: u64,
        frame: usize,
        bmp: &Bitmap,
    ) -> egui::TextureHandle {
        let key = (uid, frame);
        if let Some(e) = self.textures.get(&key) {
            if e.rev == bmp.rev && e.epoch == self.tex_epoch {
                return e.handle.clone();
            }
        }
        let img = bmp.to_color_image();
        if let Some(e) = self.textures.get_mut(&key) {
            e.handle.set(img, egui::TextureOptions::LINEAR);
            e.rev = bmp.rev;
            e.epoch = self.tex_epoch;
            e.handle.clone()
        } else {
            let handle = ctx.load_texture(
                format!("frame-{uid}-{frame}"),
                img,
                egui::TextureOptions::LINEAR,
            );
            self.textures.insert(
                key,
                TexEntry { handle: handle.clone(), rev: bmp.rev, epoch: self.tex_epoch },
            );
            handle
        }
    }

    pub fn patch_texture(&mut self, uid: u64, frame: usize, bmp: &Bitmap, r: crate::doc::DirtyRect) {
        if let Some(e) = self.textures.get_mut(&(uid, frame)) {
            let img = bmp.region_image(r);
            e.handle.set_partial([r.x as usize, r.y as usize], img, egui::TextureOptions::LINEAR);
            e.rev = bmp.rev;
            e.epoch = self.tex_epoch;
        }
    }

    // -- document lifecycle -------------------------------------------------

    pub fn fit_view(&mut self) {
        let vp = self.stage_rect;
        if vp.width() < 10.0 || vp.height() < 10.0 {
            self.view.fitted = false;
            return;
        }
        let dw = self.doc.width as f32;
        let dh = self.doc.height as f32;
        let z = ((vp.width() - 80.0) / dw).min((vp.height() - 80.0) / dh);
        self.view.zoom = z.clamp(0.02, 8.0);
        self.view.offset = Vec2::ZERO;
        self.view.fitted = true;
    }

    pub fn new_document(&mut self, setup: &DocSetup) {
        let bg = setup.bg.to_srgba_unmultiplied();
        self.doc = Document::new(
            setup.name.trim().to_string(),
            setup.width.max(1),
            setup.height.max(1),
            setup.fps.clamp(1.0, 120.0),
            bg,
        );
        self.path = None;
        self.dirty = false;
        self.textures.clear();
        self.tex_epoch += 1;
        self.history.clear();
        self.redo_stack.clear();
        self.current_frame = 0;
        self.sel_layer = 0;
        self.playing = false;
        self.tl_scroll_x = 0.0;
        self.tl_scroll_y = 0.0;
        self.fit_view();
    }

    pub fn save(&mut self, save_as: bool) -> bool {
        let mut path = if save_as { None } else { self.path.clone() };
        if path.is_none() {
            let mut dlg = rfd::FileDialog::new()
                .add_filter("TAniCraft project", &["tcraft"])
                .set_file_name(format!("{}.tcraft", self.doc.name));
            if let Some(dir) = self.path.as_ref().and_then(|p| p.parent()) {
                dlg = dlg.set_directory(dir);
            }
            path = dlg.save_file();
        }
        let Some(path) = path else {
            self.notify("Save cancelled");
            return false;
        };
        let path = if path.extension().is_none() {
            path.with_extension("tcraft")
        } else {
            path
        };
        match crate::doc::save_document(&self.doc, &path) {
            Ok(()) => {
                self.doc.name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| self.doc.name.clone());
                self.path = Some(path.clone());
                self.dirty = false;
                self.notify(format!("Saved {}", path.display()));
                true
            }
            Err(e) => {
                self.notify(format!("Save failed: {e}"));
                false
            }
        }
    }

    fn open_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("TAniCraft project", &["tcraft"])
            .pick_file()
        else {
            return;
        };
        match crate::doc::load_document(&path) {
            Ok(doc) => {
                self.doc = doc;
                self.path = Some(path.clone());
                self.dirty = false;
                self.textures.clear();
                self.tex_epoch += 1;
                self.history.clear();
                self.redo_stack.clear();
                self.current_frame = 0;
                self.sel_layer = self.doc.layers.len().saturating_sub(1);
                self.playing = false;
                self.fit_view();
                self.notify(format!("Opened {}", path.display()));
            }
            Err(e) => self.notify(format!("Open failed: {e}")),
        }
    }

    fn export_png(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name(format!("{} - frame {}.png", self.doc.name, self.current_frame + 1))
            .save_file()
        else {
            return;
        };
        match crate::doc::export_frame_png(&self.doc, self.current_frame, &path) {
            Ok(()) => self.notify(format!("Exported {}", path.display())),
            Err(e) => self.notify(format!("Export failed: {e}")),
        }
    }

    // -- commands -----------------------------------------------------------

    pub fn request(&mut self, cmd: Cmd) {
        if self.dirty && matches!(cmd, Cmd::New | Cmd::Open | Cmd::Exit) {
            self.pending = Some(cmd);
            self.unsaved_open = true;
            return;
        }
        self.exec(cmd);
    }

    pub fn exec(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::New => {
                self.new_doc = DocSetup {
                    name: format!("Untitled-{}", self.untitled_counter + 1),
                    width: self.doc.width,
                    height: self.doc.height,
                    fps: self.doc.fps,
                    bg: Color32::from_rgba_unmultiplied(
                        self.doc.bg[0], self.doc.bg[1], self.doc.bg[2], self.doc.bg[3],
                    ),
                };
                self.new_doc_open = true;
            }
            Cmd::Open => self.open_file(),
            Cmd::Save => {
                self.save(false);
            }
            Cmd::SaveAs => {
                self.save(true);
            }
            Cmd::ExportPng => self.export_png(),
            Cmd::Exit => {
                self.want_close = true;
            }
            Cmd::Undo => self.undo(),
            Cmd::Redo => self.redo(),
            Cmd::FitView => self.fit_view(),
            Cmd::ZoomIn => {
                self.view.zoom = (self.view.zoom * 1.25).min(40.0);
            }
            Cmd::ZoomOut => {
                self.view.zoom = (self.view.zoom / 1.25).max(0.02);
            }
            Cmd::ActualSize => {
                self.view.zoom = 1.0;
                self.view.offset = Vec2::ZERO;
            }
            Cmd::Play => {
                self.playing = !self.playing;
                self.play_accum = 0.0;
            }
            Cmd::LoopToggle => self.looping = !self.looping,
            Cmd::NextFrame => self.set_frame(self.current_frame + 1),
            Cmd::PrevFrame => self.set_frame(self.current_frame.saturating_sub(1)),
            Cmd::FirstFrame => self.set_frame(0),
            Cmd::LastFrame => self.set_frame(self.frame_count().saturating_sub(1)),
            Cmd::OnionToggle => self.onion.enabled = !self.onion.enabled,
            Cmd::InsertFrame => {
                let before = self.doc.clone();
                self.doc.insert_frame(self.sel_layer, self.current_frame);
                self.commit(before);
                self.notify("Inserted frame (F5)");
            }
            Cmd::InsertKey | Cmd::InsertBlankKey => {
                let before = self.doc.clone();
                let blank = cmd == Cmd::InsertBlankKey;
                self.doc.insert_keyframe(self.sel_layer, self.current_frame, blank);
                self.commit(before);
                self.notify(if blank { "Inserted blank keyframe (F7)" } else { "Inserted keyframe (F6)" });
            }
            Cmd::RemoveFrame => {
                let before = self.doc.clone();
                self.doc.remove_frame(self.sel_layer, self.current_frame);
                self.commit(before);
                self.notify("Removed frame (Shift+F5)");
            }
            Cmd::ClearKey => {
                let before = self.doc.clone();
                self.doc.clear_keyframe(self.sel_layer, self.current_frame);
                self.commit(before);
                self.notify("Cleared keyframe (Shift+F6)");
            }
            Cmd::ClearFrame => {
                let li = self.sel_layer;
                let fi = self.current_frame;
                if let Some(key) = self.doc.layers.get(li).and_then(|l| l.effective_key(fi)) {
                    let before = self
                        .doc
                        .layers[li]
                        .frames[key]
                        .bitmap
                        .clone()
                        .unwrap_or_else(|| Bitmap::new(self.doc.width as usize, self.doc.height as usize));
                    if let Some(bmp) = self.doc.layers[li].frames[key].key_bitmap_mut() {
                        bmp.clear();
                    }
                    let after = self.doc.layers[li].frames[key].bitmap.clone().unwrap_or_else(|| before.clone());
                    self.push_pixels(self.doc.layers[li].uid, key, before, after);
                    self.tex_epoch += 1;
                    self.notify("Cleared drawing");
                }
            }
            Cmd::NewLayer => {
                let before = self.doc.clone();
                let idx = self.doc.add_layer(true);
                self.sel_layer = idx;
                self.commit(before);
                self.notify("New layer");
            }
            Cmd::DuplicateLayer => {
                let before = self.doc.clone();
                let idx = self.sel_layer.min(self.doc.layers.len().saturating_sub(1));
                let mut copy = self.doc.layers[idx].clone();
                copy.uid = 0; // fixed up below
                let uid = self.doc_fresh_uid();
                copy.uid = uid;
                copy.name = format!("{} copy", copy.name);
                self.doc.layers.insert(idx + 1, copy);
                self.sel_layer = idx + 1;
                self.commit(before);
                self.notify("Duplicated layer");
            }
            Cmd::DeleteLayer => {
                if self.doc.layers.len() <= 1 {
                    self.notify("A document needs at least one layer");
                } else {
                    let before = self.doc.clone();
                    let idx = self.sel_layer;
                    self.doc.remove_layer(idx);
                    self.sel_layer = idx.saturating_sub(1);
                    self.commit(before);
                    self.notify("Deleted layer");
                }
            }
            Cmd::MoveLayerUp => {
                if self.sel_layer + 1 < self.doc.layers.len() {
                    let before = self.doc.clone();
                    self.doc.move_layer(self.sel_layer, self.sel_layer + 1);
                    self.sel_layer += 1;
                    self.commit(before);
                }
            }
            Cmd::MoveLayerDown => {
                if self.sel_layer > 0 {
                    let before = self.doc.clone();
                    self.doc.move_layer(self.sel_layer, self.sel_layer - 1);
                    self.sel_layer -= 1;
                    self.commit(before);
                }
            }
            Cmd::DocProps => {
                self.doc_props = Some(DocSetup::from_doc(&self.doc));
            }
            Cmd::LayerProps => {
                self.layer_name_buf = self
                    .current_layer()
                    .map(|l| l.name.clone())
                    .unwrap_or_default();
                self.layer_props = Some(self.sel_layer);
            }
            Cmd::About => self.about_open = true,
            Cmd::Cut | Cmd::Copy => self.notify("Not available yet"),
        }
    }

    /// Grabs a fresh layer uid without touching the doc's public API.
    fn doc_fresh_uid(&mut self) -> u64 {
        let mut uid = 1;
        for l in &self.doc.layers {
            uid = uid.max(l.uid + 1);
        }
        uid
    }

    // -- keyboard -----------------------------------------------------------

    fn handle_shortcuts(&mut self, ctx: &Context) {
        if ctx.wants_keyboard_input() {
            return;
        }
        let mut cmds: Vec<Cmd> = Vec::new();
        let mut tool: Option<Tool> = None;
        let mut brush_delta = 0.0f32;
        let mut seek_key: Option<usize> = None;

        ctx.input(|i| {
            let cmd = i.modifiers.command;
            let shift = i.modifiers.shift;
            if cmd {
                if i.key_pressed(Key::N) {
                    cmds.push(Cmd::New);
                }
                if i.key_pressed(Key::O) {
                    cmds.push(Cmd::Open);
                }
                if i.key_pressed(Key::S) {
                    cmds.push(if shift { Cmd::SaveAs } else { Cmd::Save });
                }
                if i.key_pressed(Key::E) {
                    cmds.push(Cmd::ExportPng);
                }
                if i.key_pressed(Key::Z) {
                    cmds.push(if shift { Cmd::Redo } else { Cmd::Undo });
                }
                if i.key_pressed(Key::Y) {
                    cmds.push(Cmd::Redo);
                }
                if i.key_pressed(Key::Num0) {
                    cmds.push(Cmd::FitView);
                }
                if i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals) {
                    cmds.push(Cmd::ZoomIn);
                }
                if i.key_pressed(Key::Minus) {
                    cmds.push(Cmd::ZoomOut);
                }
            } else {
                if i.key_pressed(Key::B) {
                    tool = Some(Tool::Brush);
                }
                if i.key_pressed(Key::E) {
                    tool = Some(Tool::Eraser);
                }
                if i.key_pressed(Key::H) {
                    tool = Some(Tool::Hand);
                }
                if i.key_pressed(Key::Z) {
                    tool = Some(Tool::Zoom);
                }
                if i.key_pressed(Key::F5) {
                    cmds.push(if shift { Cmd::RemoveFrame } else { Cmd::InsertFrame });
                }
                if i.key_pressed(Key::F6) {
                    cmds.push(if shift { Cmd::ClearKey } else { Cmd::InsertKey });
                }
                if i.key_pressed(Key::F7) {
                    cmds.push(Cmd::InsertBlankKey);
                }
                if i.key_pressed(Key::Enter) {
                    cmds.push(Cmd::Play);
                }
                if i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace) {
                    cmds.push(Cmd::ClearFrame);
                }
                if i.key_pressed(Key::OpenBracket) {
                    brush_delta = -2.0;
                }
                if i.key_pressed(Key::CloseBracket) {
                    brush_delta = 2.0;
                }
                if i.key_pressed(Key::Home) {
                    cmds.push(Cmd::FirstFrame);
                }
                if i.key_pressed(Key::End) {
                    cmds.push(Cmd::LastFrame);
                }
                if i.key_pressed(Key::ArrowLeft) {
                    cmds.push(Cmd::PrevFrame);
                }
                if i.key_pressed(Key::ArrowRight) {
                    cmds.push(Cmd::NextFrame);
                }
                let scroll = i.raw_scroll_delta.y;
                if scroll != 0.0 && i.modifiers.alt {
                    let steps = (scroll / 40.0).round() as i64;
                    if steps != 0 {
                        let f = (self.current_frame as i64 - steps).max(0) as usize;
                        seek_key = Some(f);
                    }
                }
            }
        });

        if let Some(t) = tool {
            self.tool = t;
            self.notify(format!("{} ({})", t.name(), t.letter()));
        }
        if brush_delta != 0.0 {
            match self.tool {
                Tool::Brush => {
                    self.brush.size = (self.brush.size + brush_delta).clamp(1.0, 400.0);
                    self.notify(format!("Brush size {:.0} px", self.brush.size));
                }
                Tool::Eraser => {
                    self.eraser.size = (self.eraser.size + brush_delta).clamp(1.0, 400.0);
                    self.notify(format!("Eraser size {:.0} px", self.eraser.size));
                }
                _ => {}
            }
        }
        if let Some(f) = seek_key {
            self.set_frame(f);
        }
        for c in cmds {
            self.request(c);
        }
    }

    // -- panels -------------------------------------------------------------

    fn menu_bar(&mut self, ctx: &Context) {
        let mut cmd: Option<Cmd> = None;
        let mut tool: Option<Tool> = None;
        let mut toggle_onion = false;
        egui::TopBottomPanel::top("menu_bar")
            .exact_height(26.0)
            .frame(ui::panel_frame(theme::TOOLBAR).inner_margin(egui::Margin::symmetric(6, 0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    let menu_text = |s: &str| egui::RichText::new(s).size(12.0).color(theme::TEXT);

                    ui.menu_button(menu_text("File"), |ui| {
                        if ui.button("New…            Ctrl+N").clicked() {
                            cmd = Some(Cmd::New);
                            ui.close();
                        }
                        if ui.button("Open…           Ctrl+O").clicked() {
                            cmd = Some(Cmd::Open);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Save            Ctrl+S").clicked() {
                            cmd = Some(Cmd::Save);
                            ui.close();
                        }
                        if ui.button("Save As…    Ctrl+Shift+S").clicked() {
                            cmd = Some(Cmd::SaveAs);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Export Frame (PNG)   Ctrl+E").clicked() {
                            cmd = Some(Cmd::ExportPng);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Exit").clicked() {
                            cmd = Some(Cmd::Exit);
                            ui.close();
                        }
                    });
                    ui.menu_button(menu_text("Edit"), |ui| {
                        if ui.button("Undo            Ctrl+Z").clicked() {
                            cmd = Some(Cmd::Undo);
                            ui.close();
                        }
                        if ui.button("Redo       Ctrl+Shift+Z").clicked() {
                            cmd = Some(Cmd::Redo);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Clear Drawing         Del").clicked() {
                            cmd = Some(Cmd::ClearFrame);
                            ui.close();
                        }
                    });
                    ui.menu_button(menu_text("View"), |ui| {
                        if ui.button("Zoom In          Ctrl++").clicked() {
                            cmd = Some(Cmd::ZoomIn);
                            ui.close();
                        }
                        if ui.button("Zoom Out         Ctrl+-").clicked() {
                            cmd = Some(Cmd::ZoomOut);
                            ui.close();
                        }
                        if ui.button("Fit in Window    Ctrl+0").clicked() {
                            cmd = Some(Cmd::FitView);
                            ui.close();
                        }
                        if ui.button("100%").clicked() {
                            cmd = Some(Cmd::ActualSize);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Onion Skin        Alt+O").clicked() {
                            toggle_onion = true;
                            ui.close();
                        }
                    });
                    ui.menu_button(menu_text("Insert"), |ui| {
                        if ui.button("Frame                   F5").clicked() {
                            cmd = Some(Cmd::InsertFrame);
                            ui.close();
                        }
                        if ui.button("Keyframe              F6").clicked() {
                            cmd = Some(Cmd::InsertKey);
                            ui.close();
                        }
                        if ui.button("Blank Keyframe     F7").clicked() {
                            cmd = Some(Cmd::InsertBlankKey);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Remove Frame   Shift+F5").clicked() {
                            cmd = Some(Cmd::RemoveFrame);
                            ui.close();
                        }
                        if ui.button("Clear Keyframe  Shift+F6").clicked() {
                            cmd = Some(Cmd::ClearKey);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("New Layer").clicked() {
                            cmd = Some(Cmd::NewLayer);
                            ui.close();
                        }
                    });
                    ui.menu_button(menu_text("Modify"), |ui| {
                        if ui.button("Document…").clicked() {
                            cmd = Some(Cmd::DocProps);
                            ui.close();
                        }
                        if ui.button("Layer…").clicked() {
                            cmd = Some(Cmd::LayerProps);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Duplicate Layer").clicked() {
                            cmd = Some(Cmd::DuplicateLayer);
                            ui.close();
                        }
                        if ui.button("Delete Layer").clicked() {
                            cmd = Some(Cmd::DeleteLayer);
                            ui.close();
                        }
                    });
                    ui.menu_button(menu_text("Control"), |ui| {
                        if ui.button(if self.playing { "Stop" } else { "Play" }).clicked() {
                            cmd = Some(Cmd::Play);
                            ui.close();
                        }
                        if ui.button("Loop").clicked() {
                            cmd = Some(Cmd::LoopToggle);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("First Frame").clicked() {
                            cmd = Some(Cmd::FirstFrame);
                            ui.close();
                        }
                        if ui.button("Last Frame").clicked() {
                            cmd = Some(Cmd::LastFrame);
                            ui.close();
                        }
                    });
                    ui.menu_button(menu_text("Window"), |ui| {
                        ui.label(egui::RichText::new("Timeline, Tools and Properties are docked.").size(11.0).color(theme::TEXT_DIM));
                    });
                    ui.menu_button(menu_text("Help"), |ui| {
                        if ui.button("About TAniCraft").clicked() {
                            cmd = Some(Cmd::About);
                            ui.close();
                        }
                        ui.separator();
                        ui.label(egui::RichText::new("B Brush   E Eraser   H Hand   Z Zoom").size(11.0).color(theme::TEXT_DIM));
                        ui.label(egui::RichText::new("F5 Frame   F6 Keyframe   F7 Blank Key").size(11.0).color(theme::TEXT_DIM));
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(6.0);
                    for (t, icon) in [
                        (Tool::Brush, Icon::Brush),
                        (Tool::Eraser, Icon::Eraser),
                        (Tool::Hand, Icon::Hand),
                        (Tool::Zoom, Icon::Zoom),
                    ] {
                        if ui::tool_btn(ui, icon, self.tool == t, &format!("{} ({})", t.name(), t.letter())).clicked() {
                            tool = Some(t);
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("Essentials  ▾").size(11.0).color(theme::TEXT_DIM));
                        ui.separator();
                        ui.label(egui::RichText::new("Workspace").size(10.5).color(theme::TEXT_FAINT));
                    });
                });
            });
        if let Some(t) = tool {
            self.tool = t;
            self.notify(format!("{} ({})", t.name(), t.letter()));
        }
        if toggle_onion {
            self.onion.enabled = !self.onion.enabled;
        }
        if let Some(c) = cmd {
            self.request(c);
        }
    }

    fn tab_bar(&mut self, ctx: &Context) {
        let mut cmd: Option<Cmd> = None;
        egui::TopBottomPanel::top("doc_tabs")
            .exact_height(28.0)
            .frame(ui::panel_frame(theme::PANEL_HEADER).inner_margin(egui::Margin::symmetric(0, 0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(4.0);
                    let name = if let Some(p) = &self.path {
                        p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
                    } else {
                        format!("{}.tcraft", self.doc.name)
                    };
                    let label = format!("{}{}", name, if self.dirty { " *" } else { "" });
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(190.0, 24.0), egui::Sense::click());
                    let active = true;
                    ui.painter().rect_filled(
                        rect,
                        egui::CornerRadius {
                            nw: 3,
                            ne: 3,
                            sw: 0,
                            se: 0,
                        },
                        if active { theme::PANEL_ALT } else { theme::PANEL },
                    );
                    ui.painter().text(
                        rect.left_center() + Vec2::new(10.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        &label,
                        egui::FontId::proportional(11.5),
                        theme::TEXT,
                    );
                    if resp.hovered() {
                        ui.painter().rect_stroke(
                            rect,
                            2.0,
                            egui::Stroke::new(1.0_f32, theme::BORDER_LIGHT),
                            egui::StrokeKind::Inside,
                        );
                    }
                    // close cross
                    let cross = Rect::from_center_size(rect.right_center() - Vec2::new(14.0, 0.0), Vec2::splat(12.0));
                    if ui
                        .interact(cross, ui.id().with("doc_close"), egui::Sense::click())
                        .on_hover_text("Close document")
                        .clicked()
                    {
                        cmd = Some(Cmd::New);
                    }
                    ui.painter().line_segment(
                        [cross.left_top(), cross.right_bottom()],
                        egui::Stroke::new(1.0_f32, theme::TEXT_DIM),
                    );
                    ui.painter().line_segment(
                        [cross.right_top(), cross.left_bottom()],
                        egui::Stroke::new(1.0_f32, theme::TEXT_DIM),
                    );
                    ui.add_space(2.0);
                    if ui::icon_btn(ui, Icon::Plus, "New document (Ctrl+N)").clicked() {
                        cmd = Some(Cmd::New);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "{} × {}   {} fps",
                                self.doc.width, self.doc.height, self.doc.fps as i32
                            ))
                            .size(10.5)
                            .color(theme::TEXT_FAINT),
                        );
                    });
                });
            });
        if let Some(c) = cmd {
            self.request(c);
        }
    }

    fn status_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::bottom("status")
            .exact_height(22.0)
            .frame(ui::panel_frame(theme::PANEL_HEADER).inner_margin(egui::Margin::symmetric(8, 0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(egui::RichText::new(&self.status).size(10.5).color(theme::TEXT_DIM));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{:.0}%", self.view.zoom * 100.0))
                                .size(10.5)
                                .color(theme::TEXT_DIM),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!(
                                "Frame {} / {}",
                                self.current_frame + 1,
                                self.frame_count()
                            ))
                            .size(10.5)
                            .color(theme::TEXT_DIM),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(self.tool.name())
                                .size(10.5)
                                .color(theme::TEXT_DIM),
                        );
                    });
                });
            });
    }

    fn tools_panel(&mut self, ctx: &Context) {
        egui::SidePanel::right("tools_panel")
            .exact_width(62.0)
            .resizable(false)
            .frame(ui::panel_frame(theme::PANEL).inner_margin(egui::Margin::symmetric(4, 6)))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(2.0, 2.0);
                ui.vertical_centered(|ui| {
                    egui::Grid::new("tool_grid")
                        .spacing(Vec2::new(2.0, 2.0))
                        .min_col_width(24.0)
                        .show(ui, |ui| {
                            let entries: [(Icon, Option<Tool>, &str); 12] = [
                                (Icon::Selection, None, "Selection (V) — not yet available"),
                                (Icon::Transform, None, "Free Transform (Q) — not yet available"),
                                (Icon::Lasso, None, "Lasso (L) — not yet available"),
                                (Icon::Pen, None, "Pen (P) — not yet available"),
                                (Icon::Pencil, None, "Pencil (Y) — not yet available"),
                                (Icon::Brush, Some(Tool::Brush), "Brush (B)"),
                                (Icon::Eraser, Some(Tool::Eraser), "Eraser (E)"),
                                (Icon::Bucket, None, "Paint Bucket (K) — not yet available"),
                                (Icon::Eyedropper, None, "Eyedropper (I) — not yet available"),
                                (Icon::Text, None, "Text (T) — not yet available"),
                                (Icon::Hand, Some(Tool::Hand), "Hand (H)"),
                                (Icon::Zoom, Some(Tool::Zoom), "Zoom (Z)"),
                            ];
                            for (i, (icon, tool, tip)) in entries.iter().enumerate() {
                                let active = tool.map(|t| t == self.tool).unwrap_or(false);
                                let resp = match tool {
                                    Some(_) => ui::tool_btn(ui, *icon, active, tip),
                                    None => {
                                        ui.add_enabled_ui(false, |ui| {
                                            ui::tool_btn(ui, *icon, false, tip)
                                        })
                                        .inner
                                    }
                                };
                                if resp.clicked() {
                                    if let Some(t) = tool {
                                        self.tool = *t;
                                        self.notify(format!("{} ({})", t.name(), t.letter()));
                                    }
                                }
                                if i % 2 == 1 {
                                    ui.end_row();
                                }
                            }
                        });

                    ui.add_space(6.0);
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(50.0, 44.0), egui::Sense::hover());
                    // stroke (outline) behind, fill in front — like Adobe's colour chips
                    let stroke_rect = Rect::from_min_size(rect.left_top() + Vec2::new(2.0, 2.0), Vec2::splat(24.0));
                    let fill_rect = Rect::from_min_size(rect.left_top() + Vec2::new(16.0, 16.0), Vec2::splat(24.0));
                    ui.painter().rect_filled(stroke_rect, 1.0, Color32::from_gray(0x18));
                    ui.painter().rect_stroke(stroke_rect, 1.0, egui::Stroke::new(1.0_f32, Color32::from_gray(0x0a)), egui::StrokeKind::Inside);
                    let stroke_resp = ui.interact(stroke_rect, ui.id().with("stroke_swatch"), egui::Sense::click());
                    ui.painter().rect_filled(fill_rect, 1.0, self.brush.color);
                    ui.painter().rect_stroke(fill_rect, 1.0, egui::Stroke::new(1.0_f32, Color32::from_gray(0x0a)), egui::StrokeKind::Inside);
                    let fill_resp = ui.interact(fill_rect, ui.id().with("fill_swatch"), egui::Sense::click());
                    if fill_resp.on_hover_text("Fill colour (used by the brush)").clicked() {
                        self.show_fill_picker = true;
                    }
                    let _ = stroke_resp.on_hover_text("Stroke colour");

                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        if ui::small_icon_btn(ui, Icon::Swap, "Swap fill and stroke").clicked() {
                            self.brush.color = Color32::from_gray(0x18);
                        }
                        if ui::small_icon_btn(ui, Icon::BlankKey, "Default colours").clicked() {
                            self.brush.color = Color32::from_gray(0x18);
                        }
                    });
                });
            });
    }

    fn properties_panel(&mut self, ctx: &Context) {
        egui::SidePanel::right("properties_panel")
            .default_width(252.0)
            .min_width(210.0)
            .max_width(380.0)
            .frame(ui::panel_frame(theme::PANEL).inner_margin(egui::Margin::symmetric(0, 0)))
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 22.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 0.0, theme::PANEL_HEADER);
                ui.painter().text(
                    rect.left_center() + Vec2::new(8.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    "PROPERTIES",
                    egui::FontId::proportional(10.5),
                    theme::TEXT_DIM,
                );

                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width() - 8.0);
                            self.properties_content(ui);
                        });
                    });
                    ui.add_space(10.0);
                });
            });
    }

    fn properties_content(&mut self, ui: &mut egui::Ui) {
        // ---- tool ---------------------------------------------------------
        ui::section_title(ui, "TOOL");
        ui.add_space(3.0);
        ui.horizontal(|ui| {
            let icon = match self.tool {
                Tool::Brush => Icon::Brush,
                Tool::Eraser => Icon::Eraser,
                Tool::Hand => Icon::Hand,
                Tool::Zoom => Icon::Zoom,
            };
            let (r, _) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::hover());
            crate::icons::paint(ui.painter(), r.shrink(1.0), icon, theme::TEXT);
            ui.label(egui::RichText::new(self.tool.name()).size(12.0).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(format!("[{}]", self.tool.letter())).size(10.0).color(theme::TEXT_FAINT));
            });
        });
        ui.add_space(2.0);

        match self.tool {
            Tool::Brush => {
                ui::slider_row(ui, "Size", &mut self.brush.size, 1.0..=300.0, 0);
                ui::slider_row(ui, "Hardness", &mut self.brush.hardness, 0.0..=1.0, 2);
                ui::slider_row(ui, "Opacity", &mut self.brush.opacity, 0.05..=1.0, 2);
                ui::slider_row(ui, "Smoothing", &mut self.brush.smoothing, 0.0..=0.95, 2);
                ui.add_space(3.0);
                ui.horizontal(|ui| {
                    ui.add_space(2.0);
                    let resp = ui.color_edit_button_srgba(&mut self.brush.color)
                        .on_hover_text("Brush colour");
                    let _ = resp;
                    ui.label(egui::RichText::new("Fill colour").size(11.0).color(theme::TEXT_DIM));
                });
            }
            Tool::Eraser => {
                ui::slider_row(ui, "Size", &mut self.eraser.size, 1.0..=300.0, 0);
                ui::slider_row(ui, "Hardness", &mut self.eraser.hardness, 0.0..=1.0, 2);
                ui::slider_row(ui, "Strength", &mut self.eraser.opacity, 0.05..=1.0, 2);
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("The eraser removes pixels and restores transparency.")
                        .size(10.5)
                        .color(theme::TEXT_FAINT),
                );
            }
            Tool::Hand => {
                ui.label(egui::RichText::new("Drag the stage to pan.\nSpace-drag works with any tool.").size(11.0).color(theme::TEXT_DIM));
            }
            Tool::Zoom => {
                ui.label(egui::RichText::new("Click to zoom in, Alt-click to zoom out.\nCtrl+wheel zooms around the cursor.").size(11.0).color(theme::TEXT_DIM));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button("Fit in window").clicked() {
                        self.fit_view();
                    }
                    if ui.button("100%").clicked() {
                        self.view.zoom = 1.0;
                        self.view.offset = Vec2::ZERO;
                    }
                });
            }
        }

        // ---- frame --------------------------------------------------------
        ui::section_title(ui, "FRAME");
        ui.add_space(3.0);
        let layer_name = self
            .current_layer()
            .map(|l| l.name.clone())
            .unwrap_or_else(|| "—".into());
        let kind = self.layer_kind_at_current(self.sel_layer);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("Frame {}", self.current_frame + 1)).size(11.0));
            ui.label(egui::RichText::new("|").size(11.0).color(theme::TEXT_FAINT));
            let kind_text = match kind {
                FrameKind::Key => {
                    let blank = self.doc.layers.get(self.sel_layer)
                        .and_then(|l| l.effective_key(self.current_frame))
                        .and_then(|k| self.doc.layers[self.sel_layer].frames[k].key_bitmap())
                        .map(|b| b.is_blank())
                        .unwrap_or(true);
                    if blank { "Blank keyframe" } else { "Keyframe" }
                }
                FrameKind::Hold => "Hold frame",
                FrameKind::Empty => "Empty frame",
            };
            ui.label(egui::RichText::new(kind_text).size(11.0).color(theme::TEXT_DIM));
        });
        ui.label(egui::RichText::new(layer_name).size(10.5).color(theme::TEXT_FAINT));
        ui.add_space(3.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("F6  Keyframe").on_hover_text("Duplicate the drawing into a new keyframe").clicked() {
                self.request(Cmd::InsertKey);
            }
            if ui.button("F7  Blank").clicked() {
                self.request(Cmd::InsertBlankKey);
            }
            if ui.button("F5  Extend").clicked() {
                self.request(Cmd::InsertFrame);
            }
            if ui.button("Clear key").clicked() {
                self.request(Cmd::ClearKey);
            }
        });

        // ---- layer --------------------------------------------------------
        ui::section_title(ui, "LAYER");
        ui.add_space(3.0);
        let layer_data = self
            .doc
            .layers
            .get(self.sel_layer)
            .map(|l| (l.name.clone(), l.visible, l.locked));
        if let Some((mut name, mut vis, mut lock)) = layer_data {
            let mut changed = false;
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Name").size(11.0));
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut name)
                        .desired_width(ui.available_width().max(80.0))
                        .font(egui::TextStyle::Body),
                );
                if resp.changed() {
                    changed = true;
                }
            });
            ui.horizontal(|ui| {
                if ui.checkbox(&mut vis, "Visible").changed() {
                    changed = true;
                }
                if ui.checkbox(&mut lock, "Locked").changed() {
                    changed = true;
                }
            });
            if changed {
                if let Some(layer) = self.doc.layers.get_mut(self.sel_layer) {
                    layer.name = name;
                    layer.visible = vis;
                    layer.locked = lock;
                }
                self.dirty = true;
            }
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                if ui.button("Layer props…").clicked() {
                    self.request(Cmd::LayerProps);
                }
                if ui.button("Delete").clicked() {
                    self.request(Cmd::DeleteLayer);
                }
            });
        }

        // ---- document -----------------------------------------------------
        ui::section_title(ui, "DOCUMENT");
        ui.add_space(3.0);
        egui::Grid::new("doc_props_grid")
            .num_columns(2)
            .spacing(Vec2::new(8.0, 3.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Size").size(11.0).color(theme::TEXT_DIM));
                ui.label(
                    egui::RichText::new(format!("{} × {} px", self.doc.width, self.doc.height))
                        .size(11.0),
                );
                ui.end_row();
                ui.label(egui::RichText::new("Frame rate").size(11.0).color(theme::TEXT_DIM));
                ui.label(egui::RichText::new(format!("{:.2} fps", self.doc.fps)).size(11.0));
                ui.end_row();
                ui.label(egui::RichText::new("Duration").size(11.0).color(theme::TEXT_DIM));
                let secs = self.frame_count() as f32 / self.doc.fps.max(1.0);
                ui.label(egui::RichText::new(format!("{} frames ({:.2}s)", self.frame_count(), secs)).size(11.0));
                ui.end_row();
                ui.label(egui::RichText::new("Layers").size(11.0).color(theme::TEXT_DIM));
                ui.label(egui::RichText::new(self.doc.layers.len().to_string()).size(11.0));
                ui.end_row();
            });
        ui.add_space(4.0);
        if ui.button("Document properties…").clicked() {
            self.request(Cmd::DocProps);
        }
        ui.add_space(6.0);
    }

    // -- playback -----------------------------------------------------------

    fn update_playback(&mut self, ctx: &Context) {
        if !self.playing {
            return;
        }
        let dt = ctx.input(|i| i.stable_dt).min(0.25);
        self.play_accum += dt * self.doc.fps.max(1.0);
        let steps = self.play_accum.floor() as usize;
        if steps > 0 {
            self.play_accum -= steps as f32;
            let fc = self.frame_count();
            let mut f = self.current_frame + steps;
            if f >= fc {
                if self.looping {
                    f %= fc;
                } else {
                    f = fc - 1;
                    self.playing = false;
                }
            }
            self.current_frame = f;
        }
        ctx.request_repaint();
    }

    fn update_title(&mut self, ctx: &Context) {
        let title = format!(
            "{}{} — TAniCraft",
            self.doc.name,
            if self.dirty { " *" } else { "" }
        );
        if title != self.title_cache {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title_cache = title;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ctx);
        self.menu_bar(ctx);
        self.tab_bar(ctx);
        self.status_bar(ctx);
        self.timeline_panel(ctx);
        self.tools_panel(ctx);
        self.properties_panel(ctx);

        egui::CentralPanel::default()
            .frame(ui::panel_frame(theme::STAGE_BG))
            .show(ctx, |ui| {
                self.stage_ui(ui);
            });

        self.dialogs(ctx);
        self.update_playback(ctx);
        self.update_title(ctx);
        if self.want_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
