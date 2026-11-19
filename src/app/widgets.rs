//! Reusable presentation pieces: cards, the ring gauge, sparkline chart and
//! the left navigation rail.

use egui::{
    Align2, Color32, CornerRadius, FontFamily, FontId, Rect, Response, Sense, Stroke, StrokeKind,
    Ui, Vec2,
};

use super::theme::*;

/// A bordered surface panel.
pub fn card(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::Frame::NONE
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, STROKE))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(egui::Margin::same(16))
        .show(ui, add);
}

/// Big numeric readout: value + unit, with the unit de-emphasised.
pub fn readout(ui: &mut Ui, value: &str, unit: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        ui.label(
            egui::RichText::new(value)
                .font(FontId::new(34.0, FontFamily::Proportional))
                .color(color)
                .strong(),
        );
        ui.add_space(1.0);
        ui.vertical(|ui| {
            ui.add_space(11.0);
            // Long units like "MHz" must not wrap to one character per line.
            ui.add(
                egui::Label::new(egui::RichText::new(unit).size(13.0).color(TEXT_MUTED))
                    .wrap_mode(egui::TextWrapMode::Extend),
            );
        });
    });
}

/// A ring gauge. `t` is 0..1 fill, drawn clockwise from 12 o'clock.
pub fn ring(
    ui: &mut Ui,
    size: f32,
    t: f32,
    color: Color32,
    center_text: &str,
    center_unit: &str,
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let p = ui.painter();
    let c = rect.center();
    let r = size * 0.5 - 8.0;
    let width = 9.0;

    // Track.
    p.circle_stroke(c, r, Stroke::new(width, alpha(STROKE, 200)));

    let t = t.clamp(0.0, 1.0);
    if t > 0.001 {
        let start = -std::f32::consts::FRAC_PI_2;
        let sweep = std::f32::consts::TAU * t;
        let steps = ((t * 96.0) as usize).max(2);
        let mut pts = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let a = start + sweep * (i as f32 / steps as f32);
            pts.push(egui::pos2(c.x + r * a.cos(), c.y + r * a.sin()));
        }
        p.add(egui::Shape::line(
            pts,
            egui::epaint::PathStroke::new(width, color),
        ));
        // Leading dot for a touch of polish.
        let a = start + sweep;
        p.circle_filled(
            egui::pos2(c.x + r * a.cos(), c.y + r * a.sin()),
            width * 0.55,
            color,
        );
    }

    p.text(
        egui::pos2(c.x, c.y - 6.0),
        Align2::CENTER_CENTER,
        center_text,
        FontId::new(size * 0.26, FontFamily::Proportional),
        TEXT,
    );
    p.text(
        egui::pos2(c.x, c.y + size * 0.15),
        Align2::CENTER_CENTER,
        center_unit,
        FontId::new(size * 0.10, FontFamily::Proportional),
        TEXT_MUTED,
    );
    resp
}

/// Two-series sparkline: power (accent) and temperature (amber), auto-scaled.
/// `cap` lets the power axis stay stable around the configured wattage instead
/// of rescaling on every sample.
pub fn chart(ui: &mut Ui, height: f32, power: &[f32], temp: &[f32], cap: f32) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let p = ui.painter_at(rect);

    p.rect_filled(rect, CornerRadius::same(10), alpha(SURFACE_HI, 120));

    // Horizontal guides.
    for i in 1..4 {
        let y = rect.top() + rect.height() * (i as f32 / 4.0);
        p.line_segment(
            [
                egui::pos2(rect.left() + 1.0, y),
                egui::pos2(rect.right() - 1.0, y),
            ],
            Stroke::new(1.0, alpha(STROKE, 120)),
        );
    }

    // Power axis: round up to a tidy step a bit above both the peak and the cap,
    // so the curve sits comfortably inside the plot.
    let peak = power.iter().cloned().fold(0.0f32, f32::max).max(cap);
    let pmax = if peak <= 10.0 {
        15.0
    } else if peak <= 20.0 {
        25.0
    } else if peak <= 30.0 {
        40.0
    } else if peak <= 45.0 {
        55.0
    } else if peak <= 60.0 {
        70.0
    } else {
        (peak * 1.2).ceil()
    };
    // Temperature axis: fixed window so the amber line does not jitter.
    let tmin = 30.0f32;
    let tmax = 95.0f32;

    let pad_l = 26.0;
    let inner = Rect::from_min_max(
        egui::pos2(rect.left() + pad_l, rect.top() + 8.0),
        egui::pos2(rect.right() - 30.0, rect.bottom() - 8.0),
    );

    // Axis labels: watts on the left, degrees on the right.
    let font = FontId::new(9.5, FontFamily::Proportional);
    for (frac, val) in [(1.0, pmax), (0.5, pmax / 2.0), (0.0, 0.0)] {
        let y = inner.bottom() - inner.height() * frac;
        p.text(
            egui::pos2(rect.left() + 5.0, y),
            Align2::LEFT_CENTER,
            format!("{val:.0}"),
            font.clone(),
            alpha(ACCENT, 150),
        );
    }
    for (frac, val) in [(1.0, tmax), (0.5, (tmin + tmax) / 2.0), (0.0, tmin)] {
        let y = inner.bottom() - inner.height() * frac;
        p.text(
            egui::pos2(rect.right() - 5.0, y),
            Align2::RIGHT_CENTER,
            format!("{val:.0}"),
            font.clone(),
            alpha(WARN, 150),
        );
    }

    if power.len() < 2 {
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "· · ·",
            FontId::new(18.0, FontFamily::Proportional),
            TEXT_MUTED,
        );
        return;
    }

    let plot = |vals: &[f32], lo: f32, hi: f32| -> Vec<egui::Pos2> {
        let n = vals.len().max(2);
        let span = (hi - lo).max(0.001);
        vals.iter()
            .enumerate()
            .map(|(i, &v)| {
                let x = inner.left() + inner.width() * (i as f32 / (n - 1) as f32);
                let t = ((v - lo) / span).clamp(0.0, 1.0);
                egui::pos2(x, inner.bottom() - inner.height() * t)
            })
            .collect()
    };

    // Filled area under the power curve.
    let pw = plot(power, 0.0, pmax);
    let mut poly = pw.clone();
    poly.push(egui::pos2(inner.right(), inner.bottom()));
    poly.push(egui::pos2(inner.left(), inner.bottom()));
    p.add(egui::Shape::convex_polygon(
        poly,
        alpha(ACCENT, 26),
        Stroke::NONE,
    ));
    p.add(egui::Shape::line(
        pw,
        egui::epaint::PathStroke::new(1.8, ACCENT),
    ));

    let tw = plot(temp, tmin, tmax);
    p.add(egui::Shape::line(
        tw,
        egui::epaint::PathStroke::new(1.5, alpha(WARN, 220)),
    ));
}

