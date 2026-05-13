use crate::copy_dir_all::copy_dir_all;
use crate::tool_setup::ToolSetup;
use std::fs::exists;
use std::path::PathBuf;
use std::process::exit;
use tracing::{error, info};

pub fn deploy_rootdir(setup_data: &ToolSetup, output_file: &str) {
    let appdir_root = setup_data.output_directory.join("appdir");
    if !appdir_root.exists() {
        error!("AppDir does not exist. Please bundle first.");
        exit(1);
    }

    let output_file = PathBuf::from(output_file);
    match exists(&output_file) {
        Ok(true) => {
            // Ensure it is a directory
            if !output_file.is_dir() {
                error!(
                    "Output file exists, but is not a directory: {}",
                    output_file.display()
                );
                exit(1);
            }
        }
        Ok(false) => {
            // Create the directory
            if let Err(e) = std::fs::create_dir(&output_file) {
                error!("Failed to create output directory: {}", e);
                exit(1);
            }
        }
        Err(e) => {
            error!("Failed to check if output file exists: {}", e);
            exit(1);
        }
    }
    
    let appdir_usr = appdir_root.join("usr");

    // Recursively walk the appdir_root and copy each file to the output directory, preserving the directory structure and permissions
    if let Err(e) = copy_dir_all(&appdir_usr, &output_file, |_| true) {
        error!("Failed to copy deployment files: {}", e);
        exit(1);
    }

    info!("Copied deployment files to {}.", output_file.display());
}
