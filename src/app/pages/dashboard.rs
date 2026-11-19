//! Dashboard: the ring gauge, live metrics, the trend chart and the cap slider.

use crate::app::{theme as th, widgets::*, App};
use crate::core::backend::DriverState;
use egui::{vec2, Ui};

pub fn show(app: &mut App, ui: &mut Ui) {
    let lang = app.lang;
    let s = app.s();

    // ---------------- hero row ----------------
    ui.horizontal_top(|ui| {
        // Ring gauge card.
        let gauge_w = 268.0;
        ui.allocate_ui_with_layout(
            vec2(gauge_w, 236.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                card(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        let snap = &app.snap;
                        let power = snap.power_w();
                        let cap = if snap.limit_w() > 1.0 {
                            snap.limit_w()
                        } else {
                            app.cap_w
                        };
                        let t = if cap > 1.0 { power / cap } else { 0.0 };
                        let col = if power > cap && cap > 1.0 {
                            th::DANGER
                        } else {
                            th::ACCENT
                        };
                        ring(
                            ui,
                            168.0,
                            t,
                            col,
                            &format!("{power:.1}"),
                            s.unit_w(lang),
                        );
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "{} {cap:.0} {}",
                                s.card_limit(lang),
                                s.unit_w(lang)
                            ))
                            .size(12.0)
                            .color(th::TEXT_MUTED),
                        );
                    });
                });
            },
        );

        ui.add_space(12.0);

        // Metric tiles.
        ui.vertical(|ui| {
            let snap = &app.snap;
            let tiles: Vec<(&str, String, String, egui::Color32)> = vec![
                (
                    s.card_power(lang),
                    format!("{:.1}", snap.power_w()),
                    s.unit_w(lang).into(),
                    th::ACCENT,
                ),
                (
                    s.card_temp(lang),
                    format!("{:.0}", snap.tctl_temp_c),
                    s.unit_c(lang).into(),
                    th::temp_color(snap.tctl_temp_c),
                ),
                (
                    s.card_clock(lang),
                    format!("{:.0}", snap.cclk_mhz),
                    s.unit_mhz(lang).into(),
                    th::INDIGO,
                ),
            ];

            ui.horizontal_wrapped(|ui| {
                for (title, val, unit, col) in tiles {
                    let w = ((ui.available_width() - 24.0) / 3.0).max(120.0);
                    ui.allocate_ui_with_layout(
                        vec2(w, 108.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            card(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(title)
                                        .size(11.5)
                                        .color(th::TEXT_MUTED),
                                );
                                ui.add_space(6.0);
                                readout(ui, &val, &unit, col);
                            });
                        },
                    );
                }
            });

            ui.add_space(10.0);

            // Secondary metrics as a compact strip.
            card(ui, |ui| {
                ui.horizontal(|ui| {
                    let items: [(&str, String); 4] = [
                        ("STAPM", format!("{:.0} W", snap.stapm_value_mw / 1000.0)),
                        ("FAST", format!("{:.0} W", snap.fast_value_mw / 1000.0)),
                        ("SLOW", format!("{:.0} W", snap.slow_value_mw / 1000.0)),
                        ("THM", format!("{:.0} °C", snap.thm_core_c)),
                    ];
                    for (k, v) in items {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(k).size(10.5).color(th::TEXT_MUTED),
                            );
                            ui.label(egui::RichText::new(v).size(14.0).color(th::TEXT));
                        });
                        ui.add_space(18.0);
                    }
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let ms = app.snap.took_ms;
                            if ms > 0 {
                                ui.label(
                                    egui::RichText::new(format!("{ms} ms"))
                                        .size(10.5)
                                        .color(th::TEXT_MUTED),
                                );
                            }
                        },
                    );
                });
            });
        });
    });

    ui.add_space(14.0);

    // ---------------- chart ----------------
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(s.chart_title(lang))
                    .size(12.5)
                    .color(th::TEXT_MUTED)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(s.chart_temp(lang))
                        .size(11.0)
                        .color(th::WARN),
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(s.chart_power(lang))
                        .size(11.0)
                        .color(th::ACCENT),
                );
            });
        });
        ui.add_space(8.0);
        let w = ui.available_width();
        chart(
            ui,
            132.0_f32.min(w * 0.34 + 70.0),
            &app.power_hist,
            &app.temp_hist,
            app.cap_w,
        );
    });

    ui.add_space(14.0);

    // ---------------- session statistics ----------------
    // Answers the "is 15 W actually cheaper than 25 W for my battery" question
    // without needing an external power meter.
    if app.samples > 2 {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(s.stat_title(lang))
                        .size(12.5)
                        .color(th::TEXT_MUTED)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(s.stat_note(lang))
                            .size(10.5)
                            .color(th::TEXT_MUTED),
                    );
                });
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let avg = app.power_sum_w / app.samples as f64;
                let items: [(&str, String); 3] = [
                    (s.stat_avg(lang), format!("{avg:.1} W")),
                    (s.stat_peak(lang), format!("{:.1} W", app.peak_w)),
                    (s.stat_energy(lang), format!("{:.2} Wh", app.energy_wh)),
                ];
                for (k, v) in items {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(k).size(10.5).color(th::TEXT_MUTED));
                        ui.label(egui::RichText::new(v).size(16.0).color(th::TEXT));
                    });
                    ui.add_space(28.0);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mins = app.start_time.elapsed().as_secs() / 60;
                    ui.label(
                        egui::RichText::new(format!("{mins} min · {} samples", app.samples))
                            .size(10.5)
                            .color(th::TEXT_MUTED),
                    );
                });
            });
        });

        ui.add_space(14.0);
    }

    // ---------------- cap control ----------------
    card(ui, |ui| {
        ui.label(
            egui::RichText::new(s.cap_title(lang))
                .size(12.5)
                .color(th::TEXT_MUTED)
                .strong(),
        );
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            // Editable numeric readout.
            let edit = egui::TextEdit::singleline(&mut app.cap_text)
                .desired_width(64.0)
                .align(egui::Align2::CENTER_CENTER)
                .font(egui::TextStyle::Monospace);
            let resp = ui.add(edit);
            ui.label(egui::RichText::new(s.unit_w(lang)).size(14.0).color(th::TEXT_DIM));

            if resp.changed() {
                if let Ok(v) = app.cap_text.trim().parse::<f32>() {
                    app.cap_w = v.clamp(3.0, 60.0);
                }
            }
            if resp.lost_focus() {
                app.cap_text = format!("{:.1}", app.cap_w);
            }

            ui.add_space(6.0);
            let mut w = app.cap_w;
            let slider = egui::Slider::new(&mut w, 3.0..=60.0)
                .show_value(false)
                .step_by(0.5);
            if ui.add(slider).changed() {
                app.cap_w = w;
                app.cap_text = format!("{w:.1}");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = if app.applying {
                    s.cap_applying(lang)
                } else {
                    s.cap_apply(lang)
                };
                if primary_button(ui, label, !app.applying).clicked() {
                    app.apply_cap();
                }
            });
        });

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(s.cap_hint(lang))
                .size(11.0)
                .color(th::TEXT_MUTED),
        );

        ui.add_space(10.0);
        divider(ui);
        ui.add_space(8.0);

        // Quick presets.
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new(s.cap_quick(lang))
                    .size(11.5)
                    .color(th::TEXT_MUTED),
            );
            ui.add_space(4.0);
            for prof in app.cfg.profiles.clone() {
                let w = prof.watts();
                let active = (app.cap_w - w).abs() < 0.05;
                let text = format!("{:.0} W", w);
                let mut btn = egui::Button::new(
                    egui::RichText::new(text)
                        .size(12.5)
                        .color(if active { th::ACCENT } else { th::TEXT_DIM }),
                );
                if active {
                    btn = btn.fill(th::alpha(th::ACCENT_DEEP, 60));
                }
                if ui.add(btn).clicked() {
                    app.cap_w = w;
                    app.cap_text = format!("{w:.1}");
                }
            }
        });
    });

    // Driver warning.
    if app.snap.driver == DriverState::Dead {
        ui.add_space(12.0);
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(s.st_dead(lang))
                        .size(13.0)
                        .color(th::DANGER)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if primary_button(ui, s.set_driverrestart(lang), !app.busy_driver)
                        .clicked()
                    {
                        app.busy_driver = true;
                        crate::core::backend::restart_driver();
                        app.busy_driver = false;
                        app.request_poll();
                        app.toast(s.msg_ok(lang), th::OK);
                    }
                });
            });
        });
    }
}