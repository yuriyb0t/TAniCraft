//! TAniCraft — a raster animation studio in the spirit of Adobe Animate.

mod app;
mod canvas;
mod dialogs;
mod doc;
mod export;
mod icons;
mod import;
mod theme;
mod timeline;
mod tools;
mod ui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1480.0, 930.0])
            .with_min_inner_size([1040.0, 680.0])
            .with_title("TAniCraft"),
        ..Default::default()
    };
    eframe::run_native(
        "TAniCraft",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
