//! Application state and the eframe update loop.

mod pages;
pub mod theme;
mod widgets;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

use crate::core::backend::{Backend, DriverState, Snapshot};
use crate::core::{config, hwinfo, Config, Lang, Strings};
use egui::Color32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Profiles,
    Automation,
    Hardware,
    Settings,
    About,
}

impl Page {
    pub fn all() -> [Page; 6] {
        [
            Page::Dashboard,
            Page::Profiles,
            Page::Automation,
            Page::Hardware,
            Page::Settings,
            Page::About,
        ]
    }

    /// Stable slugs used by `--page` and by the screenshot harness.
    pub fn from_slug(s: &str) -> Option<Page> {
        Some(match s {
            "dashboard" => Page::Dashboard,
            "profiles" => Page::Profiles,
            "automation" => Page::Automation,
            "hardware" => Page::Hardware,
            "settings" => Page::Settings,
            "about" => Page::About,
            _ => return None,
        })
    }
}

/// Result of an apply, delivered back from the worker thread.
pub enum ApplyMsg {
    Done(Result<f32, String>),
}

pub struct App {
    pub cfg: Config,
    pub lang: Lang,
    pub page: Page,
    pub snap: Snapshot,
    pub backend: Option<Backend>,
    pub hw: hwinfo::HwInfo,

    // history for the chart
    pub power_hist: Vec<f32>,
    pub temp_hist: Vec<f32>,
    pub max_points: usize,

    // polling
    snap_rx: Receiver<Snapshot>,
    poll_tx: Sender<()>,
    last_poll: Instant,

    // apply
    apply_rx: Receiver<ApplyMsg>,
    apply_tx: Sender<ApplyMsg>,
    pub applying: bool,
    pub toast: Option<(String, Color32, Instant)>,

    // editable cap (may differ from applied value)
    pub cap_w: f32,
    pub cap_text: String,

    // profile editor
    pub editing: Option<usize>,
    pub editor_name: String,
    pub editor_w: f32,

    pub start_time: Instant,
    pub busy_driver: bool,

