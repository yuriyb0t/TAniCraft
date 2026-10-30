//! An Animate-style timeline: layers on the left, frame grid on the right.

use egui::{Align2, Color32, Context, FontId, Pos2, Rect, Sense, Stroke, Vec2};

use crate::app::{App, Cmd};
use crate::doc::FrameKind;
use crate::icons::Icon;
use crate::theme;
use crate::ui;

pub const FRAME_W: f32 = 16.0;
pub const ROW_H: f32 = 22.0;
pub const RULER_H: f32 = 20.0;
const LEFT_W: f32 = 208.0;

impl App {
    pub fn timeline_panel(&mut self, ctx: &Context) {
        egui::TopBottomPanel::bottom("timeline")
            .resizable(true)
            .default_height(236.0)
            .height_range(120.0..=620.0)
            .frame(ui::panel_frame(theme::PANEL))
            .show(ctx, |ui| {
                self.timeline_toolbar(ui);
                let body = ui.available_rect_before_wrap();
                if body.height() > 20.0 {
                    // Claim the full body rect so the panel reports its real
                    // height to egui; otherwise a resized timeline collapses
                    // back to its minimum height on the next frame.
                    ui.allocate_rect(body, Sense::hover());
                    self.timeline_body(ui, body);
                }
            });
    }

