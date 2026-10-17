use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let icon_path = manifest_dir.join("../../assets/app.ico");

    println!("cargo:rerun-if-changed={}", icon_path.display());

    if !icon_path.exists() {
        return;
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let rc_path = out_dir.join("app_icon.rc");
    let res_path = out_dir.join("app_icon.res");

    let icon_str = icon_path.to_string_lossy().replace('\\', "/");
    let rc_content = format!("1 ICON \"{}\"\n", icon_str);

    if std::fs::write(&rc_path, rc_content).is_err() {
        return;
    }

    let rc_exe = find_rc_exe().unwrap_or_else(|| PathBuf::from("rc.exe"));

    let status = Command::new(&rc_exe)
        .arg("/fo")
        .arg(&res_path)
        .arg(&rc_path)
        .status();

    if let Ok(s) = status {
        if s.success() {
            println!("cargo:rustc-link-arg={}", res_path.display());
        }
    }
}

fn find_rc_exe() -> Option<PathBuf> {
    // 1. Check if rc.exe is directly invocable
    if Command::new("rc.exe").arg("/?").output().is_ok() {
        return Some(PathBuf::from("rc.exe"));
    }

    // 2. Search Windows Kits directory
    let win_kits = Path::new(r"C:\Program Files (x86)\Windows Kits\10\bin");
    if win_kits.exists() {
        if let Ok(entries) = std::fs::read_dir(win_kits) {
            for entry in entries.flatten() {
                let candidate = entry.path().join("x64").join("rc.exe");
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}