    // Session statistics: enough to compare a 15 W cap against 25 W or 30 W
    // on battery life without opening another tool.
    pub samples: u64,
    pub power_sum_w: f64,
    pub peak_w: f32,
    pub energy_wh: f64,
    last_sample_at: Option<Instant>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);

        let cfg = config::load();
        let lang = Lang::from_code(&cfg.lang);
        let backend = Backend::locate();

        let (snap_tx, snap_rx) = channel();
        let (poll_tx, poll_rx) = channel::<()>();
        let (apply_tx, apply_rx) = channel();

        // Polling thread: sleeps, then runs one --info pass and pushes it.
        // ryzenadj costs ~700-900 ms, so this must never run on the UI thread.
        {
            let be = backend.as_ref().map(|b| Backend {
                exe: b.exe.clone(),
                workdir: b.workdir.clone(),
            });
            std::thread::spawn(move || {
                // First read fires immediately so the window is never empty.
                if let Some(be) = &be {
                    let _ = snap_tx.send(be.read());
                }
                for _ in poll_rx.iter() {
                    if let Some(be) = &be {
                        let _ = snap_tx.send(be.read());
                    }
                }
            });
        }

        let hw = hwinfo::collect();

        Self {
            lang,
            page: Page::Dashboard,
            snap: Snapshot::empty(),
            backend,
            hw,
            power_hist: Vec::new(),
            temp_hist: Vec::new(),
            max_points: 90,
            snap_rx,
            poll_tx,
            last_poll: Instant::now(),
            apply_rx,
            apply_tx,
            applying: false,
            toast: None,
            cap_w: cfg.watts(),
            cap_text: format!("{:.1}", cfg.watts()),
            cfg,
            editing: None,
            editor_name: String::new(),
            editor_w: 25.0,
            start_time: Instant::now(),
            busy_driver: false,
            samples: 0,
            power_sum_w: 0.0,
            peak_w: 0.0,
            energy_wh: 0.0,
            last_sample_at: None,
        }
    }

    pub fn s(&self) -> Strings {
        Strings
    }

    /// Jump to a page by slug (`--page` support for the screenshot harness).
    pub fn goto_page_by_name(&mut self, slug: &str) {
        if let Some(p) = Page::from_slug(slug) {
            self.page = p;
        }
    }

    pub fn toast(&mut self, msg: impl Into<String>, color: Color32) {
        self.toast = Some((msg.into(), color, Instant::now()));
    }

    /// Ask the poll thread for a fresh sample right now.
    pub fn request_poll(&mut self) {
        let _ = self.poll_tx.send(());
        self.last_poll = Instant::now();
    }

    pub fn apply_cap(&mut self) {
        if self.applying {
            return;
        }
        let w = self.cap_w.clamp(3.0, 60.0);
        self.cfg.set_watts(w);
        config::write_limit_mw(self.cfg.power_mw);
        let _ = config::save(&self.cfg);

        let Some(be) = self.backend.as_ref().map(|b| Backend {
            exe: b.exe.clone(),
            workdir: b.workdir.clone(),
        }) else {
            self.toast("ryzenadj.exe not found", theme::DANGER);
            return;
        };

        let tx = self.apply_tx.clone();
        let tctl = if self.cfg.tctl_temp >= 60 {
            Some(self.cfg.tctl_temp as f32)
        } else {
            None
        };
        self.applying = true;
        std::thread::spawn(move || {
            let res = be.apply(w, tctl);
            let _ = tx.send(ApplyMsg::Done(res));
        });
    }

    fn drain_channels(&mut self) {
        while let Ok(s) = self.snap_rx.try_recv() {
            if s.driver == DriverState::Ok {
                let w = s.power_w();
                self.power_hist.push(w);
                self.temp_hist.push(s.tctl_temp_c);
                if self.power_hist.len() > self.max_points {
                    self.power_hist.remove(0);
                    self.temp_hist.remove(0);
                }

                // Integrate energy over the real gap between samples. ryzenadj
                // only refreshes its counters every few hundred ms, so gaps
                // shorter than that would over-count nothing, not under-count
                // power — clamp the floor to avoid a divide-by-tiny blow-up.
                let now = Instant::now();
                if let Some(prev) = self.last_sample_at {
                    let dt_h = now.duration_since(prev).as_secs_f64() / 3600.0;
                    if dt_h > 0.0 && dt_h < 0.25 {
                        self.energy_wh += w as f64 * dt_h;
                    }
                }
                self.last_sample_at = Some(now);

                self.samples += 1;
                self.power_sum_w += w as f64;
                if w > self.peak_w {
                    self.peak_w = w;
                }
            }
            self.snap = s;
        }

        while let Ok(ApplyMsg::Done(res)) = self.apply_rx.try_recv() {
            self.applying = false;
            match res {
                Ok(v) => {
                    self.toast(
                        format!("{} {:.1} W", self.s().msg_applied(self.lang), v),
                        theme::OK,
                    );
                    self.cfg.last_applied = now_stamp();
                    let _ = config::save(&self.cfg);
                    self.request_poll();
                }
                Err(e) => self.toast(format!("{}: {e}", self.s().msg_failed(self.lang)), theme::DANGER),
            }
        }
    }

    fn maybe_poll(&mut self) {
        let interval = Duration::from_millis(self.cfg.refresh_ms.clamp(1000, 30000));
        if self.last_poll.elapsed() >= interval && !self.applying {
            self.request_poll();
        }
    }

    /// Keyboard shortcuts: Ctrl+1..6 jumps between pages, Ctrl+Enter applies
    /// the current cap, F5 forces a refresh.
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        const KEYS: [egui::Key; 6] = [
            egui::Key::Num1,
            egui::Key::Num2,
            egui::Key::Num3,
            egui::Key::Num4,
            egui::Key::Num5,
            egui::Key::Num6,
        ];
        let pages = Page::all();
        let mut target = None;
        let mut do_apply = false;
        let mut do_poll = false;
        ctx.input_mut(|i| {
            for (n, key) in KEYS.iter().enumerate() {
                if i.consume_key(egui::Modifiers::CTRL, *key) {
                    target = Some(pages[n]);
                }
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter) {
                do_apply = true;
            }
            if i.consume_key(egui::Modifiers::NONE, egui::Key::F5) {
                do_poll = true;
            }
        });
        if let Some(p) = target {
            self.page = p;
        }
        if do_apply {
            self.apply_cap();
        }
        if do_poll {
            self.request_poll();
        }
    }
}

