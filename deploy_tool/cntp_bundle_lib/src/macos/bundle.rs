use crate::icon::get_svg_icon_contents;
use crate::tool_setup::ToolSetup;
use plist::{Dictionary, Value, to_file_xml};
use resvg::render;
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use std::collections::HashMap;
use std::fs::{OpenOptions, copy, create_dir_all, remove_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, exit};
use tempfile::TempDir;
use tracing::error;

pub fn bundle_macos(setup_data: &ToolSetup, executable_path: HashMap<String, PathBuf>) {
    let deployment = setup_data
        .contemporary_config
        .deployment(setup_data.targets.first().unwrap());

    let Ok(_) = create_dir_all(&setup_data.output_directory) else {
        error!("Failed to create output directory");
        exit(1);
    };

    let Some(application_name) = deployment.application_name() else {
        error!("No application name specified in config");
        exit(1);
    };

    let Some(desktop_entry) = deployment.desktop_entry else {
        error!("No desktop entry specified in config");
        exit(1);
    };

    let application_generic_name = deployment.application_generic_name;
    let extra_info_plist_attributes = deployment.extra_info_plist_attributes;

    let app_root = setup_data
        .output_directory
        .join(application_name.default_value())
        .with_extension("app");
    if app_root.exists() {
        remove_dir_all(&app_root).unwrap();
    };

    let contents_dir = app_root.join("Contents");
    let Ok(_) = create_dir_all(&contents_dir) else {
        error!("Failed to create application package");
        exit(1);
    };

    let macos_dir = contents_dir.join("MacOS");
    let Ok(_) = create_dir_all(&macos_dir) else {
        error!("Failed to create MacOS directory");
        exit(1);
    };

    let output_executable_path = macos_dir.join(application_name.default_value());
    if setup_data.targets.len() == 1 {
        let executable_path = executable_path
            .get(setup_data.targets.first().unwrap())
            .unwrap();
        let Ok(_) = copy(executable_path, output_executable_path) else {
            error!("Failed to copy executable to MacOS directory");
            exit(1);
        };
    } else {
        // Building a universal application bundle
        let Ok(status) = Command::new("lipo")
            .arg("-create")
            .arg("-output")
            .arg(&output_executable_path)
            .args(executable_path.values())
            .status()
        else {
            error!("Failed to run lipo command to create universal binary");
            exit(1);
        };

        if !status.success() {
            error!("Failed to create universal binary");
            exit(1);
        }
    }

    let resources_dir = contents_dir.join("Resources");
    let Ok(_) = create_dir_all(&resources_dir) else {
        error!("Failed to create Resources directory");
        exit(1);
    };

    let icon_svg = get_svg_icon_contents(
        setup_data.targets.first().unwrap(),
        &setup_data.base_path,
        &setup_data.contemporary_config,
    );
    let icon_path = resources_dir.join("icon.icns");
    create_icns_file(icon_path, icon_svg);

    let info_plist_path = contents_dir.join("Info.plist");

    let mut plist_root = Dictionary::new();
    plist_root.insert(
        "CFBundleName".to_string(),
        Value::String({
            let mut cf_bundle_name = application_name.default_value();
            cf_bundle_name.truncate(15);
            cf_bundle_name
        }),
    );
    plist_root.insert(
        "CFBundleDisplayName".to_string(),
        Value::String(application_name.default_value()),
    );
    plist_root.insert(
        "CFBundleExecutable".to_string(),
        Value::String(application_name.default_value()),
    );
    plist_root.insert(
        "CFBundleIdentifier".to_string(),
        Value::String(desktop_entry),
    );
    if let Some(ref application_generic_name) = application_generic_name {
        plist_root.insert(
            "CFBundleGetInfoString".to_string(),
            Value::String(application_generic_name.default_value()),
        );
    }
    plist_root.insert(
        "CFBundlePackageType".to_string(),
        Value::String("APPL".to_string()),
    );
    plist_root.insert(
        "LSMinimumSystemVersion".to_string(),
        Value::String(deployment.minimum_system_version),
    );
    plist_root.insert(
        "NSPrincipalClass".to_string(),
        Value::String("NSApplication".to_string()),
    );
    plist_root.insert(
        "NSSupportsAutomaticGraphicsSwitching".to_string(),
        Value::Boolean(deployment.supports_automatic_graphics_switching),
    );
    plist_root.insert(
        "CFBundleIconFile".to_string(),
        Value::String("icon.icns".to_string()),
    );
    plist_root.insert(
        "CFBundleVersion".to_string(),
        Value::String(format!(
            "{}.{}.{}",
            setup_data.version.0, setup_data.version.1, setup_data.version.2
        )),
    );
    // TODO: maybe allow overriding this?
    plist_root.insert(
        "CFBundleShortVersionString".to_string(),
        Value::String(format!(
            "{}.{}.{}",
            setup_data.version.0, setup_data.version.1, setup_data.version.2
        )),
    );

    let schemes = deployment
        .handled_url_schemes
        .unwrap_or_default()
        .iter()
        .map(|scheme| {
            let mut dict = Dictionary::new();
            dict.insert(
                "CFBundleURLSchemes".to_string(),
                Value::Array(vec![Value::String(scheme.clone())]),
            );
            Value::Dictionary(dict)
        })
        .collect::<Vec<_>>();
    if !schemes.is_empty() {
        plist_root.insert("CFBundleURLTypes".to_string(), Value::Array(schemes));
    }

    for (key, value) in &extra_info_plist_attributes {
        plist_root.insert(key.clone(), Value::String(value.default_value()));
    }

    let Ok(_) = to_file_xml(info_plist_path, &Value::Dictionary(plist_root)) else {
        error!("Failed to write Info.plist");
        exit(1);
    };

    // Create an InfoPlist.strings file for each localisation
    for localisation in setup_data.contemporary_config.available_localisations() {
        let lproj_dir = resources_dir.join(&localisation).with_extension("lproj");
        let Ok(_) = create_dir_all(&lproj_dir) else {
            error!(
                "Failed to create localisation directory for language {}",
                localisation
            );
            continue;
        };

        let info_plist_strings_path = lproj_dir.join("InfoPlist.strings");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(info_plist_strings_path)
            .unwrap();

        if let Some(application_name) = application_name.resolve_language(&localisation) {
            file.write_all(format!("CFBundleDisplayName = \"{application_name}\";\n").as_bytes())
                .unwrap();
            file.write_all(format!("CFBundleName = \"{application_name}\";\n").as_bytes())
                .unwrap();
        }

        if let Some(ref application_generic_name) = application_generic_name {
            if let Some(application_generic_name) =
                application_generic_name.resolve_language(&localisation)
            {
                file.write_all(
                    format!("CFBundleGetInfoString = \"{application_generic_name}\";\n").as_bytes(),
                )
                .unwrap();
            }
        }

        for (key, value) in &extra_info_plist_attributes {
            if let Some(value) = value.resolve_language(&localisation) {
                file.write_all(format!("{key} = \"{value}\";\n").as_bytes())
                    .unwrap();
            }
        }
    }
}

