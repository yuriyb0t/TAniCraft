//! Stage rendering and painting input.

use egui::{Color32, Context, Event, Key, Pos2, Rect, Sense, Stroke as EStroke, Ui, Vec2};

use crate::app::App;
use crate::doc::{Bitmap, DirtyRect, FrameKind};
use crate::theme;
use crate::tools::Tool;

pub struct Stroke {
    pub layer_uid: u64,
    pub frame: usize,
    pub last: Pos2,
    pub smoothed: Pos2,
    pub before: Bitmap,
    pub dirty: Option<DirtyRect>,
    pub erase: bool,
    pub spacing: f32,
    pub residual: f32,
    pub alpha: f32,
    pub radius: f32,
    pub hardness: f32,
    pub color: [f32; 3],
    pub smoothing: f32,
    pub stamps: usize,
}

fn checkerboard(painter: &egui::Painter, rect: Rect) {
    let cell = 10.0;
    painter.rect_filled(rect, 0.0, Color32::from_gray(0xc8));
    let cols = (rect.width() / cell).ceil() as i32;
    let rows = (rect.height() / cell).ceil() as i32;
    for y in 0..rows {
        for x in 0..cols {
            if (x + y) % 2 == 0 {
                continue;
            }
            let min = rect.min + Vec2::new(x as f32 * cell, y as f32 * cell);
            let r = Rect::from_min_size(min, Vec2::splat(cell)).intersect(rect);
            painter.rect_filled(r, 0.0, Color32::from_gray(0x9a));
        }
    }
}

impl App {
    pub fn doc_rect(&self) -> Rect {
        let size = Vec2::new(self.doc.width as f32, self.doc.height as f32) * self.view.zoom;
        Rect::from_center_size(self.stage_rect.center() + self.view.offset, size)
    }

    pub fn zoom_at(&mut self, screen_pos: Pos2, factor: f32) {
        let old = self.view.zoom;
        let new = (old * factor).clamp(0.02, 40.0);
        if (new - old).abs() < 1.0e-7 {
            return;
        }
        let doc_size = Vec2::new(self.doc.width as f32, self.doc.height as f32);
        let center = self.stage_rect.center() + self.view.offset;
        let doc_min = center - doc_size * old * 0.5;
        let doc_pt = (screen_pos - doc_min) / old;
        let new_center = screen_pos - doc_pt * new + doc_size * new * 0.5;
        self.view.offset = new_center - self.stage_rect.center();
        self.view.zoom = new;
        self.view.fitted = false;
    }

    pub fn stage_ui(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let rect = ui.available_rect_before_wrap();
        self.stage_rect = rect;
        if !self.view.fitted {
            self.fit_view();
        }
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, theme::STAGE_BG);