    fn timeline_toolbar(&mut self, ui: &mut egui::Ui) {
        let mut cmd: Option<Cmd> = None;
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 28.0), Sense::hover());
        ui.painter().rect_filled(rect, 0.0, theme::TOOLBAR);
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0_f32, theme::BORDER),
        );
        let mut ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect.shrink2(Vec2::new(4.0, 3.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        ui.spacing_mut().item_spacing.x = 2.0;

        if ui::icon_btn(&mut ui, Icon::ToStart, "First frame (Home)").clicked() {
            cmd = Some(Cmd::FirstFrame);
        }
        if ui::icon_btn(&mut ui, Icon::StepBack, "Previous frame (←)").clicked() {
            cmd = Some(Cmd::PrevFrame);
        }
        let play_icon = if self.playing { Icon::Pause } else { Icon::Play };
        let play = ui.allocate_exact_size(Vec2::new(26.0, 22.0), Sense::click());
        if play.1.hovered() {
            ui.painter().rect_filled(play.0, 2.0, theme::BUTTON_HOVER);
        }
        crate::icons::paint(ui.painter(), play.0.shrink(5.0), play_icon, theme::TEXT);
        let play = play.1.on_hover_text("Play / Stop (Enter)");
        if play.clicked() {
            cmd = Some(Cmd::Play);
        }
        if ui::icon_btn(&mut ui, Icon::StepForward, "Next frame (→)").clicked() {
            cmd = Some(Cmd::NextFrame);
        }
        if ui::icon_btn(&mut ui, Icon::ToEnd, "Last frame (End)").clicked() {
            cmd = Some(Cmd::LastFrame);
        }
        ui.add_space(2.0);
        ui::divider(&mut ui);
        if ui::icon_btn(&mut ui, Icon::Loop, "Loop playback").clicked() {
            cmd = Some(Cmd::LoopToggle);
        }
        // loop indicator
        let lr = ui.allocate_exact_size(Vec2::new(4.0, 12.0), Sense::hover());
        ui.painter().rect_filled(
            lr.0,
            1.0,
            if self.looping { theme::ACCENT } else { theme::BORDER_LIGHT },
        );

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!(
                "{:>4} / {}",
                self.current_frame + 1,
                self.frame_count()
            ))
            .monospace()
            .size(11.0)
            .color(theme::TEXT),
        );
        ui.add_space(2.0);
        let mut fps = self.doc.fps;
        let fps_resp = ui.add(
            egui::DragValue::new(&mut fps)
                .speed(1.0)
                .range(1.0..=120.0)
                .fixed_decimals(1)
                .suffix(" fps"),
        );
        if fps_resp.changed() {
            self.doc.fps = fps.clamp(1.0, 120.0);
            self.dirty = true;
        }

        ui.add_space(4.0);
        ui::divider(&mut ui);

        if ui::icon_btn(&mut ui, Icon::Keyframe, "Insert keyframe (F6)").clicked() {
            cmd = Some(Cmd::InsertKey);
        }
        if ui::icon_btn(&mut ui, Icon::BlankKey, "Insert blank keyframe (F7)").clicked() {
            cmd = Some(Cmd::InsertBlankKey);
        }
        if ui::icon_btn(&mut ui, Icon::Plus, "Insert frame (F5)").clicked() {
            cmd = Some(Cmd::InsertFrame);
        }
        if ui::icon_btn(&mut ui, Icon::Trash, "Remove frame (Shift+F5)").clicked() {
            cmd = Some(Cmd::RemoveFrame);
        }
        if ui::icon_btn(&mut ui, Icon::EmptyFrame, "Clear keyframe (Shift+F6)").clicked() {
            cmd = Some(Cmd::ClearKey);
        }

        ui.add_space(4.0);
        ui::divider(&mut ui);

        let onion_active = self.onion.enabled;
        if ui::tool_btn(&mut ui, Icon::Onion, onion_active, "Onion skin (Alt+O)").clicked() {
            self.onion.enabled = !self.onion.enabled;
        }
        if self.onion.enabled {
            ui.label(egui::RichText::new("before").size(9.5).color(theme::TEXT_FAINT));
            ui.add(
                egui::DragValue::new(&mut self.onion.before)
                    .speed(0.2)
                    .range(0..=20),
            );
            ui.label(egui::RichText::new("after").size(9.5).color(theme::TEXT_FAINT));
            ui.add(
                egui::DragValue::new(&mut self.onion.after)
                    .speed(0.2)
                    .range(0..=20),
            );
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let layer_name = self
                .doc
                .layers
                .get(self.sel_layer)
                .map(|l| l.name.clone())
                .unwrap_or_default();
            ui.label(
                egui::RichText::new(format!("layer: {layer_name}"))
                    .size(10.0)
                    .color(theme::TEXT_FAINT),
            );
        });

        if let Some(c) = cmd {
            self.request(c);
        }
    }

    fn timeline_body(&mut self, ui: &mut egui::Ui, body: Rect) {
        let painter = ui.painter_at(body);
        painter.rect_filled(body, 0.0, theme::TIMELINE_BG);

        let left_w = LEFT_W.min((body.width() * 0.55).max(120.0));
        let left = Rect::from_min_max(body.min, Pos2::new(body.left() + left_w, body.bottom()));
        let right = Rect::from_min_max(Pos2::new(left.right() + 1.0, body.top()), body.max);
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(left.right(), body.top()), Vec2::new(1.0, body.height())),
            0.0,
            theme::BORDER,
        );

        let header_left = Rect::from_min_size(left.min, Vec2::new(left.width(), RULER_H));
        let ruler = Rect::from_min_size(right.min, Vec2::new(right.width(), RULER_H));
        let rows = Rect::from_min_max(
            Pos2::new(right.left(), right.top() + RULER_H),
            right.max - Vec2::new(0.0, 6.0),
        );
        let left_rows = Rect::from_min_max(
            Pos2::new(left.left(), left.top() + RULER_H),
            Pos2::new(left.right(), rows.bottom()),
        );

        let layer_count = self.doc.layers.len();
        let content_h = layer_count as f32 * ROW_H;
        let max_scroll_y = (content_h - left_rows.height()).max(0.0);
        let total_frames = self.frame_count().max(self.current_frame + 1) + 48;
        let max_scroll_x = (total_frames as f32 * FRAME_W - rows.width()).max(0.0);

        // ---- scrolling -----------------------------------------------------
        if ui.rect_contains_pointer(body) && self.stroke.is_none() {
            let (delta, shift) = ui.input(|i| (i.smooth_scroll_delta, i.modifiers.shift));
            if shift {
                self.tl_scroll_x -= delta.y + delta.x;
            } else {
                self.tl_scroll_x -= delta.x;
                self.tl_scroll_y -= delta.y;
            }
        }
        self.tl_scroll_y = self.tl_scroll_y.clamp(0.0, max_scroll_y);
        self.tl_scroll_x = self.tl_scroll_x.clamp(0.0, max_scroll_x);
        let scroll_x = self.tl_scroll_x;

        let x_of = |frame: i64| ruler.left() - scroll_x + frame as f32 * FRAME_W;
        let frame_at = |x: f32| -> i64 {
            ((x - ruler.left() + scroll_x) / FRAME_W).floor() as i64
        };

        // ---- ruler interaction --------------------------------------------
        let ruler_resp = ui.interact(ruler, ui.id().with("tl_ruler"), Sense::click_and_drag());
        if (ruler_resp.clicked() || ruler_resp.dragged()) && self.stroke.is_none() {
            if let Some(p) = ruler_resp.interact_pointer_pos() {
                let f = frame_at(p.x).max(0) as usize;
                self.set_frame(f.min(self.frame_count() + 48));
                self.playing = false;
            }
        }

        // ---- cell interaction ---------------------------------------------
        let i0 = (self.tl_scroll_x / FRAME_W).floor() as i64 - 1;
        let i1 = i0 + (rows.width() / FRAME_W).ceil() as i64 + 2;
        let row_at = |y: f32, scroll_y: f32, top: f32| -> Option<usize> {
            let r = ((y - top + scroll_y) / ROW_H).floor() as i64;
            if r < 0 || r as usize >= layer_count {
                None
            } else {
                Some(layer_count - 1 - r as usize)
            }
        };

        let cells_resp = ui.interact(rows, ui.id().with("tl_cells"), Sense::click_and_drag());
        if let Some(p) = cells_resp.interact_pointer_pos() {
            let cell = row_at(p.y, self.tl_scroll_y, rows.top())
                .map(|li| (li, frame_at(p.x).max(0) as usize));
            if let Some((li, f)) = cell {
                if cells_resp.clicked() || cells_resp.double_clicked() || cells_resp.dragged() {
                    self.sel_layer = li;
                    self.set_frame(f);
                    self.playing = false;
                }
                if cells_resp.clicked_by(egui::PointerButton::Secondary) {
                    self.menu_target = Some((li, f));
                    self.sel_layer = li;
                    self.set_frame(f);
                }
            }
        }
        if cells_resp.double_clicked() {
            if let Some(p) = cells_resp.interact_pointer_pos() {
                let li = row_at(p.y, self.tl_scroll_y, rows.top());
                let f = frame_at(p.x).max(0) as usize;
                if let Some(li) = li {
                    if self.doc.layers.get(li).map(|l| l.kind_at(f)) != Some(FrameKind::Key) {
                        let before = self.doc.clone();
                        self.doc.insert_keyframe(li, f, false);
                        self.sel_layer = li;
                        self.set_frame(f);
                        self.commit(before);
                        self.notify("Inserted keyframe (double-click)");
                    }
                }
            }
        }

        let mut cmd: Option<Cmd> = None;
        cells_resp.context_menu(|ui| {
            ui.set_min_width(190.0);
            ui.label(egui::RichText::new("Timeline").size(10.5).color(theme::TEXT_FAINT));
            ui.separator();
            if ui.button("Insert Frame            F5").clicked() {
                cmd = Some(Cmd::InsertFrame);
                ui.close();
            }
            if ui.button("Insert Keyframe        F6").clicked() {
                cmd = Some(Cmd::InsertKey);
                ui.close();
            }
            if ui.button("Insert Blank Keyframe  F7").clicked() {
                cmd = Some(Cmd::InsertBlankKey);
                ui.close();
            }
            ui.separator();
            if ui.button("Remove Frame      Shift+F5").clicked() {
                cmd = Some(Cmd::RemoveFrame);
                ui.close();
            }
            if ui.button("Clear Keyframe    Shift+F6").clicked() {
                cmd = Some(Cmd::ClearKey);
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
        if let Some(c) = cmd {
            self.request(c);
        }

        // ---- layer header --------------------------------------------------
        {
            let p = ui.painter_at(header_left);
            p.rect_filled(header_left, 0.0, theme::PANEL_HEADER);
            p.line_segment(
                [header_left.left_bottom(), header_left.right_bottom()],
                Stroke::new(1.0_f32, theme::BORDER),
            );
            p.text(
                header_left.left_center() + Vec2::new(8.0, 1.0),
                Align2::LEFT_CENTER,
                "LAYERS",
                FontId::proportional(10.0),
                theme::TEXT_DIM,
            );
            let mut bx = header_left.right() - 22.0;
            let mut mk = |icon: Icon, tip: &str, p: &egui::Painter| -> bool {
                let r = Rect::from_center_size(Pos2::new(bx, header_left.center().y), Vec2::splat(18.0));
                bx -= 19.0;
                let resp = ui.interact(r, ui.id().with(("lh", bx as i32)), Sense::click());
                if resp.hovered() {
                    p.rect_filled(r, 2.0, theme::BUTTON_HOVER);
                }
                crate::icons::paint(
                    p,
                    r.shrink(3.0),
                    icon,
                    if resp.hovered() { Color32::WHITE } else { theme::TEXT_DIM },
                );
                resp.on_hover_text(tip).clicked()
            };
            let mut layer_cmd: Option<Cmd> = None;
            if mk(Icon::Plus, "New layer", &p) {
                layer_cmd = Some(Cmd::NewLayer);
            }
            if mk(Icon::Copy, "Duplicate layer", &p) {
                layer_cmd = Some(Cmd::DuplicateLayer);
            }
            if mk(Icon::Trash, "Delete layer", &p) {
                layer_cmd = Some(Cmd::DeleteLayer);
            }
            if mk(Icon::ArrowUp, "Move layer up", &p) {
                layer_cmd = Some(Cmd::MoveLayerUp);
            }
            if mk(Icon::ArrowDown, "Move layer down", &p) {
                layer_cmd = Some(Cmd::MoveLayerDown);
            }
            if let Some(c) = layer_cmd {
                self.request(c);
            }
        }

        // ---- ruler ---------------------------------------------------------
        {
            let p = ui.painter_at(ruler);
            p.rect_filled(ruler, 0.0, theme::PANEL_HEADER);
            p.line_segment(
                [ruler.left_bottom(), ruler.right_bottom()],
                Stroke::new(1.0_f32, theme::BORDER),
            );
            for i in i0.max(0)..=i1 {
                let x = x_of(i);
                if x < ruler.left() - FRAME_W || x > ruler.right() + FRAME_W {
                    continue;
                }
                let strong = (i + 1) % 5 == 0;
                let h = if strong { 7.0 } else { 3.0 };
                p.line_segment(
                    [Pos2::new(x, ruler.bottom() - h), Pos2::new(x, ruler.bottom())],
                    Stroke::new(1.0_f32, if strong { theme::GRID_STRONG } else { theme::GRID }),
                );
                if strong {
                    p.text(
                        Pos2::new(x + 2.0, ruler.top() + 1.0),
                        Align2::LEFT_TOP,
                        format!("{}", i + 1),
                        FontId::proportional(9.0),
                        theme::TEXT_DIM,
                    );
                }
            }
        }

        // ---- frame grid + layer rows --------------------------------------
        let mut rows_meta: Vec<(usize, Vec<(i64, FrameKind, bool, bool)>)> = Vec::new();
        {
            let doc = &self.doc;
            let cache = &mut self.blank_cache;
            for li in (0..layer_count).rev() {
                let layer = &doc.layers[li];
                let mut cells = Vec::new();
                for i in i0.max(0)..=i1 {
                    let kind = layer.kind_at(i as usize);
                    let has = layer.effective_key(i as usize).is_some();
                    let mut blank = false;
                    if kind == FrameKind::Key {
                        if let Some(bmp) = layer.frames[i as usize].bitmap.as_ref() {
                            match cache.get(&(layer.uid, i as usize)) {
                                Some((rev, b)) if *rev == bmp.rev => blank = *b,
                                _ => {
                                    blank = bmp.is_blank();
                                    cache.insert((layer.uid, i as usize), (bmp.rev, blank));
                                }
                            }
                        }
                    }
                    cells.push((i, kind, blank, has));
                }
                rows_meta.push((li, cells));
            }
        }
        // drop stale cache entries for edited frames
        self.blank_cache.retain(|_, (rev, _)| *rev > 0);

        {
            let p = ui.painter_at(rows);
            for (display_row, (li, cells)) in rows_meta.iter().enumerate() {
                let y = rows.top() + display_row as f32 * ROW_H - self.tl_scroll_y;
                if y + ROW_H < rows.top() || y > rows.bottom() {
                    continue;
                }
                let layer = &self.doc.layers[*li];
                let row_rect =
                    Rect::from_min_size(Pos2::new(rows.left(), y), Vec2::new(rows.width(), ROW_H));
                let selected_row = *li == self.sel_layer;
                p.rect_filled(
                    row_rect,
                    0.0,
                    if selected_row { Color32::from_rgb(0x31, 0x31, 0x31) } else if display_row % 2 == 0 {
                        theme::TIMELINE_BG
                    } else {
                        theme::ROW_ALT
                    },
                );
                p.line_segment(
                    [row_rect.left_bottom(), row_rect.right_bottom()],
                    Stroke::new(1.0_f32, theme::BORDER),
                );

                for (i, kind, blank, has) in cells {
                    let x = x_of(*i);
                    let cell = Rect::from_min_size(Pos2::new(x, y), Vec2::new(FRAME_W, ROW_H));
                    if cell.right() < rows.left() || cell.left() > rows.right() {
                        continue;
                    }
                    let mut fill = match kind {
                        FrameKind::Key => {
                            if *blank {
                                theme::FRAME_BLANK
                            } else {
                                theme::FRAME_KEY
                            }
                        }
                        FrameKind::Hold => {
                            if *has {
                                theme::FRAME_HOLD
                            } else {
                                theme::FRAME_EMPTY
                            }
                        }
                        FrameKind::Empty => theme::FRAME_EMPTY,
                    };
                    if !layer.visible {
                        fill = fill.gamma_multiply(0.6);
                    }
                    p.rect_filled(cell, 0.0, fill);

                    let selected = selected_row && *i == self.current_frame as i64;
                    if selected {
                        p.rect_filled(cell, 0.0, theme::ACCENT);
                    }

                    // keyframe dot
                    let center = cell.center();
                    match kind {
                        FrameKind::Key => {
                            if *blank {
                                p.circle_stroke(
                                    center,
                                    3.0,
                                    Stroke::new(1.0_f32, if selected { Color32::WHITE } else { Color32::from_gray(0xdf) }),
                                );
                            } else {
                                p.circle_filled(
                                    center,
                                    3.2,
                                    if selected { Color32::WHITE } else { Color32::from_gray(0x1c) },
                                );
                            }
                        }
                        FrameKind::Empty => {}
                        FrameKind::Hold => {}
                    }

                    // separators
                    let sep = if (*i + 1) % 5 == 0 { theme::GRID_STRONG } else { theme::GRID };
                    p.line_segment(
                        [cell.right_top(), cell.right_bottom()],
                        Stroke::new(1.0_f32, sep),
                    );
                    if *kind == FrameKind::Key {
                        p.line_segment(
                            [cell.left_top(), cell.left_bottom()],
                            Stroke::new(1.0_f32, Color32::from_gray(0x22)),
                        );
                    }
                }
            }
        }

        // ---- layer names ---------------------------------------------------
        {
            let p = ui.painter_at(left_rows);
            for (display_row, (li, _)) in rows_meta.iter().enumerate() {
                let y = left_rows.top() + display_row as f32 * ROW_H - self.tl_scroll_y;
                if y + ROW_H < left_rows.top() || y > left_rows.bottom() {
                    continue;
                }
                let (luid, lname, lvisible, llocked) = {
                    let l = &self.doc.layers[*li];
                    (l.uid, l.name.clone(), l.visible, l.locked)
                };
                let row_rect = Rect::from_min_size(
                    Pos2::new(left_rows.left(), y),
                    Vec2::new(left_rows.width(), ROW_H),
                );
                let selected = *li == self.sel_layer;
                p.rect_filled(
                    row_rect,
                    0.0,
                    if selected {
                        Color32::from_rgb(0x3b, 0x3b, 0x3b)
                    } else if display_row % 2 == 0 {
                        theme::PANEL
                    } else {
                        theme::PANEL_ALT
                    },
                );
                if selected {
                    p.rect_filled(
                        Rect::from_min_size(row_rect.min, Vec2::new(2.0, row_rect.height())),
                        0.0,
                        theme::ACCENT,
                    );
                }
                p.line_segment(
                    [row_rect.left_bottom(), row_rect.right_bottom()],
                    Stroke::new(1.0_f32, theme::BORDER),
                );

                // eye
                let eye_rect = Rect::from_center_size(
                    Pos2::new(row_rect.left() + 12.0, row_rect.center().y),
                    Vec2::splat(16.0),
                );
                let eye = ui.interact(
                    eye_rect,
                    ui.id().with(("eye", luid)),
                    Sense::click(),
                );
                if eye.hovered() {
                    p.rect_filled(eye_rect, 2.0, theme::BUTTON_HOVER);
                }
                crate::icons::paint(
                    &p,
                    eye_rect.shrink(3.0),
                    if lvisible { Icon::Eye } else { Icon::EyeOff },
                    if lvisible { theme::TEXT } else { theme::TEXT_FAINT },
                );

                // lock
                let lock_rect = Rect::from_center_size(
                    Pos2::new(row_rect.left() + 30.0, row_rect.center().y),
                    Vec2::splat(16.0),
                );
                let lock = ui.interact(
                    lock_rect,
                    ui.id().with(("lock", luid)),
                    Sense::click(),
                );
                if lock.hovered() {
                    p.rect_filled(lock_rect, 2.0, theme::BUTTON_HOVER);
                }
                crate::icons::paint(
                    &p,
                    lock_rect.shrink(4.0),
                    if llocked { Icon::Lock } else { Icon::Unlock },
                    if llocked { theme::TEXT } else { theme::TEXT_FAINT },
                );

                if eye.on_hover_text("Show / hide layer").clicked() {
                    if let Some(l) = self.doc.layers.get_mut(*li) {
                        l.visible = !l.visible;
                        self.dirty = true;
                    }
                }
                if lock.on_hover_text("Lock / unlock layer").clicked() {
                    if let Some(l) = self.doc.layers.get_mut(*li) {
                        l.locked = !l.locked;
                        self.dirty = true;
                    }
                }

                // name
                let name_rect = Rect::from_min_max(
                    Pos2::new(row_rect.left() + 42.0, row_rect.top()),
                    Pos2::new(row_rect.right() - 4.0, row_rect.bottom()),
                );
                let name_resp =
                    ui.interact(name_rect, ui.id().with(("lname", luid)), Sense::click());
                let color = if !lvisible {
                    theme::TEXT_FAINT
                } else if selected {
                    Color32::WHITE
                } else {
                    theme::TEXT
                };
                p.text(
                    Pos2::new(name_rect.left(), name_rect.center().y),
                    Align2::LEFT_CENTER,
                    &lname,
                    FontId::proportional(11.0),
                    color,
                );
                if selected {
                    p.circle_filled(
                        Pos2::new(row_rect.right() - 10.0, row_rect.center().y),
                        2.5,
                        theme::ACCENT,
                    );
                }
                let resp = name_resp.clone().on_hover_text("Click to select, double-click for layer properties");
                if resp.clicked() {
                    self.sel_layer = *li;
                }
                if resp.double_clicked() {
                    self.sel_layer = *li;
                    self.request(Cmd::LayerProps);
                }

                if name_resp.clicked_by(egui::PointerButton::Secondary) {
                    self.sel_layer = *li;
                }
                name_resp.context_menu(|ui| {
                    if ui.button("Layer properties…").clicked() {
                        self.request(Cmd::LayerProps);
                        ui.close();
                    }
                    if ui.button("Duplicate layer").clicked() {
                        self.request(Cmd::DuplicateLayer);
                        ui.close();
                    }
                    if ui.button("Delete layer").clicked() {
                        self.request(Cmd::DeleteLayer);
                        ui.close();
                    }
                });
            }
        }

        // ---- playhead ------------------------------------------------------
        {
            let p = ui.painter_at(body);
            let x = x_of(self.current_frame as i64) + FRAME_W * 0.5;
            if x >= right.left() - 2.0 && x <= body.right() {
                let color = if self.playing { Color32::from_rgb(0xff, 0x77, 0x77) } else { theme::PLAYHEAD };
                p.line_segment(
                    [Pos2::new(x, ruler.top()), Pos2::new(x, rows.bottom())],
                    Stroke::new(1.5_f32, color),
                );
                let ty = ruler.top() + 2.0;
                p.add(egui::Shape::convex_polygon(
                    vec![
                        Pos2::new(x - 5.0, ty),
                        Pos2::new(x + 5.0, ty),
                        Pos2::new(x, ty + 7.0),
                    ],
                    color,
                    Stroke::NONE,
                ));
            }
        }

        // ---- horizontal scrollbar -----------------------------------------
        if max_scroll_x > 0.0 {
            let track = Rect::from_min_max(
                Pos2::new(rows.left(), rows.bottom() + 2.0),
                Pos2::new(rows.right(), body.bottom() - 1.0),
            );
            ui.painter().rect_filled(track, 2.0, theme::PANEL_HEADER);
            let frac = rows.width() / (total_frames as f32 * FRAME_W);
            let w = (track.width() * frac).max(24.0);
            let t = if max_scroll_x > 0.0 { self.tl_scroll_x / max_scroll_x } else { 0.0 };
            let bar = Rect::from_min_size(
                Pos2::new(track.left() + (track.width() - w) * t, track.top()),
                Vec2::new(w, track.height()),
            );
            let resp = ui.interact(track, ui.id().with("hscroll"), Sense::click_and_drag());
            if resp.dragged() {
                let dx = resp.drag_delta().x;
                self.tl_scroll_x += dx * (max_scroll_x / (track.width() - w).max(1.0));
            }
            ui.painter().rect_filled(bar, 2.0, Color32::from_gray(0x5a));
        }

        // ---- corner filler -------------------------------------------------
        ui.painter().rect_filled(
            Rect::from_min_size(
                Pos2::new(left.left(), rows.bottom()),
                Vec2::new(left.width() + 1.0, body.bottom() - rows.bottom()),
            ),
            0.0,
            theme::TIMELINE_BG,
        );
    }
}
