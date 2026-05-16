// On Windows do NOT show a console window when opening the app
// #![cfg_attr(all(not(test), target_os = "windows"), windows_subsystem = "windows")]

use crate::actions::{DarkTheme, LightTheme, SystemTheme, register_actions};
use crate::main_window::MainWindow;
use cntp_i18n::{I18N_MANAGER, tr, tr_load, tr_noop};
use cntp_icon_tool_macros::application_icon;
use contemporary::application::new_contemporary_application;
use contemporary::macros::application_details;
use contemporary::self_update::init_self_update;
use contemporary::setup_parlance::setup_parlance_i18n_if_enabled;
use contemporary::tokio::tokio_helper::TokioHelper;
use contemporary::{
    application::{ApplicationLink, Details, License, Versions},
    setup::{Contemporary, ContemporaryMenus, setup_contemporary},
    window::contemporary_window_options,
};
use gpui::http_client::{Url, anyhow};
use gpui::private::anyhow;
use gpui::{
    App, AsyncApp, Bounds, Menu, MenuItem, WeakEntity, WindowBounds, WindowOptions, px, size,
};
use indexmap::IndexMap;
use smol_macros::main;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use tracing::error;

mod actions;
mod components;
mod main_surface;
mod main_window;
mod patterns;

tr_noop!("THEME_SYSTEM", "System");
tr_noop!("THEME_LIGHT", "Light");
tr_noop!("THEME_DARK", "Dark");

fn mane() {
    application_icon!("../dist/baseicon.svg");
    new_contemporary_application().run(|cx: &mut App| {
        I18N_MANAGER.load_source(tr_load!());
        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

        let outer_window: Rc<RefCell<WeakEntity<MainWindow>>> =
            Rc::new(RefCell::new(WeakEntity::new_invalid()));

        setup_parlance_i18n_if_enabled(
            Url::parse("https://parlance.vicr123.com/").unwrap(),
            "contemporary-rust",
            "contemporary-playground-rust",
            "playground",
            cx,
        );

        setup_contemporary(
            cx,
            Contemporary {
                details: Details {
                    generatable: application_details!(),
                    copyright_holder: "Victor Tran",
                    copyright_year: "2026",
                    application_version: "3.0",
                    license: License::Gpl3OrLater,
                    links: IndexMap::from([
                        (
                            ApplicationLink::HelpContents,
                            "https://github.com/vicr123/contemporary-rs/",
                        ),
                        (
                            ApplicationLink::FileBug,
                            "https://github.com/vicr123/contemporary-rs/issues",
                        ),
                        (
                            ApplicationLink::SourceCode,
                            "https://github.com/vicr123/contemporary-rs",
                        ),
                    ]),
                },
                menus: ContemporaryMenus {
                    menus: vec![Menu {
                        name: tr!("MENU_THEME", "Theme").into(),
                        items: vec![
                            MenuItem::action(tr!("THEME_SYSTEM"), SystemTheme),
                            MenuItem::action(tr!("THEME_LIGHT"), LightTheme),
                            MenuItem::action(tr!("THEME_DARK"), DarkTheme),
                        ],
                        disabled: false,
                    }],
                    on_about: Rc::new({
                        let outer_window = outer_window.clone();
                        move |cx| {
                            outer_window
                                .borrow()
                                .upgrade()
                                .unwrap()
                                .update(cx, |window, cx| {
                                    window.about_surface_open(true);
                                    cx.notify()
                                })
                        }
                    }),
                    on_settings: Some(Rc::new({
                        let outer_window = outer_window.clone();
                        move |cx| {
                            outer_window
                                .borrow()
                                .upgrade()
                                .unwrap()
                                .update(cx, |window, cx| {
                                    window.settings_surface_open(true);
                                    cx.notify()
                                })
                        }
                    })),
                },
            },
        );

        init_self_update(
            Url::from_str("https://binchicken.vicr123.com").unwrap(),
            "contemporary_playground",
            option_env!("BIN_CHICKEN_UUID"),
            option_env!("BIN_CHICKEN_SIGNATURE_PUBLIC_KEY"),
            cx,
        );

        let default_window_options = contemporary_window_options(cx, "Contemporary Playground");
        register_actions(cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..default_window_options
            },
            |_, cx| {
                let window = MainWindow::new(cx);
                *outer_window.borrow_mut() = window.downgrade();

                let versions = cx.global::<Versions>();
                versions
                    .versions
                    .lock()
                    .unwrap()
                    .insert("version thing".into(), "1.0".into());

                window
            },
        )
        .unwrap();
        cx.activate(true);
    });
}

main! {
    async fn main() {
        mane()
    }
}
