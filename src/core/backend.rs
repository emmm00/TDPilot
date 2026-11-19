//! RyzenAdj backend: spawn ryzenadj.exe (spawning is far faster and more
//! reliable than the in-process DLL, which fails to initialise on this box).

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverState {
    Ok,
    Dead,
    Unknown,
}

/// One full poll of the PM table.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub stapm_limit_mw: f32,
    pub fast_limit_mw: f32,
    pub slow_limit_mw: f32,
    pub stapm_value_mw: f32,
    pub fast_value_mw: f32,
    pub slow_value_mw: f32,
    pub tctl_temp_c: f32,
    pub socket_power_mw: f32,
    pub cclk_mhz: f32,
    pub fan_rpm: f32,
    pub thm_core_c: f32,
    pub driver: DriverState,
    pub took_ms: u64,
}

impl Snapshot {
    pub fn empty() -> Self {
        Self {
            stapm_limit_mw: 0.0,
            fast_limit_mw: 0.0,
            slow_limit_mw: 0.0,
            stapm_value_mw: 0.0,
            fast_value_mw: 0.0,
            slow_value_mw: 0.0,
            tctl_temp_c: 0.0,
            socket_power_mw: 0.0,
            cclk_mhz: 0.0,
            fan_rpm: 0.0,
            thm_core_c: 0.0,
            driver: DriverState::Unknown,
            took_ms: 0,
        }
    }

    pub fn power_w(&self) -> f32 {
        self.socket_power_mw / 1000.0
    }
    pub fn limit_w(&self) -> f32 {
        self.stapm_limit_mw / 1000.0
    }
}

pub struct Backend {
    pub exe: PathBuf,
    pub workdir: PathBuf,
}