        let doc_rect = self.doc_rect();
        let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));

        // drop shadow + canvas
        painter.rect_filled(
            Rect::from_min_max(doc_rect.min + Vec2::new(0.0, 3.0), doc_rect.max + Vec2::new(0.0, 3.0)),
            0.0,
            Color32::from_black_alpha(110),
        );
        let bg = Color32::from_rgba_unmultiplied(
            self.doc.bg[0],
            self.doc.bg[1],
            self.doc.bg[2],
            self.doc.bg[3],
        );
        if self.doc.bg[3] < 255 {
            checkerboard(&painter, doc_rect);
        }
        painter.rect_filled(doc_rect, 0.0, bg);

        // ---- onion skin ---------------------------------------------------
        if self.onion.enabled {
            let mut items: Vec<(u64, usize, Bitmap, f32)> = Vec::new();
            let before = self.onion.before.min(20);
            let after = self.onion.after.min(20);
            for layer in &self.doc.layers {
                if !layer.visible {
                    continue;
                }
                let own = layer.effective_key(self.current_frame);
                let mut found = 0usize;
                let mut f = self.current_frame;
                while found < before && f > 0 {
                    f -= 1;
                    if layer.kind_at(f) == FrameKind::Key && Some(f) != own {
                        if let Some(b) = layer.bitmap_at(f) {
                            let t = (found + 1) as f32 / (before + 1) as f32;
                            items.push((layer.uid, f, b.clone(), self.onion.alpha * (1.0 - t * 0.6)));
                        }
                        found += 1;
                    }
                }
                let mut found = 0usize;
                let mut f = self.current_frame;
                while found < after && f + 1 < layer.frames.len() {
                    f += 1;
                    if layer.kind_at(f) == FrameKind::Key && Some(f) != own {
                        if let Some(b) = layer.bitmap_at(f) {
                            let t = (found + 1) as f32 / (after + 1) as f32;
                            items.push((layer.uid, f, b.clone(), self.onion.alpha * (1.0 - t * 0.6)));
                        }
                        found += 1;
                    }
                }
            }
            for (uid, f, bmp, a) in items {
                let tex = self.texture_for(&ctx, uid, f, &bmp);
                painter.image(
                    tex.id(),
                    doc_rect,
                    uv,
                    Color32::from_white_alpha((a.clamp(0.0, 1.0) * 255.0) as u8),
                );
            }
        }

        // ---- layers -------------------------------------------------------
        let items: Vec<(u64, usize, Bitmap)> = self
            .doc
            .layers
            .iter()
            .filter(|l| l.visible)
            .filter_map(|l| {
                l.effective_key(self.current_frame).and_then(|k| {
                    l.frames[k]
                        .bitmap
                        .clone()
                        .map(|b| (l.uid, k, b))
                })
            })
            .collect();
        for (uid, k, bmp) in items {
            let tex = self.texture_for(&ctx, uid, k, &bmp);
            painter.image(tex.id(), doc_rect, uv, Color32::WHITE);
        }

        painter.rect_stroke(
            doc_rect,
            0.0,
            EStroke::new(1.0_f32, Color32::from_gray(0x10)),
            egui::StrokeKind::Outside,
        );

        // ---- input --------------------------------------------------------
        let pointer = ui.input(|i| i.pointer.hover_pos());
        let primary_pressed = ui.input(|i| i.pointer.primary_pressed());
        let primary_down = ui.input(|i| i.pointer.primary_down());
        let middle_down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle));
        let middle_pressed = ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Middle));
        let alt = ui.input(|i| i.modifiers.alt);
        let space = ui.input(|i| i.key_down(Key::Space));
        let hovered = response.hovered();

        let pan_mode = self.tool == Tool::Hand || space || middle_down;

        // panning
        if (pan_mode && (response.dragged() || middle_down)) && self.stroke.is_none() {
            let delta = ui.input(|i| i.pointer.delta());
            self.view.offset += delta;
            self.view.fitted = false;
            ctx.set_cursor_icon(if middle_down || response.dragged() {
                egui::CursorIcon::Grabbing
            } else {
                egui::CursorIcon::Grab
            });
        }

        // zooming
        if hovered || self.stroke.is_some() {
            let (scroll, ctrl) = ui.input(|i| (i.smooth_scroll_delta, i.modifiers.command));
            if let Some(p) = pointer {
                if ctrl && scroll.y.abs() > 0.1 {
                    self.zoom_at(p, (scroll.y * 0.0035).exp());
                } else if !ctrl && scroll != Vec2::ZERO && self.stroke.is_none() {
                    self.view.offset += scroll;
                    self.view.fitted = false;
                }
            }
        }

        // tool cursor
        if hovered {
            match self.tool {
                Tool::Brush | Tool::Eraser => ctx.set_cursor_icon(egui::CursorIcon::Crosshair),
                Tool::Hand => ctx.set_cursor_icon(egui::CursorIcon::Grab),
                Tool::Zoom => ctx.set_cursor_icon(egui::CursorIcon::ZoomIn),
            }
        }

        if self.tool == Tool::Zoom && hovered && primary_pressed {
            if let Some(p) = pointer {
                self.zoom_at(p, if alt { 0.5 } else { 2.0 });
            }
        }

        let painting_tool = matches!(self.tool, Tool::Brush | Tool::Eraser);
        if painting_tool && !pan_mode {
            if let Some(p) = pointer {
                // draw the brush outline (size preview)
                let size = match self.tool {
                    Tool::Brush => self.brush.size,
                    _ => self.eraser.size,
                };
                let r = (size * 0.5 * self.view.zoom).max(1.5);
                painter.circle_stroke(p, r, EStroke::new(1.0_f32, Color32::from_black_alpha(160)));
                painter.circle_stroke(p, r + 1.0, EStroke::new(1.0_f32, Color32::from_white_alpha(150)));
                if r < 3.0 {
                    painter.circle_filled(p, 1.0, Color32::from_white_alpha(200));
                }
            }

            if primary_pressed && hovered {
                if let Some(p) = pointer {
                    self.begin_stroke(p, doc_rect);
                }
            }

            if self.stroke.is_some() && primary_down {
                let mut pts: Vec<Pos2> = ui.input(|i| {
                    i.events
                        .iter()
                        .filter_map(|e| match e {
                            Event::PointerMoved(p) => Some(*p),
                            _ => None,
                        })
                        .collect()
                });
                if pts.is_empty() {
                    if let Some(p) = pointer {
                        pts.push(p);
                    }
                }
                if !pts.is_empty() {
                    self.feed_points(&pts, doc_rect);
                }
                ctx.request_repaint();
            }

            if self.stroke.is_some() && !primary_down {
                self.finish_stroke();
            }
        }

        if !primary_down && self.stroke.is_some() {
            self.finish_stroke();
        }

        let _ = middle_pressed;
    }

    fn begin_stroke(&mut self, screen: Pos2, doc_rect: Rect) {
        if !doc_rect.contains(screen) {
            return;
        }
        self.playing = false;
        let li = self.sel_layer;
        if li >= self.doc.layers.len() {
            return;
        }
        if self.doc.layers[li].locked {
            self.notify("Layer is locked — unlock it in the Layers panel");
            return;
        }
        if !self.doc.layers[li].visible {
            self.notify("Layer is hidden");
            return;
        }
        let frame = self.current_frame;
        if self.doc.layers[li].kind_at(frame) != FrameKind::Key {
            let before = self.doc.clone();
            if !self.doc.ensure_keyframe(li, frame) {
                return;
            }
            self.commit(before);
            self.notify("Created a keyframe (drawing always edits a keyframe)");
        }
        let Some(bmp) = self.doc.layers[li].frames[frame].bitmap.clone() else {
            return;
        };
        let uid = self.doc.layers[li].uid;
        let doc_pos = doc_rect.min + (screen - doc_rect.min) / self.view.zoom;

        let (radius, hardness, alpha, color, erase, spacing) = match self.tool {
            Tool::Eraser => (
                (self.eraser.size * 0.5).max(0.5),
                self.eraser.hardness,
                self.eraser.opacity,
                [0.0, 0.0, 0.0],
                true,
                (self.eraser.size * 0.10).max(0.7),
            ),
            _ => {
                let c = self.brush.color.to_srgba_unmultiplied();
                let alpha = self.brush.opacity * (c[3] as f32 / 255.0);
                (
                    (self.brush.size * 0.5).max(0.5),
                    self.brush.hardness,
                    alpha,
                    [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0],
                    false,
                    (self.brush.size * 0.10).max(0.7),
                )
            }
        };

        let mut stroke = Stroke {
            layer_uid: uid,
            frame,
            last: doc_pos,
            smoothed: doc_pos,
            before: bmp,
            dirty: None,
            erase,
            spacing,
            residual: 0.0,
            alpha,
            radius,
            hardness,
            color,
            smoothing: self.brush.smoothing,
            stamps: 0,
        };
        let d = self.stamp_dab(&mut stroke, doc_pos);
        stroke.dirty = union(stroke.dirty, d);
        stroke.stamps += 1;
        self.stroke = Some(stroke);
        self.flush_patch();
    }

    fn stamp_dab(&mut self, s: &mut Stroke, pos: Pos2) -> Option<DirtyRect> {
        let li = self.doc.layer_index(s.layer_uid)?;
        let frame = self.doc.layers[li].frames.get_mut(s.frame)?;
        let bmp = frame.key_bitmap_mut()?;
        let d = bmp.stamp(
            pos.x,
            pos.y,
            s.radius,
            s.hardness,
            s.color,
            s.alpha,
            s.erase,
        );
        if d.is_some() {
            s.stamps += 1;
        }
        d
    }

    fn feed_points(&mut self, points: &[Pos2], doc_rect: Rect) {
        let Some(mut s) = self.stroke.take() else {
            return;
        };
        let k = (1.0 - s.smoothing * 0.92).clamp(0.12, 1.0);
        let mut dirty = s.dirty;
        for screen in points {
            let raw = doc_rect.min + (*screen - doc_rect.min) / self.view.zoom;
            s.smoothed += (raw - s.smoothed) * k;
            let target = s.smoothed;
            let dist = s.last.distance(target);
            if dist <= 0.001 {
                continue;
            }
            let dir = (target - s.last) / dist;
            let mut travelled = 0.0f32;
            loop {
                let need = s.spacing - s.residual;
                if travelled + need > dist {
                    s.residual += dist - travelled;
                    break;
                }
                travelled += need;
                s.residual = 0.0;
                let pos = s.last + dir * travelled;
                let d = self.stamp_dab(&mut s, pos);
                dirty = union(dirty, d);
            }
            s.last = target;
        }
        s.dirty = dirty;
        self.stroke = Some(s);
        self.flush_patch();
    }

    fn flush_patch(&mut self) {
        let Some(s) = &self.stroke else {
            return;
        };
        let Some(d) = s.dirty else {
            return;
        };
        let uid = s.layer_uid;
        let frame = s.frame;
        let li = match self.doc.layer_index(uid) {
            Some(v) => v,
            None => return,
        };
        if let Some(bmp) = self.doc.layers[li].frames.get(frame).and_then(|f| f.bitmap.as_ref()) {
            let bmp = bmp.clone();
            self.patch_texture(uid, frame, &bmp, d);
        }
    }

    fn finish_stroke(&mut self) {
        let Some(s) = self.stroke.take() else {
            return;
        };
        let li = match self.doc.layer_index(s.layer_uid) {
            Some(v) => v,
            None => return,
        };
        if let Some(after) = self
            .doc
            .layers
            .get(li)
            .and_then(|l| l.frames.get(s.frame))
            .and_then(|f| f.bitmap.clone())
        {
            if s.stamps > 0 {
                self.push_pixels(s.layer_uid, s.frame, s.before, after);
            }
        }
    }
}

fn union(a: Option<DirtyRect>, b: Option<DirtyRect>) -> Option<DirtyRect> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.union(y)),
        (Some(x), None) => Some(x),
        (None, y) => y,
    }
}