struct IconDimensions {
    size: u32,
    scale_factor: u32,
}

fn create_icns_file(icns_path: PathBuf, svg_data: String) {
    // Create a temporary directory
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.path();

    // Create iconset directory
    let icon_set_dir = temp_dir_path.join("icon.iconset");
    create_dir_all(&icon_set_dir).expect("Failed to create iconset directory");

    // Define icon dimensions
    let dimensions = vec![
        IconDimensions {
            size: 16,
            scale_factor: 1,
        },
        IconDimensions {
            size: 16,
            scale_factor: 2,
        },
        IconDimensions {
            size: 32,
            scale_factor: 1,
        },
        IconDimensions {
            size: 32,
            scale_factor: 2,
        },
        IconDimensions {
            size: 128,
            scale_factor: 1,
        },
        IconDimensions {
            size: 128,
            scale_factor: 2,
        },
        IconDimensions {
            size: 256,
            scale_factor: 1,
        },
        IconDimensions {
            size: 256,
            scale_factor: 2,
        },
        IconDimensions {
            size: 512,
            scale_factor: 1,
        },
        IconDimensions {
            size: 512,
            scale_factor: 2,
        },
    ];

    // Generate PNG files for each dimension
    for dim in dimensions {
        let filename = if dim.scale_factor == 1 {
            format!("icon_{}x{}.png", dim.size, dim.size)
        } else {
            format!("icon_{}x{}@{}x.png", dim.size, dim.size, dim.scale_factor)
        };

        let output_path = icon_set_dir.join(filename);

        let opt = Options::default();
        let tree =
            Tree::from_data(svg_data.as_bytes(), &opt).expect("Could not interpret built SVG data");
        let mut pixmap =
            Pixmap::new(dim.size, dim.size).expect("Could not create pixmap to hold PNG");
        render(
            &tree,
            Transform::from_scale(
                dim.size as f32 / tree.size().width(),
                dim.size as f32 / tree.size().height(),
            ),
            &mut pixmap.as_mut(),
        );
        pixmap.save_png(output_path).expect("Could not save PNG");
    }

    // Generate ICNS file using iconutil
    let status = Command::new("iconutil")
        .current_dir(temp_dir_path)
        .arg("-c")
        .arg("icns")
        .arg("icon.iconset")
        .status()
        .expect("Failed to execute iconutil");

    if !status.success() {
        panic!("Failed to generate ICNS file");
    }

    // Copy the generated ICNS file to the output location
    copy(temp_dir_path.join("icon.icns"), icns_path).expect("Failed to copy generated ICNS file");
}
