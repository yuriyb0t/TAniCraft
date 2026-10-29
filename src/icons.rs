//! Hand-drawn vector icons so the app needs no image assets.

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Selection,
    Transform,
    Lasso,
    Pen,
    Pencil,
    Brush,
    Eraser,
    Bucket,
    Eyedropper,
    Hand,
    Zoom,
    Text,
    Eye,
    EyeOff,
    Lock,
    Unlock,
    Layer,
    Folder,
    Plus,
    Trash,
    Copy,
    ArrowUp,
    ArrowDown,
    Play,
    Pause,
    StepBack,
    StepForward,
    ToStart,
    ToEnd,
    Loop,
    Onion,
    Keyframe,
    BlankKey,
    EmptyFrame,
    ChevronDown,
    ChevronRight,
    Swap,
}

fn p(rect: Rect, x: f32, y: f32) -> Pos2 {
    Pos2::new(rect.left() + rect.width() * x, rect.top() + rect.height() * y)
}

pub fn paint(painter: &Painter, rect: Rect, icon: Icon, color: Color32) {
    let s = rect.width().min(rect.height());
    let w = (s / 14.0).clamp(1.0, 1.8);
    let stroke = Stroke::new(w as f32, color);
    let thin = Stroke::new((w * 0.85).max(1.0), color);

    match icon {
        Icon::Selection => {
            let pts = vec![
                p(rect, 0.28, 0.14),
                p(rect, 0.28, 0.80),
                p(rect, 0.45, 0.63),
                p(rect, 0.57, 0.88),
                p(rect, 0.68, 0.82),
                p(rect, 0.56, 0.58),
                p(rect, 0.76, 0.55),
            ];
            painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
        }
        Icon::Transform => {
            painter.rect_stroke(
                Rect::from_min_max(p(rect, 0.22, 0.22), p(rect, 0.78, 0.78)),
                0,
                thin,
                egui::StrokeKind::Inside,
            );
            for (x, y) in [(0.22, 0.22), (0.78, 0.22), (0.22, 0.78), (0.78, 0.78)] {
                painter.rect_filled(
                    Rect::from_center_size(p(rect, x, y), Vec2::splat(s * 0.14)),
                    0,
                    color,
                );
            }
        }
        Icon::Lasso => {
            painter.add(Shape::ellipse_stroke(
                p(rect, 0.5, 0.42),
                Vec2::new(s * 0.26, s * 0.20),
                stroke,
            ));
            painter.line_segment([p(rect, 0.33, 0.60), p(rect, 0.40, 0.85)], stroke);
            painter.circle_filled(p(rect, 0.42, 0.88), s * 0.05, color);
        }
        Icon::Pen => {
            let pts = vec![p(rect, 0.30, 0.76), p(rect, 0.42, 0.20), p(rect, 0.72, 0.42)];
            painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
            painter.line_segment([p(rect, 0.30, 0.76), p(rect, 0.56, 0.56)], thin);
        }
        Icon::Pencil => {
            painter.line_segment([p(rect, 0.24, 0.76), p(rect, 0.66, 0.22)], Stroke::new(w * 1.6, color));
            painter.line_segment([p(rect, 0.66, 0.22), p(rect, 0.78, 0.32)], stroke);
            painter.line_segment([p(rect, 0.24, 0.76), p(rect, 0.24, 0.86)], stroke);
            painter.line_segment([p(rect, 0.24, 0.86), p(rect, 0.34, 0.80)], stroke);
        }
        Icon::Brush => {
            painter.line_segment([p(rect, 0.70, 0.18), p(rect, 0.54, 0.40)], Stroke::new(w * 2.6, color));
            painter.line_segment([p(rect, 0.50, 0.34), p(rect, 0.66, 0.48)], thin);
            let pts = vec![
                p(rect, 0.52, 0.38),
                p(rect, 0.62, 0.48),
                p(rect, 0.40, 0.72),
                p(rect, 0.22, 0.84),
                p(rect, 0.30, 0.62),
            ];
            painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
        }
        Icon::Eraser => {
            let pts = vec![
                p(rect, 0.24, 0.62),
                p(rect, 0.54, 0.26),
                p(rect, 0.80, 0.46),
                p(rect, 0.52, 0.82),
            ];
            painter.add(Shape::convex_polygon(pts, Color32::TRANSPARENT, stroke));
            painter.line_segment([p(rect, 0.36, 0.72), p(rect, 0.66, 0.36)], thin);
            painter.line_segment([p(rect, 0.22, 0.88), p(rect, 0.78, 0.88)], thin);
        }
        Icon::Bucket => {
            let pts = vec![
                p(rect, 0.22, 0.44),
                p(rect, 0.52, 0.16),
                p(rect, 0.78, 0.46),
                p(rect, 0.50, 0.78),
            ];
            painter.add(Shape::convex_polygon(pts, Color32::TRANSPARENT, stroke));
            painter.line_segment([p(rect, 0.40, 0.28), p(rect, 0.66, 0.56)], thin);
            painter.circle_filled(p(rect, 0.82, 0.72), s * 0.07, color);
        }
        Icon::Eyedropper => {
            painter.line_segment([p(rect, 0.28, 0.80), p(rect, 0.64, 0.30)], Stroke::new(w * 1.6, color));
            painter.line_segment([p(rect, 0.56, 0.22), p(rect, 0.76, 0.40)], Stroke::new(w * 1.4, color));
            painter.line_segment([p(rect, 0.22, 0.84), p(rect, 0.34, 0.86)], thin);
            painter.line_segment([p(rect, 0.24, 0.74), p(rect, 0.30, 0.86)], thin);
        }
        Icon::Hand => {
            painter.circle_stroke(p(rect, 0.50, 0.62), s * 0.22, thin);
            painter.line_segment([p(rect, 0.38, 0.46), p(rect, 0.38, 0.26)], thin);
            painter.line_segment([p(rect, 0.47, 0.42), p(rect, 0.47, 0.20)], thin);
            painter.line_segment([p(rect, 0.56, 0.42), p(rect, 0.56, 0.24)], thin);
            painter.line_segment([p(rect, 0.64, 0.48), p(rect, 0.66, 0.34)], thin);
            painter.line_segment([p(rect, 0.34, 0.74), p(rect, 0.42, 0.94)], thin);
        }
        Icon::Zoom => {
            painter.circle_stroke(p(rect, 0.44, 0.42), s * 0.24, stroke);
            painter.line_segment([p(rect, 0.62, 0.60), p(rect, 0.82, 0.82)], Stroke::new(w * 1.5, color));
            painter.line_segment([p(rect, 0.33, 0.42), p(rect, 0.55, 0.42)], thin);
            painter.line_segment([p(rect, 0.44, 0.31), p(rect, 0.44, 0.53)], thin);
        }
        Icon::Text => {
            painter.line_segment([p(rect, 0.26, 0.24), p(rect, 0.74, 0.24)], Stroke::new(w * 1.4, color));
            painter.line_segment([p(rect, 0.50, 0.24), p(rect, 0.50, 0.80)], Stroke::new(w * 1.4, color));
            painter.line_segment([p(rect, 0.36, 0.80), p(rect, 0.64, 0.80)], thin);
        }
        Icon::Eye | Icon::EyeOff => {
            let pts: Vec<Pos2> = (0..=16)
                .map(|i| {
                    let t = i as f32 / 16.0;
                    let x = 0.14 + 0.72 * t;
                    let y = 0.48 + (t * std::f32::consts::PI).sin() * -0.16;
                    p(rect, x, y)
                })
                .collect();
            painter.add(Shape::line(pts, thin));
            let pts: Vec<Pos2> = (0..=16)
                .map(|i| {
                    let t = i as f32 / 16.0;
                    let x = 0.14 + 0.72 * t;
                    let y = 0.48 + (t * std::f32::consts::PI).sin() * 0.16;
                    p(rect, x, y)
                })
                .collect();
            painter.add(Shape::line(pts, thin));
            painter.circle_filled(p(rect, 0.5, 0.48), s * 0.10, color);
            if icon == Icon::EyeOff {
                painter.line_segment([p(rect, 0.18, 0.78), p(rect, 0.82, 0.20)], Stroke::new(w * 1.2, color));
            }
        }
        Icon::Lock | Icon::Unlock => {
            painter.rect_filled(Rect::from_min_max(p(rect, 0.28, 0.46), p(rect, 0.72, 0.80)), 1.0, color);
            let pts: Vec<Pos2> = (0..=12)
                .map(|i| {
                    let t = i as f32 / 12.0;
                    let ang = std::f32::consts::PI * (1.0 + t);
                    let cx = if icon == Icon::Unlock { 0.58 } else { 0.50 };
                    p(rect, cx + 0.17 * ang.cos(), 0.46 + 0.20 * ang.sin())
                })
                .collect();
            painter.add(Shape::line(pts, thin));
        }
        Icon::Layer | Icon::Folder => {
            let pts = vec![
                p(rect, 0.18, 0.30),
                p(rect, 0.42, 0.30),
                p(rect, 0.48, 0.38),
                p(rect, 0.82, 0.38),
                p(rect, 0.82, 0.76),
                p(rect, 0.18, 0.76),
            ];
            painter.add(Shape::closed_line(pts, thin));
        }
        Icon::Plus => {
            painter.line_segment([p(rect, 0.5, 0.22), p(rect, 0.5, 0.78)], stroke);
            painter.line_segment([p(rect, 0.22, 0.5), p(rect, 0.78, 0.5)], stroke);
        }
        Icon::Trash => {
            painter.rect_stroke(
                Rect::from_min_max(p(rect, 0.30, 0.34), p(rect, 0.70, 0.82)),
                1.0,
                thin,
                egui::StrokeKind::Inside,
            );
            painter.line_segment([p(rect, 0.22, 0.28), p(rect, 0.78, 0.28)], stroke);
            painter.line_segment([p(rect, 0.42, 0.20), p(rect, 0.58, 0.20)], thin);
            painter.line_segment([p(rect, 0.44, 0.44), p(rect, 0.44, 0.72)], thin);
            painter.line_segment([p(rect, 0.56, 0.44), p(rect, 0.56, 0.72)], thin);
        }
        Icon::Copy => {
            painter.rect_stroke(
                Rect::from_min_max(p(rect, 0.30, 0.18), p(rect, 0.68, 0.58)),
                1.0,
                thin,
                egui::StrokeKind::Inside,
            );
            painter.rect_stroke(
                Rect::from_min_max(p(rect, 0.42, 0.38), p(rect, 0.80, 0.78)),
                1.0,
                thin,
                egui::StrokeKind::Inside,
            );
        }
        Icon::ArrowUp => {
            painter.line_segment([p(rect, 0.5, 0.78), p(rect, 0.5, 0.24)], stroke);
            painter.line_segment([p(rect, 0.5, 0.24), p(rect, 0.30, 0.44)], stroke);
            painter.line_segment([p(rect, 0.5, 0.24), p(rect, 0.70, 0.44)], stroke);
        }
        Icon::ArrowDown => {
            painter.line_segment([p(rect, 0.5, 0.22), p(rect, 0.5, 0.76)], stroke);
            painter.line_segment([p(rect, 0.5, 0.76), p(rect, 0.30, 0.56)], stroke);
            painter.line_segment([p(rect, 0.5, 0.76), p(rect, 0.70, 0.56)], stroke);
        }
        Icon::Play => {
            let pts = vec![p(rect, 0.30, 0.20), p(rect, 0.80, 0.50), p(rect, 0.30, 0.80)];
            painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
        }
        Icon::Pause => {
            painter.rect_filled(Rect::from_min_max(p(rect, 0.30, 0.22), p(rect, 0.44, 0.78)), 0.0, color);
            painter.rect_filled(Rect::from_min_max(p(rect, 0.56, 0.22), p(rect, 0.70, 0.78)), 0.0, color);
        }
        Icon::StepBack | Icon::StepForward => {
            let fwd = icon == Icon::StepForward;
            let sign = if fwd { 1.0 } else { -1.0 };
            let cx = if fwd { 0.40 } else { 0.60 };
            let pts = vec![
                p(rect, cx, 0.24),
                p(rect, cx + sign * 0.30, 0.50),
                p(rect, cx, 0.76),
            ];
            painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
            let bx = if fwd { 0.68 } else { 0.32 };
            painter.rect_filled(
                Rect::from_min_max(p(rect, bx - 0.04, 0.24), p(rect, bx + 0.04, 0.76)),
                0.0,
                color,
            );
        }
        Icon::ToStart | Icon::ToEnd => {
            let sign = if icon == Icon::ToEnd { 1.0 } else { -1.0 };
            for k in 0..2 {
                let cx = 0.34 + k as f32 * 0.24;
                let pts = vec![
                    p(rect, cx, 0.26),
                    p(rect, cx + sign * 0.24, 0.50),
                    p(rect, cx, 0.74),
                ];
                painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
            }
            let bx = if icon == Icon::ToEnd { 0.80 } else { 0.20 };
            painter.rect_filled(
                Rect::from_min_max(p(rect, bx - 0.03, 0.26), p(rect, bx + 0.03, 0.74)),
                0.0,
                color,
            );
        }
        Icon::Loop => {
            painter.add(Shape::ellipse_stroke(
                p(rect, 0.50, 0.52),
                Vec2::new(s * 0.28, s * 0.20),
                thin,
            ));
            painter.line_segment([p(rect, 0.66, 0.28), p(rect, 0.78, 0.34)], thin);
            painter.line_segment([p(rect, 0.66, 0.40), p(rect, 0.78, 0.34)], thin);
        }
        Icon::Onion => {
            painter.circle_stroke(p(rect, 0.40, 0.50), s * 0.18, Stroke::new(w as f32, color.gamma_multiply(0.55)));
            painter.circle_stroke(p(rect, 0.60, 0.50), s * 0.18, Stroke::new(w as f32, color.gamma_multiply(0.8)));
            painter.circle_filled(p(rect, 0.50, 0.50), s * 0.05, color);
        }
        Icon::Keyframe => {
            painter.circle_filled(p(rect, 0.50, 0.50), s * 0.20, color);
        }
        Icon::BlankKey => {
            painter.circle_stroke(p(rect, 0.50, 0.50), s * 0.18, Stroke::new(w as f32, color));
        }
        Icon::EmptyFrame => {
            painter.circle_stroke(p(rect, 0.50, 0.50), s * 0.12, Stroke::new(w * 0.8, color.gamma_multiply(0.6)));
        }
        Icon::ChevronDown => {
            painter.line_segment([p(rect, 0.32, 0.42), p(rect, 0.50, 0.60)], thin);
            painter.line_segment([p(rect, 0.50, 0.60), p(rect, 0.68, 0.42)], thin);
        }
        Icon::ChevronRight => {
            painter.line_segment([p(rect, 0.42, 0.32), p(rect, 0.60, 0.50)], thin);
            painter.line_segment([p(rect, 0.60, 0.50), p(rect, 0.42, 0.68)], thin);
        }
        Icon::Swap => {
            painter.line_segment([p(rect, 0.22, 0.36), p(rect, 0.78, 0.36)], thin);
            painter.line_segment([p(rect, 0.78, 0.36), p(rect, 0.64, 0.24)], thin);
            painter.line_segment([p(rect, 0.78, 0.64), p(rect, 0.22, 0.64)], thin);
            painter.line_segment([p(rect, 0.22, 0.64), p(rect, 0.36, 0.76)], thin);
        }
    }
}

/// Small icon rendered inside a button-sized rect.
pub fn icon_widget(ui: &mut egui::Ui, icon: Icon, size: Vec2) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let color = if resp.hovered() {
        Color32::WHITE
    } else if ui.is_enabled() {
        crate::theme::TEXT
    } else {
        crate::theme::TEXT_FAINT
    };
    paint(ui.painter(), rect.shrink(size.x * 0.16), icon, color);
    resp
}

/// Icon button with an active (pressed) state.
pub fn toggle_icon(ui: &mut egui::Ui, icon: Icon, active: bool, size: Vec2) -> egui::Response {
    let resp = icon_widget(ui, icon, size);
    if active {
        ui.painter().rect_stroke(
            resp.rect.shrink(1.0),
            2.0,
            Stroke::new(1.0_f32, crate::theme::ACCENT),
            egui::StrokeKind::Inside,
        );
        paint(
            ui.painter(),
            resp.rect.shrink(size.x * 0.16),
            icon,
            crate::theme::ACCENT,
        );
    }
    resp
}
