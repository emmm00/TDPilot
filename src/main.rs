#![windows_subsystem = "windows"]

mod app;
mod core;

use std::os::windows::ffi::OsStrExt;

use windows_sys::Win32::UI::Shell::{IsUserAnAdmin, ShellExecuteW};

fn wide(s: &std::ffi::OsStr) -> Vec<u16> {
    s.encode_wide().chain(std::iter::once(0)).collect()
}

pub fn is_admin() -> bool {
    unsafe { IsUserAnAdmin() != 0 }
}

/// Relaunch ourselves elevated through the UAC "runas" verb.
/// The original command line is forwarded, otherwise flags such as `--page`
/// would be silently dropped by the elevation hop.
pub fn relaunch_elevated() -> bool {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let op = wide(std::ffi::OsStr::new("runas"));
    let file = wide(exe.as_os_str());
    let params = std::env::args()
        .skip(1)
        .collect::<Vec<_>>()
        .join(" ");
    let params_w = wide(std::ffi::OsStr::new(&params));
    let h = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            op.as_ptr(),
            file.as_ptr(),
            if params.is_empty() {
                std::ptr::null()
            } else {
                params_w.as_ptr()
            },
            std::ptr::null(),
            1, // SW_SHOWNORMAL
        )
    };
    h as isize > 32
}

fn main() -> eframe::Result<()> {
    // Headless logon mode: apply the saved cap and exit. The Startup entry
    // calls this, so the GUI never has to be open for auto-apply to work.
    if std::env::args().any(|a| a == "--apply-at-logon") {
        let cfg = core::config::load();
        let dir = core::config::data_dir();
        match core::backend::Backend::locate() {
            Some(be) => {
                let r = be.apply(cfg.watts(), tctl_opt(&cfg));
                core::backend::write_log(
                    &dir,
                    &format!(
                        "[logon] apply {:.1} W -> {}",
                        cfg.watts(),
                        match &r {
                            Ok(v) => format!("ok ({v:.2} W)"),
                            Err(e) => format!("failed: {e}"),
                        }
                    ),
                );
            }
            None => core::backend::write_log(&dir, "[logon] ryzenadj.exe not found"),
        }
        return Ok(());
    }

    // The driver needs an elevated process. Re-launch once, then continue.
    if !is_admin() {
        if relaunch_elevated() {
            return Ok(());
        }
        // If elevation was refused we still start; the UI explains the problem.
    }

    let cfg = core::config::load();
    let start_min = cfg.start_minimized;

    // `--page <name>` opens straight onto a page; used by the screenshot
    // harness so every page can be captured without simulating clicks.
    let boot_page = std::env::args()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|w| w[0] == "--page")
        .map(|w| w[1].clone());

    // A hidden `--tall` switch lets the acceptance harness render content that
    // normally sits below the fold, without changing the shipped default size.
    let tall = std::env::args().any(|a| a == "--tall");
    let inner = if tall {
        [1060.0, 1180.0]
    } else {
        [1060.0, 720.0]
    };

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(inner)
        .with_min_inner_size([900.0, 600.0])
        .with_title("TDPilot");
    if start_min {
        viewport = viewport.with_visible(false);
    }

    let options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "TDPilot",
        options,
        Box::new(move |cc| {
            let mut app = app::App::new(cc);
            if let Some(p) = boot_page.as_deref() {
                app.goto_page_by_name(p);
            }
            Ok(Box::new(app))
        }),
    )
}

pub fn tctl_opt(cfg: &core::Config) -> Option<f32> {
    if cfg.tctl_temp >= 60 && cfg.tctl_temp <= 105 {
        Some(cfg.tctl_temp as f32)
    } else {
        None
    }
}