use anyhow::anyhow;
use anyhow::anyhow::{Context, ensure};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio, exit};

const RESTART_SCRIPT: &str = include_str!("macos/restart.sh");

fn running_app_path() -> Result<PathBuf, anyhow::Error> {
    let exe_path = std::env::current_exe()?;

    exe_path
        .ancestors()
        .find(|path| path.extension().and_then(OsStr::to_str) == Some("app"))
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("app bundle required"))
}

pub fn can_macos_self_update() -> bool {
    running_app_path().is_ok()
}
pub fn perform_macos_self_update(package_path: &PathBuf) -> Result<(), anyhow::Error> {
    let running_path = running_app_path()?;
    let temp_root = package_path
        .parent()
        .ok_or_else(|| anyhow!("package path has no parent: {:?}", package_path))?;
    let extract_dir = temp_root.join("extract");

    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir)?;
    }

    let output = Command::new("/usr/bin/ditto")
        .arg("-x")
        .arg("-k")
        .arg(package_path)
        .arg(&extract_dir)
        .output()
        .with_context(|| "failed to extract")?;
    ensure!(
        output.status.success(),
        "failed to extract: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let contents = fs::read_dir(&extract_dir)?;
    let mut extracted_app = None;
    for entry in contents {
        let entry = entry?;
        if entry.metadata()?.is_dir()
            && entry.path().extension().and_then(OsStr::to_str) == Some("app")
        {
            if extracted_app.is_some() {
                return Err(anyhow!("multiple apps found in package"));
            }
            extracted_app = Some(entry.path());
        }
    }
    let extracted_app = extracted_app.ok_or_else(|| anyhow!("no app found in package"))?;

    let restart_script = temp_root.join("restart.sh");
    fs::write(&restart_script, RESTART_SCRIPT)
        .with_context(|| "failed to write update helper script")?;

    Command::new("/bin/sh")
        .arg(&restart_script)
        .arg(std::process::id().to_string())
        .arg(&extracted_app)
        .arg(&running_path)
        .arg(temp_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    exit(0);
}
