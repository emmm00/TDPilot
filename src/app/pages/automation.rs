//! Automation: logon auto-apply, backed by the Startup folder.

use crate::app::{th, widgets::*, App};
use crate::core::{autostart, config};
use egui::Ui;

pub fn show(app: &mut App, ui: &mut Ui) {
    let lang = app.lang;
    let s = app.s();
    let enabled = autostart::is_enabled();

    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(s.auto_enable(lang))
                        .size(14.5)
                        .color(th::TEXT)
                        .strong(),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(s.auto_desc(lang))
                        .size(11.5)
                        .color(th::TEXT_MUTED),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if enabled {
                    pill(ui, s.auto_active(lang), th::OK);
                } else {
                    pill(ui, s.auto_inactive(lang), th::TEXT_MUTED);
                }
            });
        });

        ui.add_space(14.0);
        divider(ui);
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(s.auto_value(lang))
                    .size(12.5)
                    .color(th::TEXT_DIM),
            );
            ui.add_space(8.0);
            let mut w = app.cfg.watts();
            if ui
                .add(
                    egui::Slider::new(&mut w, 3.0..=60.0)
                        .step_by(0.5)
                        .suffix(format!(" {}", s.unit_w(lang))),
                )
                .changed()
            {
                app.cfg.set_watts(w);
                config::write_limit_mw(app.cfg.power_mw);
                let _ = config::save(&app.cfg);
            }
        });

        ui.add_space(12.0);

        ui.horizontal(|ui| {
            if enabled {
                if ui.button(s.auto_uninstall(lang)).clicked() {
                    match autostart::disable() {
                        Ok(()) => app.toast(s.msg_ok(lang), th::OK),
                        Err(e) => app.toast(e, th::DANGER),
                    }
                }
            } else {
                if primary_button(ui, s.auto_install(lang), true).clicked() {
                    let exe = std::env::current_exe().unwrap_or_default();
                    match autostart::enable(&exe) {
                        Ok(()) => app.toast(s.msg_ok(lang), th::OK),
                        Err(e) => app.toast(e, th::DANGER),
                    }
                }
            }
            ui.add_space(8.0);
            if ui.button(s.auto_restore(lang)).clicked() {
                app.apply_cap();
            }
        });

        ui.add_space(10.0);
        ui.label(
            egui::RichText::new(s.auto_note(lang))
                .size(11.0)
                .color(th::TEXT_MUTED),
        );
    });

    ui.add_space(12.0);

    card(ui, |ui| {
        ui.label(
            egui::RichText::new(s.set_driverrestart(lang))
                .size(12.5)
                .color(th::TEXT_MUTED)
                .strong(),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(s.set_drvhint(lang))
                .size(11.0)
                .color(th::TEXT_MUTED),
        );
        ui.add_space(10.0);
        if ui.button(s.set_driverrestart(lang)).clicked() {
            crate::core::backend::restart_driver();
            app.request_poll();
            app.toast(s.msg_ok(lang), th::OK);
        }
    });
}