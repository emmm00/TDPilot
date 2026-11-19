//! Logon auto-apply. Mirrors the PowerShell build's proven approach: a .cmd
//! dropped into the user's Startup folder that re-runs ryzenadj elevated.
//! Scheduled tasks were measured to fail on this machine, so we do not use them.

use std::path::PathBuf;

fn startup_dir() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    Some(
        PathBuf::from(appdata)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup"),
    )
}

fn startup_file() -> Option<PathBuf> {
    Some(startup_dir()?.join("TDPilot AutoApply.cmd"))
}

pub fn is_enabled() -> bool {
    startup_file().map(|p| p.is_file()).unwrap_or(false)
}

/// The startup script is deliberately tiny: it elevates and calls our own exe
/// with `--apply-at-logon`, so the wattage lives in config.json (single source
/// of truth) and can be changed from the UI without rewriting the script.
pub fn enable(exe: &std::path::Path) -> Result<(), String> {
    let dir = startup_dir().ok_or("APPDATA not set")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir: {e}"))?;
    let target = startup_file().ok_or("no startup path")?;

    // Must be pure ASCII: cmd.exe reads it with the OEM codepage.
    let body = format!(
        "@echo off\r\n\
         rem TDPilot logon auto-apply (generated - safe to delete)\r\n\
         start \"\" powershell.exe -NoProfile -WindowStyle Hidden -Command \"Start-Process -FilePath '{exe}' -ArgumentList '--apply-at-logon' -Verb RunAs -WindowStyle Hidden\"\r\n",
        exe = exe.display()
    );
    std::fs::write(&target, body).map_err(|e| format!("write: {e}"))?;
    Ok(())
}

pub fn disable() -> Result<(), String> {
    match startup_file() {
        Some(p) if p.is_file() => std::fs::remove_file(p).map_err(|e| format!("remove: {e}")),
        _ => Ok(()),
    }
}