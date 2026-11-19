//! Hardware identification. Everything here is cheap registry reads so the
//! Hardware page renders instantly even while RyzenAdj is still polling.

use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Default)]
pub struct HwInfo {
    pub cpu_name: String,
    pub vendor: String,
    pub cores: String,
    pub threads: String,
    pub max_mhz: String,
    pub model_line: String,
    pub product: String,
    pub board: String,
    pub bios: String,
    pub os: String,
    pub arch: String,
    pub ryzenadj: String,
    pub driver: String,
}

fn reg_read(path: &str, key: &str) -> String {
    // reg.exe is present everywhere and costs ~15 ms; we only run this once.
    let mut c = Command::new("reg.exe");
    c.args(["query", path, "/v", key]);
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    let out = match c.output() {
        Ok(o) => o,
        Err(_) => return String::new(),
    };
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    for line in text.lines() {
        if line.trim_start().starts_with(key) {
            // `<name>    REG_SZ    <value>` or `<name>    REG_DWORD    0xed1`
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let mut it = tokens.iter().skip(2);
            let first = match it.next() {
                Some(v) => *v,
                None => return String::new(),
            };
            // REG_DWORD values come back in hex; callers that want a number
            // (e.g. `~MHz`) must not receive a raw `0x` string.
            if let Some(hex) = first.strip_prefix("0x") {
                if let Ok(n) = u32::from_str_radix(hex, 16) {
                    return n.to_string();
                }
            }
            let mut parts = vec![first];
            parts.extend(it);
            return parts.join(" ");
        }
    }
    String::new()
}

fn os_version() -> String {
    // `ver` writes in the OEM code page (GBK here), so any localised text is
    // unreadable once decoded as UTF-8. Only the numeric part is safe, and the
    // product name comes from the registry instead.
    let mut c = Command::new("cmd.exe");
    c.args(["/c", "ver"]);
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    let build = match c.output() {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout).to_string();
            extract_version(&s)
        }
        Err(_) => String::new(),
    };

    let product = reg_read(
        r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion",
        "ProductName",
    );
    match (product.trim(), build.as_str()) {
        ("", "") => String::new(),
        ("", b) => b.to_string(),
        (p, "") => p.to_string(),
        (p, b) => format!("{p} {b}"),
    }
}

/// Pull the first `a.b.c.d` style version out of a string.
fn extract_version(s: &str) -> String {
    let bytes: Vec<char> = s.chars().collect();
    let mut best = String::new();
    let mut cur = String::new();
    for &ch in &bytes {
        if ch.is_ascii_digit() || ch == '.' {
            cur.push(ch);
        } else {
            if cur.matches('.').count() >= 2 && cur.len() > best.len() {
                best = cur.clone();
            }
            cur.clear();
        }
    }
    if cur.matches('.').count() >= 2 && cur.len() > best.len() {
        best = cur;
    }
    best.trim_matches('.').to_string()
}

fn driver_state() -> String {
    let mut c = Command::new("sc.exe");
    c.args(["query", "inpoutx64"]);
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    match c.output() {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout).to_string();
            if s.contains("RUNNING") {
                "RUNNING".into()
            } else if s.contains("STOPPED") {
                "STOPPED".into()
            } else if s.contains("1060") || s.contains("does not exist") {
                "NOT INSTALLED".into()
            } else {
                "UNKNOWN".into()
            }
        }
        Err(_) => "UNKNOWN".into(),
    }
}

pub fn collect() -> HwInfo {
    const CPU: &str = r"HKLM\HARDWARE\DESCRIPTION\System\CentralProcessor\0";
    let mut hw = HwInfo {
        cpu_name: reg_read(CPU, "ProcessorNameString"),
        vendor: reg_read(CPU, "VendorIdentifier"),
        max_mhz: reg_read(CPU, "~MHz"),
        threads: std::thread::available_parallelism()
            .map(|n| n.get().to_string())
            .unwrap_or_default(),
        ..Default::default()
    };
    hw.cpu_name = hw.cpu_name.trim().to_string();
    if let Ok(n) = hw.max_mhz.parse::<u32>() {
        hw.max_mhz = format!("{:.2} GHz", n as f32 / 1000.0);
    }
    hw.cores = (hw
        .threads
        .parse::<u32>()
        .map(|t| (t / 2).max(1))
        .unwrap_or(0))
    .to_string();

    hw.model_line = reg_read(
        r"HKLM\HARDWARE\DESCRIPTION\System\BIOS",
        "SystemManufacturer",
    ) + " "
        + &reg_read(r"HKLM\HARDWARE\DESCRIPTION\System\BIOS", "SystemProductName");
    hw.product = reg_read(r"HKLM\HARDWARE\DESCRIPTION\System\BIOS", "SystemProductName");
    hw.board = reg_read(r"HKLM\HARDWARE\DESCRIPTION\System\BIOS", "BaseBoardProduct");
    let bios_ver = reg_read(r"HKLM\HARDWARE\DESCRIPTION\System\BIOS", "BIOSVersion");
    let bios_date = reg_read(r"HKLM\HARDWARE\DESCRIPTION\System\BIOS", "BIOSReleaseDate");
    hw.bios = format!("{}  {}", bios_ver.trim(), bios_date.trim())
        .trim()
        .to_string();
    hw.os = os_version();
    hw.arch = std::env::consts::ARCH.to_string();
    hw.ryzenadj = crate::core::backend::Backend::locate()
        .map(|b| b.exe.display().to_string())
        .unwrap_or_else(|| "not found".into());
    hw.driver = driver_state();
    hw
}