fn now_stamp() -> String {
    // Avoid a chrono dependency: derive local-ish time from the OS clock.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let rem = secs % 86400;
    format!(
        "d{} {:02}:{:02}:{:02}",
        days,
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.drain_channels();
        self.maybe_poll();

        let ctx = ui.ctx().clone();
        self.handle_shortcuts(&ctx);

        // egui only repaints on input by default, so a live-data dashboard must
        // schedule its own wake-ups; without this the poll loop never advances
        // and the history chart stays empty.
        let refresh = Duration::from_millis(self.cfg.refresh_ms.clamp(1000, 30000));
        ui.ctx().request_repaint_after(refresh);

        // Keep animating while an apply is in flight so the toast and spinner
        // stay responsive.
        if self.applying || self.toast.is_some() {
            ui.ctx().request_repaint_after(Duration::from_millis(120));
        }

        // ---- left rail -------------------------------------------------
        egui::Panel::left("rail")
            .exact_size(196.0)
            .show_separator_line(false)
            .frame(
                egui::Frame::NONE
                    .fill(theme::RAIL)
                    .inner_margin(egui::Margin::symmetric(12, 16)),
            )
            .show(ui, |ui| {
                self.rail(ui);
            });

        // ---- top bar ---------------------------------------------------
        egui::Panel::top("topbar")
            .exact_size(62.0)
            .show_separator_line(false)
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::symmetric(18, 8)),
            )
            .show(ui, |ui| {
                self.topbar(ui);
            });

        // ---- content ---------------------------------------------------
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::symmetric(18, 12)),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.page {
                        Page::Dashboard => pages::dashboard::show(self, ui),
                        Page::Profiles => pages::profiles::show(self, ui),
                        Page::Automation => pages::automation::show(self, ui),
                        Page::Hardware => pages::hardware::show(self, ui),
                        Page::Settings => pages::settings::show(self, ui),
                        Page::About => pages::about::show(self, ui),
                    });
            });

        // Toast, bottom-right.
        let ctx = ui.ctx().clone();
        self.draw_toast(&ctx);
    }
}

impl App {
    fn rail(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("TDPilot")
                    .size(19.0)
                    .strong()
                    .color(theme::TEXT),
            );
        });
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .size(11.0)
                    .color(theme::TEXT_MUTED),
            );
        });
        ui.add_space(18.0);

        let lang = self.lang;
        for p in Page::all() {
            let label = match p {
                Page::Dashboard => self.s().tab_dashboard(lang),
                Page::Profiles => self.s().tab_profiles(lang),
                Page::Automation => self.s().tab_auto(lang),
                Page::Hardware => self.s().tab_hardware(lang),
                Page::Settings => self.s().tab_settings(lang),
                Page::About => self.s().tab_about(lang),
            };
            if widgets::nav_item(ui, self.page == p, label, "") {
                self.page = p;
            }
        }

        // Push status to the bottom of the rail.
        let h = ui.available_height();
        if h > 70.0 {
            ui.add_space(h - 66.0);
        }
        widgets::divider(ui);
        ui.add_space(6.0);
        let (txt, col) = match self.snap.driver {
            DriverState::Ok => (self.s().st_ready(lang), theme::OK),
            DriverState::Dead => (self.s().st_dead(lang), theme::DANGER),
            DriverState::Unknown => (self.s().st_unknown(lang), theme::TEXT_MUTED),
        };
        ui.horizontal(|ui| {
            ui.add_space(2.0);
            let (r, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            ui.painter().circle_filled(r.center(), 4.0, col);
            ui.add_space(2.0);
            ui.label(egui::RichText::new(txt).size(12.0).color(col));
        });
    }

    fn topbar(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        ui.horizontal(|ui| {
            ui.add_space(2.0);
            ui.vertical(|ui| {
                ui.add_space(4.0);
                let (title, sub) = match self.page {
                    Page::Dashboard => (self.s().tab_dashboard(lang), self.s().dash_sub(lang)),
                    Page::Profiles => (self.s().tab_profiles(lang), self.s().prof_sub(lang)),
                    Page::Automation => (self.s().tab_auto(lang), self.s().auto_sub(lang)),
                    Page::Hardware => (self.s().tab_hardware(lang), self.s().hw_sub(lang)),
                    Page::Settings => (self.s().tab_settings(lang), self.s().set_sub(lang)),
                    Page::About => (self.s().tab_about(lang), self.s().about_sub(lang)),
                };
                ui.label(egui::RichText::new(title).size(17.0).strong());
                ui.label(egui::RichText::new(sub).size(11.5).color(theme::TEXT_MUTED));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(2.0);
                // Language toggle.
                let other = match lang {
                    Lang::Zh => "EN",
                    Lang::En => "中文",
                };
                if ui.button(other).clicked() {
                    self.lang = self.lang.flip();
                    self.cfg.lang = self.lang.code().to_string();
                    let _ = config::save(&self.cfg);
                }
                ui.add_space(6.0);
                if widgets::primary_button(ui, self.s().set_refresh(lang), !self.applying).clicked() {
                    self.request_poll();
                }
            });
        });
    }

    fn draw_toast(&mut self, ctx: &egui::Context) {
        let Some((msg, col, at)) = self.toast.clone() else {
            return;
        };
        if at.elapsed() > Duration::from_secs(4) {
            self.toast = None;
            return;
        }
        egui::Area::new(egui::Id::unique("toast"))
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-22.0, -22.0))
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(theme::SURFACE_HI)
                    .stroke(egui::Stroke::new(1.0, col))
                    .corner_radius(egui::CornerRadius::same(10))
                    .inner_margin(egui::Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(msg).size(13.0).color(col));
                    });
            });
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

/// Re-export for pages.
pub use theme as th;