use clap::Parser;
use clap_cargo::style::CLAP_STYLING;
use clap_verbosity_flag::InfoLevel;
use clap_verbosity_flag::tracing::Level;
use cntp_bundle_lib::tool_setup::{DeploymentType, setup_tool};
use current_platform::CURRENT_PLATFORM;
use std::collections::HashMap;
use std::env::consts::EXE_EXTENSION;
use std::path::PathBuf;
use std::process::exit;
use tracing::{error, info};

#[cfg(target_os = "linux")]
use cntp_bundle_lib::linux::bundle::bundle_linux;

#[cfg(target_os = "macos")]
use cntp_bundle_lib::macos::bundle::bundle_macos;

#[cfg(target_os = "windows")]
use cntp_bundle_lib::windows::bundle::bundle_windows;

#[derive(Parser, Debug)]
#[command(name = "cargo cntp-bundle")] // all of this is necessary so things work as expected wrt. cargo
#[command(bin_name = "cargo")]
#[command(styles = CLAP_STYLING)]
enum Command {
    CntpBundle(Args),
}

// #[command(version, about, long_about = None)]
#[derive(Parser, Debug)]
struct Args {
    /// The profile to build
    #[arg(short, long)]
    profile: Option<String>,

    /// The targets to build for
    #[arg(short, long)]
    target: Vec<String>,

    /// How loud should we be?
    #[clap(flatten)]
    verbosity: clap_verbosity_flag::Verbosity<InfoLevel>,

    /// Don't open the output directory after deployment is complete
    #[clap(long, default_value_t = false)]
    no_open: bool,
}

fn main() {
    let Command::CntpBundle(args) = Command::parse();

    tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .with_max_level(args.verbosity.tracing_level().or_else(|| {
            if std::env::var("RUNNER_DEBUG").is_ok_and(|runner_debug| runner_debug == "1") {
                Some(Level::DEBUG)
            } else {
                None
            }
        }))
        .init();

    let setup_data = setup_tool(args.profile, args.target, "bundle");

    info!(
        "Bundling {}",
        setup_data.cargo_metadata.root_package().unwrap().name
    );
    info!("Profile: {}", setup_data.profile);
    info!("Target:  {}", setup_data.targets.join(";"));
    info!("Output:  {}", setup_data.output_directory.display());

    // Find the executable(s) to publish
    let mut bin_targets: HashMap<String, PathBuf> = HashMap::new();
    for target in &setup_data.targets {
        let mut target_directory = setup_data.cargo_metadata.target_directory.clone();
        if target != CURRENT_PLATFORM {
            target_directory = target_directory.join(target.clone());
        }
        target_directory = target_directory.join(setup_data.profile.clone());

        let bin_target = target_directory
            .join(
                setup_data
                    .cargo_metadata
                    .root_package()
                    .unwrap()
                    .name
                    .as_str(),
            )
            .with_extension(EXE_EXTENSION);

        // TODO: Run cargo build
        if !bin_target.exists() {
            error!("Unable to find executable at {}", bin_target);
            exit(1);
        }

        bin_targets.insert(target.clone(), bin_target.into());
    }

    match setup_data.deployment_type {
        DeploymentType::Linux => {
            #[cfg(target_os = "linux")]
            bundle_linux(&setup_data, bin_targets);

            #[cfg(not(target_os = "linux"))]
            {
                error!("Unable to bundle for Linux on non-Linux platform");
                exit(1);
            }
        }
        DeploymentType::MacOS => {
            #[cfg(target_os = "macos")]
            bundle_macos(&setup_data, bin_targets);

            #[cfg(not(target_os = "macos"))]
            {
                error!("Unable to bundle for macOS on non-Macintosh platform");
                exit(1);
            }
        }
        DeploymentType::Windows => {
            #[cfg(target_os = "windows")]
            bundle_windows(&setup_data, bin_targets);

            #[cfg(not(target_os = "windows"))]
            {
                error!("Unable to bundle for Windows on non-Windows platform");
                exit(1);
            }
        }
    }

    if !args.no_open {
        let _ = open::that(setup_data.output_directory);
    }
}
