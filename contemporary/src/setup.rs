use crate::application::{ApplicationLink, Details, Versions};
use crate::components::context_menu::bind_context_menu_keys;
use crate::components::text_field::bind_text_field_keys;
use crate::jobs::job_manager::JobManager;
use crate::platform_support::platform_settings::PlatformSettings;
use crate::platform_support::setup_platform;
use crate::settings::SettingsManager;
use crate::styling::theme::Theme;
use crate::tokio::tokio_helper::TokioHelper;
use crate::tracing::application_log::ApplicationLog;
use crate::window::window_globals::WindowGlobals;
use cntp_i18n::{I18N_MANAGER, tr, tr_load};
use gpui::{Action, App, AsyncApp, Global, KeyBinding, Menu, MenuItem, SystemMenuType, actions};
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tracing::error;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};
use url::Url;

actions!(
    contemporary,
    [Quit, HideSelf, HideOthers, ShowAll, About, Settings]
);

#[derive(PartialEq, Clone, Default, Deserialize, JsonSchema, Action)]
pub struct OpenLink {
    #[serde(default)]
    pub(crate) link: String,
}

pub struct Contemporary {
    pub details: Details,
    pub menus: ContemporaryMenus,
}

pub struct ContemporaryMenus {
    pub menus: Vec<Menu>,
    pub on_about: Rc<dyn Fn(&mut App)>,
    pub on_settings: Option<Rc<dyn Fn(&mut App)>>,
}

struct Callbacks {
    pub on_about: Rc<dyn Fn(&mut App)>,
    pub on_settings: Option<Rc<dyn Fn(&mut App)>>,
}

impl Global for Callbacks {}

