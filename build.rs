// Compiles the Windows resource script (icon + version info) into an object
// file and links it into the executable.
//
// Only does anything on Windows targets; on other hosts it is a no-op so that
// `cargo check` still works for people reading the code on Linux/macOS.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/icon.ico");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let rc_file = manifest.join("assets").join("app.rc");
    let obj = out_dir.join("app_res.o");

    // windres ships with the mingw-w64 toolchain that the GNU target needs
    // anyway, so there is no extra dependency.
    let status = Command::new("windres")
        .arg("--input")
        .arg(&rc_file)
        .arg("--output")
        .arg(&obj)
        .arg("--output-format=coff")
        .arg("-I")
        .arg(manifest.join("assets"))
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:rustc-link-arg={}", obj.display());
        }
        Ok(s) => panic!("windres failed with status {s}"),
        Err(e) => panic!(
            "could not run `windres` ({e}). Install a mingw-w64 toolchain and put its bin/ \
             directory on PATH — the GNU target needs it for both linking and resources."
        ),
    }
}
