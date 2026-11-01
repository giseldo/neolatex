use egui::{
    Color32, Context, FontData, FontDefinitions, FontFamily, Rounding, Stroke, Visuals,
};

pub const BG:         Color32 = Color32::from_rgb(26, 27, 31);   // editor
pub const PANEL:      Color32 = Color32::from_rgb(34, 36, 41);   // bars
pub const BORDER:     Color32 = Color32::from_rgb(52, 54, 60);
pub const PREVIEW_BG: Color32 = Color32::from_rgb(44, 46, 52);
pub const TEXT:       Color32 = Color32::from_rgb(200, 203, 211);
pub const MUTED:      Color32 = Color32::from_rgb(125, 130, 142);
pub const ACCENT:     Color32 = Color32::from_rgb(88, 139, 255);
pub const OK:         Color32 = Color32::from_rgb(98, 200, 120);
pub const WARN:       Color32 = Color32::from_rgb(229, 192, 100);
pub const ERR:        Color32 = Color32::from_rgb(235, 100, 100);
pub const LINE_HL:    Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 9);

pub fn install(ctx: &Context) {
    install_fonts(ctx);

    let mut v = Visuals::dark();
    v.panel_fill       = PANEL;
    v.extreme_bg_color = BG;
    v.window_fill      = PANEL;
    v.window_stroke    = Stroke::new(1.0_f32, BORDER);
    v.window_rounding  = Rounding::same(10.0);
    v.menu_rounding    = Rounding::same(8.0);

    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT);

    let rounding = Rounding::same(6.0);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(48, 51, 58);
    v.widgets.inactive.bg_fill      = Color32::from_rgb(48, 51, 58);
    v.widgets.inactive.fg_stroke    = Stroke::new(1.0_f32, TEXT);
    v.widgets.hovered.weak_bg_fill  = Color32::from_rgb(62, 66, 75);
    v.widgets.hovered.bg_fill       = Color32::from_rgb(62, 66, 75);
    v.widgets.hovered.bg_stroke     = Stroke::NONE;
    v.widgets.active.weak_bg_fill   = Color32::from_rgb(76, 81, 92);
    v.widgets.active.bg_fill        = Color32::from_rgb(76, 81, 92);
    v.widgets.inactive.bg_stroke    = Stroke::NONE;
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.rounding = rounding;
    }

    v.selection.bg_fill = Color32::from_rgba_premultiplied(30, 52, 98, 110);
    v.selection.stroke  = Stroke::new(1.0_f32, ACCENT);
    v.text_cursor.stroke = Stroke::new(2.0_f32, ACCENT);

    ctx.set_visuals(v);

    let mut s = (*ctx.style()).clone();
    s.spacing.item_spacing   = egui::vec2(8.0, 6.0);
    s.spacing.button_padding = egui::vec2(10.0, 5.0);
    s.spacing.interact_size.y = 26.0;
    ctx.set_style(s);
}

fn install_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    let mut add = |name: &str, path: &str, family: FontFamily| {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert(name.to_owned(), FontData::from_owned(bytes));
            if let Some(list) = fonts.families.get_mut(&family) {
                list.insert(0, name.to_owned());
            }
        }
    };

    add("segoe-ui", r"C:\Windows\Fonts\segoeui.ttf", FontFamily::Proportional);
    // First match wins at index 0, so add the less-preferred mono font first.
    add("consolas", r"C:\Windows\Fonts\consola.ttf", FontFamily::Monospace);
    add("cascadia", r"C:\Windows\Fonts\CascadiaMono.ttf", FontFamily::Monospace);

    ctx.set_fonts(fonts);
}
