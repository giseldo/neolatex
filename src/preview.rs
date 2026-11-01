use crate::theme;
use egui::{
    pos2, vec2, Align2, Color32, ColorImage, Context, Frame, Margin, Order, Rect, RichText,
    Rounding, Sense, Stroke, TextureHandle, TextureOptions, Ui,
};
use std::path::PathBuf;

pub struct PdfPreview {
    pdfium:    Option<pdfium_render::prelude::Pdfium>,
    pages:     Vec<TextureHandle>,
    page_w:    f32,
    pub zoom:  f32, // 1.0 = fit to pane width
    pub current: usize,
    err:       Option<String>,
}

impl PdfPreview {
    pub fn new() -> Self {
        Self { pdfium: load_pdfium(), pages: Vec::new(), page_w: 1600.0, zoom: 1.0, current: 0, err: None }
    }

    pub fn page_count(&self) -> usize { self.pages.len() }

    pub fn reload(&mut self, path: &PathBuf, ctx: &Context) {
        self.err = None;

        let Some(pdf) = &self.pdfium else {
            self.err = Some(
                "pdfium.dll não encontrado.\n\nBaixe em github.com/bblanchon/pdfium-binaries/releases\ne coloque ao lado do neolatex.exe."
                    .into(),
            );
            return;
        };

        use pdfium_render::prelude::*;

        let doc = match pdf.load_pdf_from_file(path, None) {
            Ok(d)  => d,
            Err(e) => { self.err = Some(format!("Erro ao abrir o PDF: {e}")); return; }
        };

        let cfg = PdfRenderConfig::new().set_target_width(1600).set_maximum_height(2400);

        let mut pages = Vec::new();
        for (i, page) in doc.pages().iter().enumerate() {
            if let Ok(bmp) = page.render_with_config(&cfg) {
                let rgba = bmp.as_image().into_rgba8();
                let (w, h) = rgba.dimensions();
                if i == 0 { self.page_w = w as f32; }
                let ci = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba.as_raw());
                pages.push(ctx.load_texture(format!("p{i}"), ci, TextureOptions::LINEAR));
            }
        }
        self.pages = pages;
        self.current = self.current.min(self.pages.len().saturating_sub(1));
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let panel = ui.max_rect();

        if let Some(e) = &self.err {
            placeholder(ui, "Preview indisponível", e, theme::ERR);
            return;
        }
        if self.pages.is_empty() {
            placeholder(ui, "Nenhum PDF ainda", "Pressione F5 para compilar o documento.", theme::MUTED);
            return;
        }

        // Ctrl + scroll / pinch zoom while hovering the preview
        if ui.rect_contains_pointer(panel) {
            let z = ui.input(|i| i.zoom_delta());
            if z != 1.0 { self.zoom = (self.zoom * z).clamp(0.25, 4.0); }
        }

        let fit   = ((panel.width() - 56.0) / self.page_w).max(0.1);
        let scale = fit * self.zoom;
        let mut current = self.current;

        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            let view = ui.clip_rect();
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                for (i, tex) in self.pages.iter().enumerate() {
                    let [w, h] = tex.size();
                    let (rect, _) = ui.allocate_exact_size(vec2(w as f32, h as f32) * scale, Sense::hover());

                    if ui.is_rect_visible(rect) {
                        let p = ui.painter();
                        p.add(egui::epaint::Shadow {
                            offset: vec2(0.0, 4.0),
                            blur: 18.0,
                            spread: 0.0,
                            color: Color32::from_black_alpha(110),
                        }.as_shape(rect, Rounding::same(2.0)));
                        p.rect_filled(rect, 2.0, Color32::WHITE);
                        p.image(tex.id(), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                    }
                    if rect.top() <= view.center().y { current = i; }
                    ui.add_space(20.0);
                }
            });
        });
        self.current = current;

        self.floating_controls(ui, panel);
    }

    fn floating_controls(&mut self, ui: &mut Ui, panel: Rect) {
        egui::Area::new(egui::Id::new("preview_controls"))
            .order(Order::Foreground)
            .pivot(Align2::CENTER_BOTTOM)
            .fixed_pos(pos2(panel.center().x, panel.bottom() - 16.0))
            .show(ui.ctx(), |ui| {
                Frame::none()
                    .fill(theme::PANEL.gamma_multiply(0.96))
                    .stroke(Stroke::new(1.0_f32, theme::BORDER))
                    .rounding(Rounding::same(18.0))
                    .inner_margin(Margin::symmetric(12.0, 3.0))
                    .shadow(egui::epaint::Shadow {
                        offset: vec2(0.0, 3.0), blur: 12.0, spread: 0.0,
                        color: Color32::from_black_alpha(90),
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{} / {}", self.current + 1, self.pages.len()))
                                    .size(12.0).color(theme::TEXT),
                            );
                            ui.separator();
                            if ui.small_button("−").on_hover_text("Diminuir zoom").clicked() {
                                self.zoom = (self.zoom - 0.15).max(0.25);
                            }
                            ui.label(RichText::new(format!("{:.0}%", self.zoom * 100.0)).size(12.0).color(theme::MUTED));
                            if ui.small_button("+").on_hover_text("Aumentar zoom").clicked() {
                                self.zoom = (self.zoom + 0.15).min(4.0);
                            }
                            if ui.small_button("Ajustar").on_hover_text("Ajustar à largura").clicked() {
                                self.zoom = 1.0;
                            }
                        });
                    });
            });
    }
}

fn placeholder(ui: &mut Ui, title: &str, body: &str, color: Color32) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.38);
            ui.label(RichText::new(title).size(18.0).color(color));
            ui.add_space(6.0);
            ui.label(RichText::new(body).size(13.0).color(theme::MUTED));
        });
    });
}

fn load_pdfium() -> Option<pdfium_render::prelude::Pdfium> {
    use pdfium_render::prelude::*;

    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
    let candidates = exe_dir.into_iter().chain(std::iter::once(PathBuf::from("./")));

    for dir in candidates {
        let lib = Pdfium::pdfium_platform_library_name_at_path(&dir);
        if let Ok(b) = Pdfium::bind_to_library(lib) {
            return Some(Pdfium::new(b));
        }
    }
    Pdfium::bind_to_system_library().ok().map(Pdfium::new)
}