/// Sidebar entry. Returns true when clicked.
pub fn nav_item(ui: &mut Ui, selected: bool, label: &str, hint: &str) -> bool {
    let h = 42.0;
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), h),
        Sense::click(),
    );
    let hovered = resp.hovered();

    if selected {
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(10),
            alpha(ACCENT_DEEP, 70),
        );
        // Accent bar on the left edge.
        let bar = Rect::from_min_size(
            egui::pos2(rect.left() + 1.0, rect.center().y - 9.0),
            Vec2::new(3.0, 18.0),
        );
        ui.painter()
            .rect_filled(bar, CornerRadius::same(2), ACCENT);
    } else if hovered {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(10), alpha(SURFACE_HI, 150));
    }

    let col = if selected {
        TEXT
    } else if hovered {
        TEXT
    } else {
        TEXT_DIM
    };
    ui.painter().text(
        egui::pos2(rect.left() + 16.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::new(14.0, FontFamily::Proportional),
        col,
    );
    if !hint.is_empty() {
        ui.painter().text(
            egui::pos2(rect.right() - 10.0, rect.center().y),
            Align2::RIGHT_CENTER,
            hint,
            FontId::new(11.0, FontFamily::Proportional),
            TEXT_MUTED,
        );
    }
    resp.clicked()
}

/// Small pill badge, e.g. driver status.
pub fn pill(ui: &mut Ui, text: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        FontId::new(11.5, FontFamily::Proportional),
        color,
    );
    let pad = Vec2::new(9.0, 4.0);
    let (rect, _) = ui.allocate_exact_size(galley.size() + pad * 2.0, Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(8), alpha(color, 34));
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(1.0, alpha(color, 90)),
        StrokeKind::Inside,
    );
    ui.painter().galley(
        egui::pos2(rect.left() + pad.x, rect.top() + pad.y),
        galley,
        color,
    );
}

/// Accent-filled primary button.
pub fn primary_button(ui: &mut Ui, text: &str, enabled: bool) -> Response {
    let h = 34.0;
    let w = (text.chars().count() as f32 * 9.0 + 44.0).max(96.0);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), sense);
    let hovered = resp.hovered() && enabled;

    let fill = if !enabled {
        alpha(SURFACE_HI, 120)
    } else if hovered {
        mix(ACCENT_DEEP, ACCENT, 0.35)
    } else {
        ACCENT_DEEP
    };
    ui.painter().rect_filled(rect, CornerRadius::same(10), fill);
    if enabled {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(10),
            Stroke::new(1.0, alpha(ACCENT, if hovered { 200 } else { 120 })),
            StrokeKind::Inside,
        );
    }
    let col = if enabled { TEXT } else { TEXT_MUTED };
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        FontId::new(14.0, FontFamily::Proportional),
        col,
    );
    if enabled && hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// Thin separator line.
pub fn divider(ui: &mut Ui) {
    ui.add_space(2.0);
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 1.0),
        Sense::hover(),
    );
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, alpha(STROKE, 200));
    ui.add_space(2.0);
}

/// Key/value row used on the Hardware page.
pub fn kv(ui: &mut Ui, key: &str, val: &str) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [150.0, 18.0],
            egui::Label::new(egui::RichText::new(key).size(13.0).color(TEXT_MUTED)),
        );
        ui.label(egui::RichText::new(val).size(13.0).color(TEXT));
    });
}