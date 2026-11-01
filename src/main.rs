#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod compiler;
mod editor;
mod highlight;
mod preview;
mod theme;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([900.0, 600.0])
            .with_drag_and_drop(true)
            .with_title("NeoLatex"),
        ..Default::default()
    };

    eframe::run_native(
        "NeoLatex",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Ok(Box::new(app::NeoLatex::new(cc)))
        }),
    )
}