pub fn setup_contemporary(cx: &mut App, mut application: Contemporary) {
    let (_tracing_channel_tx, tracing_channel_rx) = async_channel::bounded(5);

    let application_log = ApplicationLog::new(cx, tracing_channel_rx);
    cx.set_global(application_log);

    tracing_subscriber::registry()
        // .with(ContemporaryLayer::new(tracing_channel_tx))
        .with(tracing_subscriber::fmt::layer().without_time().with_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        ))
        .init();

    bind_text_field_keys(cx);
    bind_context_menu_keys(cx);

    cx.on_action(quit);
    cx.on_action(hide_self);
    cx.on_action(hide_others);
    cx.on_action(show_all);
    cx.on_action(about);
    cx.on_action(settings);
    cx.on_action(open_link);
    cx.bind_keys([KeyBinding::new("cmd-h", HideSelf, None)]);
    cx.bind_keys([KeyBinding::new("cmd-alt-h", HideOthers, None)]);
    cx.bind_keys([KeyBinding::new("secondary-q", Quit, None)]);
    cx.bind_keys([KeyBinding::new("secondary-,", Settings, None)]);

    if let Some(link) = application
        .details
        .links
        .get(&ApplicationLink::HelpContents)
    {
        cx.bind_keys([KeyBinding::new(
            "F1",
            OpenLink {
                link: link.to_string(),
            },
            None,
        )])
    }

    I18N_MANAGER.load_source(tr_load!());

    let (i18n_cache_eviction_tx, i18n_cache_eviction_rx) = async_channel::bounded(1);
    cx.spawn(async move |cx: &mut AsyncApp| {
        while i18n_cache_eviction_rx.recv().await.is_ok() {
            cx.update(|cx| {
                // Redraw everything to use the new translations
                cx.refresh_windows();
            });
        }
    })
    .detach();
    I18N_MANAGER
        .write()
        .unwrap()
        .subscribe_to_cache_eviction(move || {
            let _ = smol::block_on(i18n_cache_eviction_tx.send(()));
        });

    let locale = I18N_MANAGER.locale();

    let mut application_menu_items = vec![
        MenuItem::action(
            tr!("APPLE_APP_MENU_ABOUT", "About {{application}}", application=application.details.generatable.application_name
                            .resolve_languages_or_default(&locale.messages),
                #description = "Please use the string that macOS uses for the About action in the application menu."
            ),
            About,
        ),
        MenuItem::separator(),
    ];
    if application.menus.on_settings.is_some() {
        application_menu_items.push(MenuItem::action(
            tr!("APPLE_APP_MENU_SETTINGS", "Settings...",
                    #description = "Please use the string that macOS uses for the Settings action in the application menu. Don't forget the ellipsis at the end."
                ),
            Settings,
        ));
        application_menu_items.push(MenuItem::separator());
    }
    application_menu_items.append(&mut vec![
        MenuItem::os_submenu(tr!("APPLE_APP_MENU_SERVICES", "Services",
                    #description = "Please use the string that macOS uses for the Services action in the application menu."
                ), SystemMenuType::Services),
        MenuItem::separator(),
        MenuItem::action(
            tr!("APPLE_APP_MENU_HIDE_SELF", "Hide {{application}}", application = application.details.generatable.application_name
                                .resolve_languages_or_default(&locale.messages),
                    #description = "Please use the string that macOS uses for the Hide this application action in the application menu."),
            HideSelf,
        ),
        MenuItem::action(
            tr!("APPLE_APP_MENU_HIDE_OTHERS", "Hide Others",
                    #description = "Please use the string that macOS uses for the Hide Others action in the application menu."),
            HideOthers,
        ),
        MenuItem::action(
            tr!("APPLE_APP_MENU_SHOW_ALL", "Show All",
                    #description = "Please use the string that macOS uses for the Show All action in the application menu."),
            ShowAll,
        ),
        MenuItem::separator(),
        MenuItem::action(
            tr!(
                    "APPLE_APP_MENU_QUIT",
                    "Quit {{application}}",
                    application = application.details.generatable.application_name
                                .resolve_languages_or_default(&locale.messages),
                    #description = "Please use the string that macOS uses for the Quit action in the application menu."
                ),
            Quit,
        ),
    ]);

    let mut menus = vec![Menu {
        name: application
            .details
            .generatable
            .application_name
            .resolve_languages_or_default(&locale.messages)
            .into(),
        items: application_menu_items,
        disabled: false,
    }];
    let window_menu = application
        .menus
        .menus
        .iter()
        .position(|menu| menu.name == "Window");
    if let Some(window_menu_index) = window_menu {
        let window_menu = &mut application.menus.menus[window_menu_index];
        window_menu.name = tr!("WINDOW_MENU").into();
    }
    menus.append(&mut application.menus.menus);
    if window_menu.is_none() {
        // Create a default "Window" menu
        menus.push(Menu {
            name: tr!("WINDOW_MENU", "Window",
                #description = "Please use the string that macOS uses for the Window menu."
            )
            .into(),
            items: vec![],
            disabled: false,
        })
    }

    let help_menu_items = application
        .details
        .links
        .iter()
        .flat_map(|(key, url)| {
            if *key == ApplicationLink::HelpContents {
                [
                    Some(MenuItem::action(
                        tr!(
                            "MENU_HELP_CONTENTS",
                            "{{application}} Help",
                            application = application
                                .details
                                .generatable
                                .application_name
                                .resolve_languages_or_default(&locale.messages)
                        ),
                        OpenLink {
                            link: url.to_string(),
                        },
                    )),
                    Some(MenuItem::separator()),
                ]
            } else {
                [
                    Some(MenuItem::action(
                        key.get_name(),
                        OpenLink {
                            link: url.to_string(),
                        },
                    )),
                    None,
                ]
            }
        })
        .flatten()
        .collect();

    menus.push(Menu {
        name: tr!("MENU_HELP", "Help",
            #description = "Please use the string that macOS uses for the Help menu."
        )
        .into(),
        items: help_menu_items,
        disabled: false,
    });

    cx.set_menus(menus);

    cx.set_global(application.details);
    cx.set_global(Theme::default());
    cx.set_global(PlatformSettings::default());
    cx.set_global(Callbacks {
        on_about: application.menus.on_about,
        on_settings: application.menus.on_settings,
    });
    cx.set_global(Versions {
        contemporary_version: "alpha",
        versions: Arc::new(Mutex::new(HashMap::new())),
    });
    cx.set_global(JobManager::new());
    cx.set_global(WindowGlobals::new());

    setup_platform(cx);

    #[cfg(feature = "tokio")]
    crate::tokio::init(cx);

    #[cfg(feature = "parlance")]
    crate::setup_parlance::setup_parlance_i18n_if_enabled(
        Url::parse("https://parlance.vicr123.com/").unwrap(),
        "contemporary-rust",
        "contemporary-rust",
        "contemporary",
        cx,
    );
}

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn hide_self(_: &HideSelf, cx: &mut App) {
    cx.hide();
}

fn hide_others(_: &HideOthers, cx: &mut App) {
    cx.hide_other_apps();
}

fn show_all(_: &ShowAll, cx: &mut App) {
    cx.unhide_other_apps();
}

fn about(_: &About, cx: &mut App) {
    let callbacks = cx.global::<Callbacks>();
    callbacks.on_about.clone()(cx);
}

fn settings(_: &Settings, cx: &mut App) {
    let callbacks = cx.global::<Callbacks>();
    if let Some(on_settings) = &callbacks.on_settings {
        on_settings.clone()(cx);
    }
}

fn open_link(action: &OpenLink, cx: &mut App) {
    cx.open_url(&action.link);
}
