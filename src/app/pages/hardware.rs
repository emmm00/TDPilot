//! Hardware: processor and firmware details.

use crate::app::{th, widgets::*, App};
use egui::Ui;

pub fn show(app: &mut App, ui: &mut Ui) {
    let lang = app.lang;
    let s = app.s();
    let hw = app.hw.clone();

    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&hw.cpu_name)
                    .size(15.0)
                    .color(th::TEXT)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(s.hw_refresh(lang)).clicked() {
                    app.hw = crate::core::hwinfo::collect();
                }
            });
        });
        ui.add_space(10.0);
        divider(ui);
        ui.add_space(8.0);

        kv(ui, s.hw_vendor(lang), &hw.vendor);
        kv(
            ui,
            s.hw_cores(lang),
            &format!("{} / {}", hw.cores, hw.threads),
        );
        kv(ui, s.hw_maxclock(lang), &hw.max_mhz);
        kv(ui, s.hw_arch(lang), &hw.arch);
    });

    ui.add_space(12.0);

    card(ui, |ui| {
        kv(ui, s.hw_model(lang), hw.model_line.trim());
        kv(ui, s.hw_product(lang), &hw.product);
        kv(ui, s.hw_board(lang), &hw.board);
        kv(ui, s.hw_bios(lang), &hw.bios);
        kv(ui, s.hw_os(lang), &hw.os);
    });

    ui.add_space(12.0);

    card(ui, |ui| {
        kv(ui, s.hw_path(lang), &hw.ryzenadj);
        let (txt, col) = match hw.driver.as_str() {
            "RUNNING" => (s.st_ready(lang), th::OK),
            "STOPPED" => (s.st_dead(lang), th::WARN),
            _ => (s.st_unknown(lang), th::DANGER),
        };
        let (txt, col) = if hw.driver == "RUNNING" {
            ("RUNNING", th::OK)
        } else {
            (txt, col)
        };
        ui.horizontal(|ui| {
            ui.add_sized(
                [150.0, 18.0],
                egui::Label::new(
                    egui::RichText::new("inpoutx64")
                        .size(13.0)
                        .color(th::TEXT_MUTED),
                ),
            );
            ui.label(egui::RichText::new(txt).size(13.0).color(col));
        });
    });
}
