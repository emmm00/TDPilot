//! Configuration, stored as JSON in `%ProgramData%\TDPilot\config.json`.
//! Field names mirror the PowerShell build so an in-place upgrade keeps the
//! user's existing settings.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "MW")]
    pub mw: u32,
}

impl Profile {
    pub fn watts(&self) -> f32 {
        self.mw as f32 / 1000.0
    }
}

fn default_lang() -> String {
    "zh".into()
}
fn default_refresh() -> u64 {
    3000
}
fn default_power_mw() -> u32 {
    25000
}
fn default_profiles() -> Vec<Profile> {
    vec![
        Profile { name: "15.0 W".into(), mw: 15000 },
        Profile { name: "25.0 W".into(), mw: 25000 },
        Profile { name: "30.0 W".into(), mw: 30000 },
        Profile { name: "45.0 W".into(), mw: 45000 },
        Profile { name: "54.0 W".into(), mw: 54000 },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "Lang", default = "default_lang")]
    pub lang: String,
    #[serde(rename = "LastApplied", default)]
    pub last_applied: String,
    #[serde(rename = "PowerMW", default = "default_power_mw")]
    pub power_mw: u32,
    #[serde(rename = "RefreshMs", default = "default_refresh")]
    pub refresh_ms: u64,
    #[serde(rename = "AutoApply", default)]
    pub auto_apply: bool,
    #[serde(rename = "Profiles", default = "default_profiles")]
    pub profiles: Vec<Profile>,
    /// Kept at 0 unless the user overrides it; the UI treats 0 as "leave alone".
    #[serde(rename = "TctlTemp", default)]
    pub tctl_temp: u32,
    /// Start minimised to the tray-equivalent (small window) at logon.
    #[serde(rename = "StartMinimized", default)]
    pub start_minimized: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lang: default_lang(),
            last_applied: String::new(),
            power_mw: default_power_mw(),
            refresh_ms: default_refresh(),
            auto_apply: false,
            profiles: default_profiles(),
            tctl_temp: 0,
            start_minimized: false,
        }
    }
}

impl Config {
    pub fn watts(&self) -> f32 {
        self.power_mw as f32 / 1000.0
    }
    pub fn set_watts(&mut self, w: f32) {
        self.power_mw = (w * 1000.0).round().clamp(3000.0, 60000.0) as u32;
    }
}

pub fn data_dir() -> PathBuf {
    let base = std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".into());
    PathBuf::from(base).join("TDPilot")
}

pub fn config_path() -> PathBuf {
    data_dir().join("config.json")
}

pub fn load() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(s) => {
            // The PowerShell build wrote config.json with a UTF-8 BOM, which
            // serde_json rejects. Strip it so an in-place upgrade keeps settings.
            let body = s.trim_start_matches('\u{feff}');
            serde_json::from_str(body).unwrap_or_else(|e| {
                crate::core::backend::write_log(
                    &data_dir(),
                    &format!("config parse failed ({e}); using defaults"),
                );
                Config::default()
            })
        }
        Err(_) => Config::default(),
    }
}

pub fn save(cfg: &Config) -> Result<(), String> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir: {e}"))?;
    let json = serde_json::to_string_pretty(cfg).map_err(|e| format!("serialise: {e}"))?;
    let tmp = dir.join("config.json.tmp");
    std::fs::write(&tmp, json).map_err(|e| format!("write: {e}"))?;
    std::fs::rename(&tmp, config_path()).map_err(|e| format!("rename: {e}"))?;
    Ok(())
}

pub fn limit_file() -> PathBuf {
    data_dir().join("limit-mw.txt")
}

pub fn write_limit_mw(mw: u32) {
    let _ = std::fs::create_dir_all(data_dir());
    let _ = std::fs::write(limit_file(), mw.to_string());
}