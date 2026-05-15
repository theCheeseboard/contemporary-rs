pub mod appstream_metainfo;
pub mod desktop_entry;
pub mod shared_libraries;

use crate::icon::get_svg_icon_contents;
use crate::linux::APPRUN_TEMPLATE;
use crate::linux::bundle::appstream_metainfo::copy_appstream_metainfo;
use crate::linux::bundle::desktop_entry::generate_desktop_entry;
use crate::linux::bundle::shared_libraries::copy_shared_libraries;
use crate::tool_setup::ToolSetup;
use cntp_config::{ContemporaryConfig, LocalisedString};
use resvg::render;
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use std::collections::HashMap;
use std::fmt::Error;
use std::fs::{Permissions, copy, create_dir_all, remove_dir_all, set_permissions, write};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::exit;
use tracing::error;

pub fn bundle_linux(setup_data: &ToolSetup, executable_path: HashMap<String, PathBuf>) {
    let target_triple = setup_data.targets.first().unwrap();
    let executable_path = executable_path.get(target_triple).unwrap();

    let deployment = setup_data.contemporary_config.deployment(target_triple);

    let Some(desktop_entry) = deployment.desktop_entry else {
        error!("No desktop entry specified in config");
        exit(1);
    };

    let desktop_entry_with_desktop_extension = desktop_entry.clone() + ".desktop";
    let desktop_entry_with_svg_extension = desktop_entry.clone() + ".svg";
    let desktop_entry_with_metainfo_extension = desktop_entry.clone() + ".metainfo.xml";

    let Ok(_) = create_dir_all(&setup_data.output_directory) else {
        error!("Failed to create output directory");
        exit(1);
    };

    let appdir_root = setup_data.output_directory.join("appdir");
    if appdir_root.exists() {
        let Ok(_) = remove_dir_all(&appdir_root) else {
            error!("Failed to remove existing appdir");
            exit(1);
        };
    }

    let appdir_usr = appdir_root.join("usr");
    let appdir_bin = appdir_usr.join("bin");
    let Ok(_) = create_dir_all(&appdir_bin) else {
        error!("Failed to create appdir bin folder");
        exit(1);
    };

    let Ok(_) = copy(
        executable_path,
        appdir_bin.join(executable_path.file_name().unwrap()),
    ) else {
        error!("Failed to copy executable to bin directory");
        exit(1);
    };

    let appdir_lib = appdir_usr.join("lib");
    let Ok(_) = create_dir_all(&appdir_lib) else {
        error!("Failed to create appdir lib folder");
        exit(1);
    };

    if let Err(e) = copy_shared_libraries(executable_path, &appdir_lib) {
        error!("Failed to copy shared libraries: {}", e);
        exit(1);
    }

    let appdir_share = appdir_usr.join("share");
    let appdir_share_applications = appdir_share.join("applications");
    let Ok(_) = create_dir_all(&appdir_share_applications) else {
        error!("Failed to create appdir applications folder");
        exit(1);
    };

    let appdir_scalable_app_icons = appdir_share
        .join("icons")
        .join("hicolor")
        .join("scalable")
        .join("apps");
    let Ok(_) = create_dir_all(&appdir_scalable_app_icons) else {
        error!("Failed to create appdir icons folder");
        exit(1);
    };

    let apprun_path = appdir_root.join("AppRun");
    let apprun_contents = APPRUN_TEMPLATE.replace(
        "{{APPLICATION_PAYLOAD}}",
        &PathBuf::from("usr/bin")
            .join(executable_path.file_name().unwrap())
            .to_string_lossy(),
    );
    let Ok(_) = write(&apprun_path, apprun_contents) else {
        error!("Failed to write AppRun");
        exit(1);
    };
    let Ok(_) = set_permissions(&apprun_path, Permissions::from_mode(0o755)) else {
        error!("Failed to set permissions on AppRun");
        exit(1);
    };

    let Ok(desktop_entry_contents) = generate_desktop_entry(
        target_triple,
        executable_path,
        &setup_data.contemporary_config,
    ) else {
        error!("Failed to generate desktop entry");
        exit(1);
    };

    let desktop_entry_path = appdir_share_applications.join(&desktop_entry_with_desktop_extension);
    let Ok(_) = write(&desktop_entry_path, desktop_entry_contents) else {
        error!("Failed to write desktop entry");
        exit(1);
    };

    let root_desktop_entry_path = appdir_root.join(&desktop_entry_with_desktop_extension);
    let Ok(_) = symlink(
        PathBuf::from("usr/share/applications").join(&desktop_entry_with_desktop_extension),
        root_desktop_entry_path,
    ) else {
        error!("Failed to create desktop entry symlink");
        exit(1);
    };

    let icon_svg = get_svg_icon_contents(
        target_triple,
        &setup_data.base_path,
        &setup_data.contemporary_config,
    );
    let Ok(_) = write(
        appdir_scalable_app_icons.join(&desktop_entry_with_svg_extension),
        &icon_svg,
    ) else {
        error!("Failed to write SVG icon");
        exit(1);
    };

    let diricon_path = appdir_root.join(".DirIcon");
    {
        let opt = Options::default();
        let tree =
            Tree::from_data(icon_svg.as_bytes(), &opt).expect("Could not interpret built SVG data");
        let mut pixmap = Pixmap::new(256, 256).expect("Could not create pixmap to hold PNG");
        render(
            &tree,
            Transform::from_scale(256. / tree.size().width(), 256. / tree.size().height()),
            &mut pixmap.as_mut(),
        );
        pixmap.save_png(diricon_path).expect("Could not save PNG");
    }

    let root_icon_path = appdir_root.join(&desktop_entry_with_svg_extension);
    let Ok(_) = symlink(
        PathBuf::from("usr/share/icons/hicolor/scalable/apps")
            .join(&desktop_entry_with_svg_extension),
        root_icon_path,
    ) else {
        error!("Failed to create icon symlink");
        exit(1);
    };

    if let Some(appstream_metainfo_file) = deployment.appstream_metainfo_file {
        let appdir_share_metainfo = appdir_share.join("metainfo");
        let Ok(_) = create_dir_all(&appdir_share_metainfo) else {
            error!("Failed to create appdir metainfo folder");
            exit(1);
        };
        let metainfo_path = appdir_share_metainfo.join(&desktop_entry_with_metainfo_extension);

        let input_path = setup_data.base_path.join(appstream_metainfo_file);

        if let Err(e) =
            copy_appstream_metainfo(&input_path, &metainfo_path, &setup_data.contemporary_config)
        {
            error!("Failed to write appstream metainfo: {}", e);
            exit(1);
        }
    }
}
