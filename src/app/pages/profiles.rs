//! Profiles: saved wattages with add / edit / delete.

use crate::app::{th, widgets::*, App};
use crate::core::config::{self, Profile};
use egui::{vec2, Ui};

pub fn show(app: &mut App, ui: &mut Ui) {
    let lang = app.lang;
    let s = app.s();

    // Header with the "new" button.
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{} · {}", s.tab_profiles(lang), app.cfg.profiles.len()))
                .size(12.0)
                .color(th::TEXT_MUTED),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if primary_button(ui, s.prof_add(lang), true).clicked() {
                app.editor_name = format!("{:.0} W", app.cap_w);
                app.editor_w = app.cap_w;
                app.editing = Some(usize::MAX); // MAX = creating
            }
        });
    });
    ui.add_space(10.0);

    if app.cfg.profiles.is_empty() {
        card(ui, |ui| {
            ui.label(
                egui::RichText::new(s.prof_empty(lang))
                    .size(13.0)
                    .color(th::TEXT_MUTED),
            );
        });
        return;
    }

    let profiles = app.cfg.profiles.clone();
    let mut delete_at: Option<usize> = None;
    let mut apply_idx: Option<usize> = None;
    let mut edit_idx: Option<usize> = None;

    for (i, p) in profiles.iter().enumerate() {
        let active = (app.cap_w - p.watts()).abs() < 0.05;
        card(ui, |ui| {
            ui.horizontal(|ui| {
                // Accent dot
                let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), egui::Sense::hover());
                ui.painter().circle_filled(
                    r.center(),
                    4.5,
                    if active { th::ACCENT } else { th::STROKE_HI },
                );
                ui.add_space(4.0);

                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(&p.name)
                            .size(14.0)
                            .color(th::TEXT)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(format!("{:.1} {}", p.watts(), s.unit_w(lang)))
                            .size(11.5)
                            .color(th::TEXT_MUTED),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(egui::RichText::new(s.prof_delete(lang)).color(th::DANGER))
                        .clicked()
                    {
                        delete_at = Some(i);
                    }
                    ui.add_space(4.0);
                    if ui.button(s.prof_save(lang)).clicked() {
                        edit_idx = Some(i);
                    }
                    ui.add_space(4.0);
                    if primary_button(ui, s.cap_apply(lang), true).clicked() {
                        apply_idx = Some(i);
                    }
                });
            });
        });
        ui.add_space(8.0);
    }

    if let Some(i) = delete_at {
        app.cfg.profiles.remove(i);
        let _ = config::save(&app.cfg);
    }
    if let Some(i) = apply_idx {
        app.cap_w = profiles[i].watts();
        app.cap_text = format!("{:.1}", app.cap_w);
        app.apply_cap();
    }
    if let Some(i) = edit_idx {
        app.editor_name = profiles[i].name.clone();
        app.editor_w = profiles[i].watts();
        app.editing = Some(i);
    }

    // ---- editor sheet -------------------------------------------------
    let mut open = app.editing.is_some();
    let mut commit = false;
    let mut cancel = false;

    if open {
        egui::Window::new(s.prof_add(lang))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.set_width(300.0);
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(s.prof_name(lang))
                        .size(11.5)
                        .color(th::TEXT_MUTED),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut app.editor_name).desired_width(f32::INFINITY),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(s.prof_value(lang))
                        .size(11.5)
                        .color(th::TEXT_MUTED),
                );
                ui.add(
                    egui::Slider::new(&mut app.editor_w, 3.0..=60.0)
                        .step_by(0.5)
                        .suffix(format!(" {}", s.unit_w(lang))),
                );
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if primary_button(ui, s.prof_save(lang), true).clicked() {
                        commit = true;
                    }
                    if ui.button(s.prof_cancel(lang)).clicked() {
                        cancel = true;
                    }
                });
            });
    }

    if commit {
        let name = if app.editor_name.trim().is_empty() {
            format!("{:.0} W", app.editor_w)
        } else {
            app.editor_name.trim().to_string()
        };
        let mw = (app.editor_w * 1000.0).round() as u32;
        match app.editing {
            Some(usize::MAX) => app.cfg.profiles.push(Profile { name, mw }),
            Some(i) if i < app.cfg.profiles.len() => {
                app.cfg.profiles[i] = Profile { name, mw };
            }
            _ => {}
        }
        let _ = config::save(&app.cfg);
        app.editing = None;
        open = false;
    }
    if cancel {
        app.editing = None;
        open = false;
    }
    let _ = open;
}