impl Backend {
    /// Locate `ryzenadj.exe`. Search order:
    ///   1. `./bin/` next to the executable
    ///   2. `%ProgramData%\TDPilot\bin`
    ///   3. the legacy PowerShell install's `bin`
    pub fn locate() -> Option<Self> {
        let mut candidates: Vec<PathBuf> = Vec::new();

        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                candidates.push(dir.join("bin"));
                candidates.push(dir.to_path_buf());
            }
        }
        if let Ok(pd) = std::env::var("ProgramData") {
            candidates.push(PathBuf::from(&pd).join("TDPilot").join("bin"));
        }
        if let Ok(home) = std::env::var("USERPROFILE") {
            candidates.push(
                PathBuf::from(home)
                    .join("Documents/deepseek-harness/default-workspace/TDPilot/bin"),
            );
        }

        for dir in candidates {
            let exe = dir.join("ryzenadj.exe");
            if exe.is_file() {
                return Some(Self {
                    exe,
                    workdir: dir,
                });
            }
        }
        None
    }

    fn base_cmd(&self) -> Command {
        let mut c = Command::new(&self.exe);
        c.current_dir(&self.workdir);
        #[cfg(windows)]
        c.creation_flags(CREATE_NO_WINDOW);
        c
    }

    fn run(&self, args: &[&str], timeout: Duration) -> Option<Output> {
        let mut child = self
            .base_cmd()
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null())
            .spawn()
            .ok()?;

        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if started.elapsed() > timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        return None;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => return None,
            }
        }
        child.wait_with_output().ok()
    }

    /// Read the PM table. Never trust the exit code: ryzenadj v0.19.0
    /// segfaults (0xC0000005) *after* doing its work, so we parse stdout
    /// and treat a successful parse as success.
    pub fn read(&self) -> Snapshot {
        let started = Instant::now();
        let mut snap = Snapshot::empty();

        let out = match self.run(&["--info"], Duration::from_millis(6000)) {
            Some(o) => o,
            None => return snap,
        };
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        let err = String::from_utf8_lossy(&out.stderr).to_string();

        // Table rows look like: `| STAPM LIMIT         |    25.000 | stapm-limit |`
        // so split on '|' and use fields[1] as the name, fields[2] as the value.
        let mut saw_any = false;
        let mut socket_power = 0.0f32;
        for line in text.lines() {
            let mut f = line.split('|');
            let _ = f.next(); // leading empty cell
            let key = match f.next() {
                Some(k) => k.trim(),
                None => continue,
            };
            let val = match f.next() {
                Some(v) => v.trim(),
                None => continue,
            };
            let num: f32 = match val.parse() {
                Ok(n) => n,
                Err(_) => continue,
            };
            if !num.is_finite() {
                continue;
            }
            saw_any = true;
            // ryzenadj prints watts; our fields are milliwatts.
            let mw = num * 1000.0;
            match key {
                "STAPM LIMIT" => snap.stapm_limit_mw = mw,
                "PPT LIMIT FAST" => snap.fast_limit_mw = mw,
                "PPT LIMIT SLOW" => snap.slow_limit_mw = mw,
                "STAPM VALUE" => snap.stapm_value_mw = mw,
                "PPT VALUE FAST" => snap.fast_value_mw = mw,
                "PPT VALUE SLOW" => snap.slow_value_mw = mw,
                "THM LIMIT CORE" => snap.thm_core_c = num,
                // The *VALUE* row is the live reading; the *LIMIT* row is the cap.
                "THM VALUE CORE" => snap.tctl_temp_c = num,
                "STT VALUE APU" => {
                    if snap.tctl_temp_c == 0.0 {
                        snap.tctl_temp_c = num;
                    }
                }
                "SOCKET POWER" => socket_power = mw,
                "CCLK BUSY VALUE" => snap.cclk_mhz = num,
                "FAN RPM" => snap.fan_rpm = num,
                _ => {}
            }
        }

        // Most Raven/Hawk Point tables have no SOCKET POWER row; the STAPM
        // average is the closest equivalent to package power.
        snap.socket_power_mw = if socket_power > 0.0 {
            socket_power
        } else {
            snap.stapm_value_mw
        };

        // `CCLK BUSY VALUE` reads `nan` on Hawk Point, so fall back to the
        // kernel's per-core counters.
        if snap.cclk_mhz <= 0.0 {
            snap.cclk_mhz = crate::core::power::cpu_mhz();
        }

        snap.driver = if saw_any {
            DriverState::Ok
        } else if err.contains("os_access") || err.contains("Unable to init") {
            DriverState::Dead
        } else {
            DriverState::Unknown
        };
        snap.took_ms = started.elapsed().as_millis() as u64;
        snap
    }

    /// Apply a wattage cap to STAPM/FAST/SLOW and verify by reading back.
    pub fn apply(&self, watts: f32, tctl: Option<f32>) -> Result<f32, String> {
        let mw = (watts * 1000.0).round().max(0.0) as u64;
        let mws = mw.to_string();

        let mut args: Vec<&str> = vec![
            "--stapm-limit",
            &mws,
            "--fast-limit",
            &mws,
            "--slow-limit",
            &mws,
        ];
        let tctl_s;
        if let Some(t) = tctl {
            tctl_s = format!("{}", t.round() as i64);
            args.push("--tctl-temp");
            args.push(&tctl_s);
        }

        // A write is expected to crash on this platform; ignore the result.
        let _ = self.run(&args, Duration::from_millis(6000));
        std::thread::sleep(Duration::from_millis(200));

        // Verification is the only trustworthy signal.
        let snap = self.read();
        if snap.driver != DriverState::Ok {
            return Err("driver unavailable".into());
        }
        let ok = |v: f32| (v - watts).abs() <= 0.6;
        if ok(snap.stapm_limit_mw / 1000.0)
            && ok(snap.fast_limit_mw / 1000.0)
            && ok(snap.slow_limit_mw / 1000.0)
        {
            Ok(snap.stapm_limit_mw / 1000.0)
        } else {
            Err(format!(
                "readback mismatch: {:.2}/{:.2}/{:.2} W",
                snap.stapm_limit_mw / 1000.0,
                snap.fast_limit_mw / 1000.0,
                snap.slow_limit_mw / 1000.0
            ))
        }
    }
}

/// `sc stop/start inpoutx64` — the only known recovery for a wedged driver.
pub fn restart_driver() -> bool {
    let run_sc = |verb: &str| -> bool {
        let mut c = Command::new("sc.exe");
        c.args([verb, "inpoutx64"]);
        #[cfg(windows)]
        c.creation_flags(CREATE_NO_WINDOW);
        c.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };
    run_sc("stop");
    std::thread::sleep(Duration::from_millis(700));
    run_sc("start");
    std::thread::sleep(Duration::from_millis(900));
    true
}

pub fn write_log(dir: &std::path::Path, msg: &str) {
    let path = dir.join("tdpilot.log");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(f, "{}", msg);
    }
}