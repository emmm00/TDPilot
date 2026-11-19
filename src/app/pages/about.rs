//! About: version, license and links.

use crate::app::{th, widgets::*, App};
use egui::Ui;

const REPO: &str = "https://github.com/emmm00/TDPilot";

pub fn show(app: &mut App, ui: &mut Ui) {
    let lang = app.lang;
    let s = app.s();

    card(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("TDPilot")
                    .size(30.0)
                    .strong()
                    .color(th::TEXT),
            );
            ui.label(
                egui::RichText::new(match lang {
                    crate::core::Lang::Zh => "AMD 锐龙功耗上限调节器",
                    crate::core::Lang::En => "Power cap tuner for AMD Ryzen",
                })
                .size(13.0)
                .color(th::TEXT_MUTED),
            );
            ui.add_space(10.0);
            pill(
                ui,
                &format!("v{}", env!("CARGO_PKG_VERSION")),
                th::ACCENT,
            );
            ui.add_space(10.0);
        });

        ui.add_space(12.0);
        divider(ui);
        ui.add_space(10.0);

        kv(ui, s.about_version(lang), env!("CARGO_PKG_VERSION"));
        kv(ui, s.about_license(lang), "MIT");
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.add_sized(
                [150.0, 18.0],
                egui::Label::new(
                    egui::RichText::new(s.about_source(lang))
                        .size(13.0)
                        .color(th::TEXT_MUTED),
                ),
            );
            if ui
                .link(egui::RichText::new(REPO).size(13.0).color(th::ACCENT))
                .clicked()
            {
                let _ = std::process::Command::new("cmd.exe")
                    .args(["/c", "start", "", REPO])
                    .spawn();
            }
        });

        ui.add_space(14.0);
        divider(ui);
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new(match lang {
                crate::core::Lang::Zh => {
                    "功耗写入依赖 RyzenAdj（MIT 许可）。修改电压/功耗有风险，\u{8bf7}\u{81ea}\u{884c}\u{8bc4}\u{4f30}。"
                }
                crate::core::Lang::En => {
                    "Power writes go through RyzenAdj (MIT). Changing power limits carries risk."
                }
            })
            .size(11.5)
            .color(th::TEXT_MUTED),
        );
    });

    ui.add_space(12.0);

    card(ui, |ui| {
        ui.label(
            egui::RichText::new(match lang {
                crate::core::Lang::Zh => "致谢",
                crate::core::Lang::En => "Credits",
            })
            .size(12.5)
            .color(th::TEXT_MUTED)
            .strong(),
        );
        ui.add_space(8.0);
        for (name, note) in [
            ("RyzenAdj", "FlyGoat/RyzenAdj — SMU access"),
            ("egui / eframe", "emilk/egui — immediate-mode UI"),
            ("InpOut", "highrez — kernel port I/O driver"),
        ] {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(name)
                        .size(13.0)
                        .color(th::TEXT)
                        .strong(),
                );
                ui.label(egui::RichText::new(note).size(11.5).color(th::TEXT_MUTED));
            });
        }
    });
}