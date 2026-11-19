//! Visual identity: a refined dark "graphite + cyan" theme.
//! All colours live here so the rest of the UI never hard-codes a hex value.

use egui::{Color32, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle, Visuals};

pub const BG: Color32 = Color32::from_rgb(0x0B, 0x0E, 0x14);
pub const RAIL: Color32 = Color32::from_rgb(0x0E, 0x12, 0x1A);
pub const SURFACE: Color32 = Color32::from_rgb(0x14, 0x19, 0x24);
pub const SURFACE_HI: Color32 = Color32::from_rgb(0x1B, 0x22, 0x31);
pub const STROKE: Color32 = Color32::from_rgb(0x23, 0x2B, 0x3A);
pub const STROKE_HI: Color32 = Color32::from_rgb(0x31, 0x3C, 0x50);

pub const TEXT: Color32 = Color32::from_rgb(0xE7, 0xEC, 0xF5);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x9A, 0xA6, 0xBA);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x67, 0x74, 0x8B);

pub const ACCENT: Color32 = Color32::from_rgb(0x22, 0xD3, 0xEE);
pub const ACCENT_DEEP: Color32 = Color32::from_rgb(0x0E, 0x74, 0x90);
pub const INDIGO: Color32 = Color32::from_rgb(0x81, 0x8C, 0xF8);
pub const OK: Color32 = Color32::from_rgb(0x34, 0xD3, 0x99);
pub const WARN: Color32 = Color32::from_rgb(0xF5, 0x9E, 0x0B);
pub const DANGER: Color32 = Color32::from_rgb(0xF4, 0x3F, 0x5E);

/// Linear blend, used for gauges and the chart gradient.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_unmultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
        255,
    )
}

pub fn alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

/// Colour for a temperature reading: cool → amber → red.
pub fn temp_color(c: f32) -> Color32 {
    if c < 60.0 {
        ACCENT
    } else if c < 75.0 {
        mix(ACCENT, WARN, (c - 60.0) / 15.0)
    } else if c < 90.0 {
        mix(WARN, DANGER, (c - 75.0) / 15.0)
    } else {
        DANGER
    }
}

pub fn install(ctx: &egui::Context) {
    install_fonts(ctx);

    ctx.set_theme(egui::ThemePreference::Dark);

    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();

    let mut v = Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = SURFACE;
    v.extreme_bg_color = Color32::from_rgb(0x08, 0x0B, 0x10);
    v.faint_bg_color = SURFACE_HI;
    v.override_text_color = Some(TEXT);
    v.hyperlink_color = ACCENT;
    v.window_stroke = Stroke::new(1.0, STROKE);
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(10);
    v.selection.bg_fill = alpha(ACCENT, 60);
    v.selection.stroke = Stroke::new(1.0, ACCENT);

    // Widgets ---------------------------------------------------------
    v.widgets.noninteractive.bg_fill = SURFACE;
    v.widgets.noninteractive.weak_bg_fill = SURFACE;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, STROKE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
    v.widgets.noninteractive.corner_radius = CornerRadius::same(10);

    v.widgets.inactive.bg_fill = SURFACE_HI;
    v.widgets.inactive.weak_bg_fill = SURFACE_HI;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, STROKE);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.inactive.corner_radius = CornerRadius::same(10);

    v.widgets.hovered.bg_fill = mix(SURFACE_HI, STROKE_HI, 0.6);
    v.widgets.hovered.weak_bg_fill = mix(SURFACE_HI, STROKE_HI, 0.6);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, STROKE_HI);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.hovered.corner_radius = CornerRadius::same(10);

    v.widgets.active.bg_fill = mix(SURFACE_HI, ACCENT_DEEP, 0.45);
    v.widgets.active.weak_bg_fill = mix(SURFACE_HI, ACCENT_DEEP, 0.45);
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_DEEP);
    v.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.active.corner_radius = CornerRadius::same(10);

    v.widgets.open.bg_fill = SURFACE_HI;
    v.widgets.open.weak_bg_fill = SURFACE_HI;
    v.widgets.open.bg_stroke = Stroke::new(1.0, STROKE_HI);
    v.widgets.open.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.open.corner_radius = CornerRadius::same(10);

    style.visuals = v;

    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 7.0);
    style.spacing.slider_width = 240.0;
    style.spacing.interact_size.y = 30.0;
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.floating = true;
    style.spacing.window_margin = Margin::same(14);

    style.text_styles = [
        (TextStyle::Heading, FontId::new(21.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(14.5, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(13.5, FontFamily::Monospace)),
    ]
    .into();

    ctx.set_style_of(egui::Theme::Dark, style);
}

/// Append a system CJK font as a *fallback* so Latin text keeps egui's crisp
/// default face while Chinese resolves to a real glyph set.
fn install_fonts(ctx: &egui::Context) {
    const CANDIDATES: &[&str] = &[
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\Deng.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
    ];
    let mut fonts = egui::FontDefinitions::default();

    for path in CANDIDATES {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        let name = "cjk".to_owned();
        // .ttc collections need index 0; plain .ttf ignores it.
        fonts.font_data.insert(
            name.clone(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        for fam in [FontFamily::Proportional, FontFamily::Monospace] {
            fonts
                .families
                .entry(fam)
                .or_default()
                .push(name.clone());
        }
        break;
    }

    ctx.set_fonts(fonts);
}