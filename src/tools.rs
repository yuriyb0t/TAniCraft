//! Tools and brush settings.

use egui::Color32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Brush,
    Eraser,
    Eyedropper,
    Hand,
    Zoom,
}

impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Tool::Brush => "Brush",
            Tool::Eraser => "Eraser",
            Tool::Eyedropper => "Eyedropper",
            Tool::Hand => "Hand",
            Tool::Zoom => "Zoom",
        }
    }

    pub fn letter(self) -> &'static str {
        match self {
            Tool::Brush => "B",
            Tool::Eraser => "E",
            Tool::Eyedropper => "I",
            Tool::Hand => "H",
            Tool::Zoom => "Z",
        }
    }
}

#[derive(Clone, Debug)]
pub struct BrushSettings {
    /// Diameter in document pixels.
    pub size: f32,
    pub hardness: f32,
    pub opacity: f32,
    pub smoothing: f32,
    pub color: Color32,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            size: 12.0,
            hardness: 0.85,
            opacity: 1.0,
            smoothing: 0.35,
            color: Color32::from_rgb(0x1a, 0x1a, 0x1a),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EraserSettings {
    pub size: f32,
    pub hardness: f32,
    pub opacity: f32,
}

impl Default for EraserSettings {
    fn default() -> Self {
        Self { size: 24.0, hardness: 0.9, opacity: 1.0 }
    }
}

#[derive(Clone, Debug)]
pub struct OnionSettings {
    pub enabled: bool,
    pub before: usize,
    pub after: usize,
    pub alpha: f32,
}

impl Default for OnionSettings {
    fn default() -> Self {
        Self { enabled: false, before: 2, after: 1, alpha: 0.35 }
    }
}
