#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod compiler;
mod editor;
mod highlight;
mod preview;
mod theme;

fn load_icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(include_bytes!("../assets/icon.png")).ok()?.into_rgba8();
    let (width, height) = img.dimensions();
    Some(egui::IconData { rgba: img.into_raw(), width, height })
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1440.0, 900.0])
        .with_min_inner_size([900.0, 600.0])
        .with_drag_and_drop(true)
        .with_title("NeoLatex");
    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions { viewport, ..Default::default() };

    eframe::run_native(
        "NeoLatex",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Ok(Box::new(app::NeoLatex::new(cc)))
        }),
    )
}
