use crate::{highlight, theme};
use egui::{pos2, Align2, FontId, Galley, Rect, Shape, Stroke, TextEdit, Ui};
use std::sync::Arc;

pub struct Output {
    pub changed: bool,
    pub line:    usize,
    pub col:     usize,
}

type Cached = Arc<(String, u32, u32, Arc<Galley>)>;

pub fn show(ui: &mut Ui, text: &mut String, font_size: f32) -> Output {
    let font_id = FontId::monospace(font_size);
    let (char_w, row_h) = ui.fonts(|f| (f.glyph_width(&font_id, '0'), f.row_height(&font_id)));

    let digits   = text.bytes().filter(|&c| c == b'\n').count().saturating_add(1).to_string().len().max(3);
    let gutter_w = char_w * digits as f32 + 30.0;
    let ppp_bits = ui.ctx().pixels_per_point().to_bits();
    let size_bits = font_size.to_bits();
    let cache_id = egui::Id::new("highlight_cache");

    // Re-highlighting only when text or size changes keeps typing snappy.
    let mut layouter = |ui: &Ui, s: &str, _wrap: f32| -> Arc<Galley> {
        if let Some(c) = ui.data(|d| d.get_temp::<Cached>(cache_id)) {
            if c.1 == size_bits && c.2 == ppp_bits && c.0 == s {
                return c.3.clone();
            }
        }
        let galley = ui.fonts(|f| f.layout_job(highlight::highlight(s, font_size)));
        ui.data_mut(|d| {
            d.insert_temp::<Cached>(cache_id, Arc::new((s.to_owned(), size_bits, ppp_bits, galley.clone())))
        });
        galley
    };

    let mut out = Output { changed: false, line: 1, col: 1 };

    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        let clip = ui.clip_rect();
        let hl_slot = ui.painter().add(Shape::Noop);

        let rows_to_fill = (clip.height() / row_h).ceil() as usize + 1;
        let res = ui
            .horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.add_space(gutter_w + 10.0);
                TextEdit::multiline(text)
                    .font(font_id.clone())
                    .frame(false)
                    .desired_width(f32::INFINITY)
                    .desired_rows(rows_to_fill)
                    .lock_focus(true)
                    .layouter(&mut layouter)
                    .show(ui)
            })
            .inner;

        out.changed = res.response.changed();

        let cur_row = res.cursor_range.map(|c| {
            out.line = c.primary.rcursor.row + 1;
            out.col  = c.primary.rcursor.column + 1;
            c.primary.rcursor.row
        });

        // current-line highlight (painted behind the text via the reserved slot)
        if let Some(r) = cur_row.and_then(|r| res.galley.rows.get(r)) {
            let y = res.galley_pos.y + r.rect.min.y;
            let rect = Rect::from_min_max(
                pos2(clip.left() + gutter_w, y),
                pos2(clip.right(), y + r.rect.height()),
            );
            ui.painter().set(hl_slot, Shape::rect_filled(rect, 0.0, theme::LINE_HL));
        }

        // sticky gutter: painted on top, pinned to the viewport's left edge
        let painter = ui.painter_at(clip);
        let gutter  = Rect::from_min_max(clip.left_top(), pos2(clip.left() + gutter_w, clip.bottom()));
        painter.rect_filled(gutter, 0.0, theme::BG);
        painter.vline(gutter.right(), clip.y_range(), Stroke::new(1.0_f32, theme::BORDER));

        for (i, row) in res.galley.rows.iter().enumerate() {
            let y = res.galley_pos.y + row.rect.min.y;
            if y + row.rect.height() < clip.top() { continue; }
            if y > clip.bottom() { break; }
            let color = if Some(i) == cur_row { theme::TEXT } else { theme::MUTED.gamma_multiply(0.7) };
            painter.text(
                pos2(gutter.right() - 12.0, y),
                Align2::RIGHT_TOP,
                (i + 1).to_string(),
                font_id.clone(),
                color,
            );
        }
    });

    out
}
