//! Settings: refresh interval, language, log + data folder, driver recovery.

use crate::app::{th, widgets::*, App};
use crate::core::{config, Config};
use egui::Ui;

pub fn show(app: &mut App, ui: &mut Ui) {
    let lang = app.lang;
    let s = app.s();

    card(ui, |ui| {
        ui.label(
            egui::RichText::new(s.set_interval(lang))
                .size(12.5)
                .color(th::TEXT_MUTED)
                .strong(),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let mut ms = app.cfg.refresh_ms as f32;
            let resp = ui.add(
                egui::Slider::new(&mut ms, 1000.0..=10000.0)
                    .step_by(500.0)
                    .suffix(" ms"),
            );
            if resp.changed() {
                app.cfg.refresh_ms = ms as u64;
                let _ = config::save(&app.cfg);
            }
            ui.label(
                egui::RichText::new(format!("{:.1} s", ms / 1000.0))
                    .size(11.5)
                    .color(th::TEXT_MUTED),
            );
        });
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(match lang {
                crate::core::Lang::Zh => "数值越大越省电，但曲线也越粗糙",
                crate::core::Lang::En => "Higher is lighter on the CPU but coarser on the chart",
            })
            .size(11.0)
            .color(th::TEXT_MUTED),
        );
    });

    ui.add_space(12.0);

    card(ui, |ui| {
        ui.label(
            egui::RichText::new(s.set_lang(lang))
                .size(12.5)
                .color(th::TEXT_MUTED)
                .strong(),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for (code, label) in [("zh", "中文"), ("en", "English")] {
                let active = app.lang.code() == code;
                let btn = egui::Button::new(
                    egui::RichText::new(label)
                        .size(13.0)
                        .color(if active { th::ACCENT } else { th::TEXT_DIM }),
                )
                .fill(if active {
                    th::alpha(th::ACCENT_DEEP, 70)
                } else {
                    th::SURFACE_HI
                });
                if ui.add(btn).clicked() {
                    app.lang = crate::core::Lang::from_code(code);
                    app.cfg.lang = code.to_string();
                    let _ = config::save(&app.cfg);
                }
            }
        });
    });

    ui.add_space(12.0);

    card(ui, |ui| {
        ui.label(
            egui::RichText::new(s.set_log(lang))
                .size(12.5)
                .color(th::TEXT_MUTED)
                .strong(),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button(s.set_openlog(lang)).clicked() {
                let p = config::data_dir().join("tdpilot.log");
                let _ = std::process::Command::new("notepad.exe").arg(p).spawn();
            }
            if ui.button(s.set_opendata(lang)).clicked() {
                let p = config::data_dir();
                let _ = std::process::Command::new("explorer.exe").arg(p).spawn();
            }
        });
        ui.add_space(10.0);
        divider(ui);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(s.set_drvhint(lang))
                    .size(11.0)
                    .color(th::TEXT_MUTED),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(s.set_driverrestart(lang)).clicked() {
                    crate::core::backend::restart_driver();
                    app.request_poll();
                    app.toast(s.msg_ok(lang), th::OK);
                }
            });
        });
    });

    ui.add_space(12.0);

    card(ui, |ui| {
        let mut start_min = app.cfg.start_minimized;
        if ui
            .checkbox(&mut start_min, match lang {
                crate::core::Lang::Zh => "启动时最小化",
                crate::core::Lang::En => "Start minimized",
            })
            .changed()
        {
            app.cfg.start_minimized = start_min;
            let _ = config::save(&app.cfg);
        }

        ui.add_space(8.0);
        let mut tctl = app.cfg.tctl_temp;
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(match lang {
                    crate::core::Lang::Zh => "温度墙",
                    crate::core::Lang::En => "Thermal limit",
                })
                .size(12.0)
                .color(th::TEXT_DIM),
            );
            let mut v = tctl as f32;
            // 0 means "leave the firmware value alone"; showing a 0 °C slider
            // reads like a real target, so label that state explicitly.
            let suffix = if tctl == 0 { "" } else { " °C" };
            if ui
                .add(egui::Slider::new(&mut v, 0.0..=105.0).step_by(1.0).suffix(suffix))
                .changed()
            {
                tctl = v as u32;
                app.cfg.tctl_temp = tctl;
                let _ = config::save(&app.cfg);
            }
            if tctl == 0 {
                ui.label(
                    egui::RichText::new(match lang {
                        crate::core::Lang::Zh => "不修改",
                        crate::core::Lang::En => "untouched",
                    })
                    .size(11.5)
                    .color(th::TEXT_MUTED),
                );
            }
        });
        ui.label(
            egui::RichText::new(match lang {
                crate::core::Lang::Zh => "设为 0 表示不修改，60–105 °C 之间才会写入",
                crate::core::Lang::En => "0 leaves it untouched; 60-105 °C is written on apply",
            })
            .size(11.0)
            .color(th::TEXT_MUTED),
        );
    });

    ui.add_space(12.0);

    card(ui, |ui| {
        ui.horizontal(|ui| {
            if ui.button(match lang {
                crate::core::Lang::Zh => "恢复默认设置",
                crate::core::Lang::En => "Reset settings",
            })
            .clicked()
            {
                app.cfg = Config {
                    lang: app.cfg.lang.clone(),
                    ..Default::default()
                };
                app.cap_w = app.cfg.watts();
                app.cap_text = format!("{:.1}", app.cap_w);
                let _ = config::save(&app.cfg);
                app.toast(s.msg_ok(lang), th::OK);
            }
        });
    });
}