//! Adobe-Animate-like dark theme.

use egui::{Color32, Stroke};

pub const PANEL: Color32 = Color32::from_rgb(0x2b, 0x2b, 0x2b);
pub const PANEL_ALT: Color32 = Color32::from_rgb(0x31, 0x31, 0x31);
pub const PANEL_HEADER: Color32 = Color32::from_rgb(0x24, 0x24, 0x24);
pub const TOOLBAR: Color32 = Color32::from_rgb(0x28, 0x28, 0x28);
pub const BORDER: Color32 = Color32::from_rgb(0x1b, 0x1b, 0x1b);
pub const BORDER_LIGHT: Color32 = Color32::from_rgb(0x3d, 0x3d, 0x3d);
pub const STAGE_BG: Color32 = Color32::from_rgb(0x1f, 0x1f, 0x1f);
pub const TEXT: Color32 = Color32::from_rgb(0xd2, 0xd2, 0xd2);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x8f, 0x8f, 0x8f);
pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x6a, 0x6a, 0x6a);
pub const ACCENT: Color32 = Color32::from_rgb(0x2f, 0x7d, 0xc8);
pub const ACCENT_DARK: Color32 = Color32::from_rgb(0x22, 0x55, 0x86);
pub const PLAYHEAD: Color32 = Color32::from_rgb(0xe8, 0x5a, 0x5a);
pub const FIELD_BG: Color32 = Color32::from_rgb(0x1e, 0x1e, 0x1e);
pub const BUTTON: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);
pub const BUTTON_HOVER: Color32 = Color32::from_rgb(0x45, 0x45, 0x45);
pub const BUTTON_ACTIVE: Color32 = Color32::from_rgb(0x50, 0x50, 0x50);

pub const TIMELINE_BG: Color32 = Color32::from_rgb(0x25, 0x25, 0x25);
pub const ROW_ALT: Color32 = Color32::from_rgb(0x29, 0x29, 0x29);
pub const FRAME_EMPTY: Color32 = Color32::from_rgb(0x2c, 0x2c, 0x2c);
pub const FRAME_BLANK: Color32 = Color32::from_rgb(0x5a, 0x5a, 0x5a);
pub const FRAME_HOLD: Color32 = Color32::from_rgb(0x7c, 0x7c, 0x7c);
pub const FRAME_KEY: Color32 = Color32::from_rgb(0x9c, 0x9c, 0x9c);
pub const GRID: Color32 = Color32::from_rgb(0x33, 0x33, 0x33);
pub const GRID_STRONG: Color32 = Color32::from_rgb(0x45, 0x45, 0x45);
/// Цвет полосы заголовка (совпадает с основным фоном приложения).
pub const TITLE_BAR: egui::Color32 = egui::Color32::from_rgb(0x2b, 0x2b, 0x2b);
/// Цвет текста в заголовке.
pub const TITLE_TEXT: egui::Color32 = egui::Color32::from_rgb(0xe0, 0xe0, 0xe0);

pub fn apply(ctx: &egui::Context) {
    // egui 0.33 выбирает стиль по системной теме (ThemePreference::System по
    // умолчанию). На Windows со светлой темой это даёт светлые чекбоксы, слайдеры
    // и поля ввода поверх тёмных панелей. Приложение тёмное всегда — фиксируем тему
    // ДО чтения ctx.style(), иначе правки уйдут в неактивный dark_style.
    ctx.set_theme(egui::ThemePreference::Dark);

    let mut style = (*ctx.style()).clone();
    let v = &mut style.visuals;
    *v = egui::Visuals::dark();
    v.panel_fill = PANEL;
    v.window_fill = PANEL;
    v.extreme_bg_color = FIELD_BG;
    v.faint_bg_color = PANEL_ALT;
    v.code_bg_color = FIELD_BG;
    v.override_text_color = Some(TEXT);
    v.hyperlink_color = ACCENT;
    v.selection.bg_fill = ACCENT;
    v.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);
    v.widgets.noninteractive.bg_fill = PANEL;
    v.widgets.noninteractive.weak_bg_fill = PANEL;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT_DIM);
    v.widgets.inactive.bg_fill = BUTTON;
    v.widgets.inactive.weak_bg_fill = BUTTON;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(0x22, 0x22, 0x22));
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    v.widgets.hovered.bg_fill = BUTTON_HOVER;
    v.widgets.hovered.weak_bg_fill = BUTTON_HOVER;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(0x5a, 0x5a, 0x5a));
    v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    v.widgets.active.bg_fill = ACCENT;
    v.widgets.active.weak_bg_fill = ACCENT;
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(0x1b, 0x1b, 0x1b));
    v.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    v.widgets.open.bg_fill = BUTTON_ACTIVE;
    v.widgets.open.weak_bg_fill = BUTTON_ACTIVE;

    style.spacing.item_spacing = egui::vec2(5.0, 4.0);
    style.spacing.button_padding = egui::vec2(7.0, 3.0);
    style.spacing.slider_width = 120.0;
    style.spacing.interact_size = egui::vec2(22.0, 20.0);
    style.text_styles = [
        (egui::TextStyle::Small, egui::FontId::proportional(10.5)),
        (egui::TextStyle::Body, egui::FontId::proportional(12.0)),
        (egui::TextStyle::Button, egui::FontId::proportional(12.0)),
        (egui::TextStyle::Heading, egui::FontId::proportional(13.0)),
        (egui::TextStyle::Monospace, egui::FontId::monospace(11.0)),
    ]
    .into();
    ctx.set_style(style);

    let mut visuals = ctx.style().visuals.clone();
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 6],
        blur: 18,
        spread: 0,
        color: Color32::from_black_alpha(120),
    };
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 3],
        blur: 10,
        spread: 0,
        color: Color32::from_black_alpha(140),
    };
    ctx.set_visuals(visuals);
}
