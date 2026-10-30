//! Modal dialogs: new document, document properties, layer properties, about.

use egui::{Color32, Context, RichText};

use crate::app::{App, Cmd, DocSetup};
use crate::theme;
use crate::ui;

impl App {
    pub fn dialogs(&mut self, ctx: &Context) {
        self.new_document_dialog(ctx);
        self.doc_props_dialog(ctx);
        self.layer_props_dialog(ctx);
        self.unsaved_dialog(ctx);
        self.about_dialog(ctx);
        self.fill_picker_dialog(ctx);
        self.error_dialog(ctx);
    }

    fn error_dialog(&mut self, ctx: &Context) {
        let Some(msg) = self.error_msg.clone() else {
            return;
        };
        let mut open = true;
        let mut close = false;
        egui::Window::new("Error")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .frame(ui::panel_frame(theme::PANEL).inner_margin(egui::Margin::same(12)))
            .show(ctx, |ui| {
                ui.set_width(400.0);
                ui.label(RichText::new(&msg).size(11.0).color(theme::TEXT));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("OK").clicked() {
                            close = true;
                        }
                    });
                });
            });
        if !open || close {
            self.error_msg = None;
        }
    }

    fn new_document_dialog(&mut self, ctx: &Context) {
        if !self.new_doc_open {
            return;
        }
        let mut open = true;
        let mut create = false;
        let mut setup = self.new_doc.clone();
        egui::Modal::new(egui::Id::new("new_document"))
            .frame(
                egui::Frame::default()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::same(14))
                    .stroke(egui::Stroke::new(1.0_f32, theme::BORDER)),
            )
            .show(ctx, |ui| {
                ui.set_width(360.0);
                ui.label(RichText::new("New Document").size(14.0).color(theme::TEXT));
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                egui::Grid::new("new_doc_grid")
                    .num_columns(2)
                    .spacing(egui::vec2(10.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("Name").size(11.0).color(theme::TEXT_DIM));
                        ui.add(
                            egui::TextEdit::singleline(&mut setup.name)
                                .desired_width(230.0),
                        );
                        ui.end_row();

                        ui.label(RichText::new("Width").size(11.0).color(theme::TEXT_DIM));
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::new(&mut setup.width)
                                    .speed(1.0)
                                    .range(16..=8192)
                                    .suffix(" px"),
                            );
                            ui.label(RichText::new("Height").size(11.0).color(theme::TEXT_DIM));
                            ui.add(
                                egui::DragValue::new(&mut setup.height)
                                    .speed(1.0)
                                    .range(16..=8192)
                                    .suffix(" px"),
                            );
                        });
                        ui.end_row();

                        ui.label(RichText::new("Preset").size(11.0).color(theme::TEXT_DIM));
                        ui.horizontal_wrapped(|ui| {
                            for (w, h, label) in [
                                (1920u32, 1080u32, "1920×1080"),
                                (1280, 720, "1280×720"),
                                (1080, 1080, "1080×1080"),
                                (800, 600, "800×600"),
                            ] {
                                if ui.small_button(label).clicked() {
                                    setup.width = w;
                                    setup.height = h;
                                }
                            }
                        });
                        ui.end_row();

                        ui.label(RichText::new("Frame rate").size(11.0).color(theme::TEXT_DIM));
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::new(&mut setup.fps)
                                    .speed(1.0)
                                    .range(1.0..=120.0)
                                    .fixed_decimals(1)
                                    .suffix(" fps"),
                            );
                            ui.label(
                                RichText::new("frames per second")
                                    .size(10.0)
                                    .color(theme::TEXT_FAINT),
                            );
                        });
                        ui.end_row();

                        ui.label(RichText::new("Background").size(11.0).color(theme::TEXT_DIM));
                        ui.horizontal(|ui| {
                            ui.color_edit_button_srgba(&mut setup.bg);
                            let c = setup.bg.to_srgba_unmultiplied();
                            ui.label(
                                RichText::new(format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2]))
                                    .size(10.5)
                                    .color(theme::TEXT_FAINT)
                                    .monospace(),
                            );
                        });
                        ui.end_row();
                    });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            open = false;
                        }
                        if ui
                            .add(egui::Button::new(RichText::new("Create").strong()))
                            .clicked()
                        {
                            create = true;
                        }
                    });
                });
            });

        if create {
            self.untitled_counter += 1;
            self.new_document(&setup);
            self.new_doc_open = false;
            self.notify(format!("Created {}", self.doc.name));
        } else if !open {
            self.new_doc_open = false;
        }
    }

    fn doc_props_dialog(&mut self, ctx: &Context) {
        let Some(mut setup) = self.doc_props.clone() else {
            return;
        };
        let mut open = true;
        let mut apply = false;
        egui::Window::new("Document Properties")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .frame(ui::panel_frame(theme::PANEL).inner_margin(egui::Margin::same(12)))
            .show(ctx, |ui| {
                ui.set_width(330.0);
                egui::Grid::new("props_grid")
                    .num_columns(2)
                    .spacing(egui::vec2(10.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("Title").size(11.0).color(theme::TEXT_DIM));
                        ui.add(egui::TextEdit::singleline(&mut setup.name).desired_width(210.0));
                        ui.end_row();
                        ui.label(RichText::new("Width").size(11.0).color(theme::TEXT_DIM));
                        ui.add(
                            egui::DragValue::new(&mut setup.width)
                                .speed(1.0)
                                .range(16..=8192)
                                .suffix(" px"),
                        );
                        ui.end_row();
                        ui.label(RichText::new("Height").size(11.0).color(theme::TEXT_DIM));
                        ui.add(
                            egui::DragValue::new(&mut setup.height)
                                .speed(1.0)
                                .range(16..=8192)
                                .suffix(" px"),
                        );
                        ui.end_row();
                        ui.label(RichText::new("Frame rate").size(11.0).color(theme::TEXT_DIM));
                        ui.add(
                            egui::DragValue::new(&mut setup.fps)
                                .speed(1.0)
                                .range(1.0..=120.0)
                                .fixed_decimals(1)
                                .suffix(" fps"),
                        );
                        ui.end_row();
                        ui.label(RichText::new("Background").size(11.0).color(theme::TEXT_DIM));
                        ui.color_edit_button_srgba(&mut setup.bg);
                        ui.end_row();
                    });
                let changed_size = setup.width != self.doc.width || setup.height != self.doc.height;
                if changed_size {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Resizing keeps the artwork in place (no scaling).")
                            .size(10.0)
                            .color(theme::TEXT_FAINT),
                    );
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            self.doc_props = None;
                        }
                        if ui.button("OK").clicked() {
                            apply = true;
                        }
                    });
                });
            });
        if apply {
            let before = self.doc.clone();
            let bg = setup.bg.to_srgba_unmultiplied();
            self.doc.name = setup.name.clone();
            self.doc.fps = setup.fps.clamp(1.0, 120.0);
            self.doc.bg = bg;
            if setup.width != self.doc.width || setup.height != self.doc.height {
                self.doc.resize_canvas(setup.width.max(16), setup.height.max(16));
                self.fit_view();
            }
            self.commit(before);
            self.doc_props = None;
            self.notify("Document properties updated");
        }
        if !open {
            self.doc_props = None;
        }
    }

    fn layer_props_dialog(&mut self, ctx: &Context) {
        let Some(idx) = self.layer_props else {
            return;
        };
        if idx >= self.doc.layers.len() {
            self.layer_props = None;
            return;
        }
        let mut open = true;
        let mut name = self.doc.layers[idx].name.clone();
        let mut visible = self.doc.layers[idx].visible;
        let mut locked = self.doc.layers[idx].locked;
        let mut apply = false;
        egui::Window::new("Layer Properties")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .frame(ui::panel_frame(theme::PANEL).inner_margin(egui::Margin::same(12)))
            .show(ctx, |ui| {
                ui.set_width(280.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Name").size(11.0).color(theme::TEXT_DIM));
                    ui.add(egui::TextEdit::singleline(&mut name).desired_width(180.0));
                });
                ui.add_space(4.0);
                ui.checkbox(&mut visible, "Visible");
                ui.checkbox(&mut locked, "Locked");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            self.layer_props = None;
                        }
                        if ui.button("OK").clicked() {
                            apply = true;
                        }
                    });
                });
            });
        if apply {
            let before = self.doc.clone();
            if let Some(layer) = self.doc.layers.get_mut(idx) {
                layer.name = name;
                layer.visible = visible;
                layer.locked = locked;
            }
            self.commit(before);
            self.layer_props = None;
        }
        if !open {
            self.layer_props = None;
        }
    }

    fn unsaved_dialog(&mut self, ctx: &Context) {
        if !self.unsaved_open {
            return;
        }
        let mut save = false;
        let mut discard = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::new("unsaved"))
            .frame(
                egui::Frame::default()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::same(14))
                    .stroke(egui::Stroke::new(1.0_f32, theme::BORDER)),
            )
            .show(ctx, |ui| {
                ui.set_width(360.0);
                ui.label(RichText::new("Save changes?").size(13.0).color(theme::TEXT));
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!(
                        "“{}” has unsaved changes that will be lost.",
                        self.doc.name
                    ))
                    .size(11.0)
                    .color(theme::TEXT_DIM),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            cancel = true;
                        }
                        if ui.button("Don’t Save").clicked() {
                            discard = true;
                        }
                        if ui
                            .add(egui::Button::new(RichText::new("Save").strong()))
                            .clicked()
                        {
                            save = true;
                        }
                    });
                });
            });
        if save {
            if self.save(false) {
                self.unsaved_open = false;
                if let Some(cmd) = self.pending.take() {
                    self.exec(cmd);
                }
            }
        } else if discard {
            self.unsaved_open = false;
            // «Don’t Save» всё равно должен выполнить отложенную команду
            // (New/Open/CloseAllProjects/Exit), а не молча её терять. Флаг `dirty` сбрасываем
            // при выходе и закрытии проектов: документ уничтожается. Для
            // New/Open документ ещё может остаться (если пользователь отменит
            // следующий диалог), и изменения обязаны остаться несохранёнными.
            let cmd = self.pending.take();
            if discard_clears_dirty(cmd) {
                self.dirty = false;
            }
            if let Some(cmd) = cmd {
                self.exec(cmd);
            }
        } else if cancel {
            self.unsaved_open = false;
            self.pending = None;
        }
    }

    fn about_dialog(&mut self, ctx: &Context) {
        if !self.about_open {
            return;
        }
        let mut open = true;
        egui::Window::new("About TAniCraft")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .frame(ui::panel_frame(theme::PANEL).inner_margin(egui::Margin::same(14)))
            .show(ctx, |ui| {
                ui.set_width(380.0);
                ui.label(RichText::new("TAniCraft 0.1").size(16.0).strong());
                ui.label(
                    RichText::new("A raster animation studio inspired by Adobe Animate.")
                        .size(11.0)
                        .color(theme::TEXT_DIM),
                );
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);
                for line in [
                    "B — Brush        E — Eraser",
                    "H — Hand          Z — Zoom",
                    "[ / ] — brush size",
                    "F5 — insert frame, F6 — keyframe, F7 — blank keyframe",
                    "Shift+F5 — remove frame, Shift+F6 — clear keyframe",
                    "Enter — play / stop, ← → — step frames",
                    "Ctrl+Z / Ctrl+Shift+Z — undo / redo",
                    "Ctrl+S — save, Ctrl+O — open, Ctrl+N — new",
                    "Ctrl+wheel — zoom at cursor, Space+drag — pan",
                    "Alt+wheel — scrub the timeline",
                ] {
                    ui.label(RichText::new(line).size(10.5).color(theme::TEXT));
                }
            });
        if !open {
            self.about_open = false;
        }
    }

    fn fill_picker_dialog(&mut self, ctx: &Context) {
        if !self.show_fill_picker {
            return;
        }
        let mut open = true;
        egui::Window::new("Fill Colour")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .frame(ui::panel_frame(theme::PANEL).inner_margin(egui::Margin::same(10)))
            .show(ctx, |ui| {
                let mut c = self.brush.color;
                egui::color_picker::color_picker_color32(
                    ui,
                    &mut c,
                    egui::color_picker::Alpha::Opaque,
                );
                if c != self.brush.color {
                    self.brush.color = c;
                }
            });
        if !open {
            self.show_fill_picker = false;
        }
    }
}

impl DocSetup {
    pub fn bg_color32(&self) -> Color32 {
        self.bg
    }
}

/// Сбрасывать ли `dirty` сразу по нажатию «Don’t Save».
///
/// Для выхода и закрытия проектов документ уничтожается. Для New/Open он
/// ещё жив, пока команда действительно не заменит его.
fn discard_clears_dirty(pending: Option<Cmd>) -> bool {
    matches!(pending, Some(Cmd::Exit | Cmd::CloseAllProjects))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discard_clears_dirty_for_exit_or_close() {
        assert!(discard_clears_dirty(Some(Cmd::Exit)));
        assert!(discard_clears_dirty(Some(Cmd::CloseAllProjects)));
        assert!(!discard_clears_dirty(Some(Cmd::New)));
        assert!(!discard_clears_dirty(Some(Cmd::Open)));
        assert!(!discard_clears_dirty(None));
    }
}
