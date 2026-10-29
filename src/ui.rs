//! Small reusable widgets styled after Adobe panels.

use crate::icons::{self, Icon};
use crate::theme;
use egui::{Color32, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

pub fn panel_frame(fill: Color32) -> egui::Frame {
    egui::Frame::default()
        .fill(fill)
        .inner_margin(egui::Margin::ZERO)
}

/// Vertical sizing for the top bars.
pub fn bar(ui: &mut Ui, height: f32, add: impl FnOnce(&mut Ui)) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.horizontal_centered(|ui| add(ui));
    });
}

pub fn tool_btn(ui: &mut Ui, icon: Icon, active: bool, tip: &str) -> Response {
    let size = Vec2::splat(24.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let bg = if active {
        theme::ACCENT_DARK
    } else if resp.hovered() {
        theme::BUTTON_HOVER
    } else {
        Color32::TRANSPARENT
    };
    if bg != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 2.0, bg);
    }
    if active {
        ui.painter().rect_stroke(
            rect,
            2.0,
            Stroke::new(1.0_f32, theme::ACCENT),
            StrokeKind::Inside,
        );
    }
    let color = if !ui.is_enabled() {
        theme::TEXT_FAINT
    } else if active {
        Color32::WHITE
    } else if resp.hovered() {
        Color32::WHITE
    } else {
        theme::TEXT
    };
    icons::paint(ui.painter(), rect.shrink(5.0), icon, color);
    if tip.is_empty() {
        resp
    } else {
        resp.on_hover_text(tip)
    }
}

pub fn icon_btn(ui: &mut Ui, icon: Icon, tip: &str) -> Response {
    let size = Vec2::splat(22.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, 2.0, theme::BUTTON_HOVER);
    }
    icons::paint(ui.painter(), rect.shrink(5.0), icon, theme::TEXT);
    resp.on_hover_text(tip)
}

pub fn small_icon_btn(ui: &mut Ui, icon: Icon, tip: &str) -> Response {
    let size = Vec2::splat(17.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, 2.0, theme::BUTTON_HOVER);
    }
    icons::paint(ui.painter(), rect.shrink(3.5), icon, theme::TEXT_DIM);
    resp.on_hover_text(tip)
}

pub fn section_title(ui: &mut Ui, text: &str) {
    ui.add_space(3.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 18.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::PANEL_HEADER);
    ui.painter().text(
        rect.left_center() + Vec2::new(6.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(10.5),
        theme::TEXT_DIM,
    );
}

pub fn divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 16.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::BORDER_LIGHT);
}

/// A slider row that keeps the label on the left and the value on the right.
pub fn slider_row(ui: &mut Ui, label: &str, value: &mut f32, range: std::ops::RangeInclusive<f32>, decimals: usize) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [72.0, 18.0],
            egui::Label::new(egui::RichText::new(label).size(11.0).color(theme::TEXT)),
        );
        let slider = egui::Slider::new(value, range)
            .show_value(true)
            .fixed_decimals(decimals)
            .trailing_fill(true);
        ui.add_sized([ui.available_width().max(60.0), 18.0], slider);
    });
}

/// Colour swatch with a border, used for the fill / stroke chips.
pub fn swatch(ui: &mut Ui, color: &mut Color32, size: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    ui.painter()
        .rect_filled(rect, 1.0, *color);
    ui.painter().rect_stroke(
        rect,
        1.0,
        Stroke::new(1.0_f32, Color32::from_gray(0x18)),
        StrokeKind::Inside,
    );
    if resp.hovered() {
        ui.painter().rect_stroke(
            rect,
            1.0,
            Stroke::new(1.0_f32, Color32::from_gray(0x90)),
            StrokeKind::Inside,
        );
    }
    resp
}
