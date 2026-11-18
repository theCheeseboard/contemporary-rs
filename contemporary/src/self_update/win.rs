use anyhow::{anyhow, Context, ensure};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio, exit};

const RESTART_SCRIPT: &str = include_str!("win/restart.ps1");

pub fn can_self_update() -> bool {
    true
}

pub fn perform_win_self_update(package_path: &PathBuf) -> Result<(), anyhow::Error> {
    let running_path = std::env::current_exe().map_err(|e| anyhow!(e))?;

    let temp_root = package_path
        .parent()
        .ok_or_else(|| anyhow!("package path has no parent: {:?}", package_path))?;

    let restart_script = temp_root.join("restart.ps1");
    fs::write(&restart_script, RESTART_SCRIPT)
        .with_context(|| "failed to write update helper script")?;

    Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(&restart_script)
        .arg(std::process::id().to_string())
        .arg(package_path)
        .arg(&running_path)
        .arg(temp_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    exit(0);
}
