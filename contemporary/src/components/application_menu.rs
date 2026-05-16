#[cfg(feature = "self-update")]
pub mod update_notification;

use crate::application::{ApplicationLink, Details};
use crate::components::button::button;
use crate::components::icon::icon;
use crate::components::icon_text::icon_text;
use crate::components::scrim::scrim;
use crate::setup::{About, OpenLink};
use crate::styling::theme::{ThemeStorage, VariableColor};
use cntp_i18n::{I18N_MANAGER, tr};
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, ClickEvent, Context, Div, Entity, FocusHandle, ImageSource,
    InteractiveElement, IntoElement, Menu, MenuItem, OwnedMenu, OwnedMenuItem, ParentElement,
    Render, RenderOnce, Resource, SharedString, Styled, Window, div, img, px,
};
use std::rc::Rc;

pub struct ApplicationMenu {
    is_open: bool,
    focus_handle: Option<FocusHandle>,
    root_menu: Rc<OwnedMenu>,
    menu_stack: Vec<Rc<OwnedMenu>>,
}

impl ApplicationMenu {
    pub fn new(cx: &mut App, menu: Menu) -> Entity<Self> {
        let root_menu = Rc::new(menu.owned());
        cx.new(|_cx| ApplicationMenu {
            is_open: false,
            focus_handle: None,
            root_menu,
            menu_stack: vec![],
        })
    }

    pub fn open(&mut self, focus_handle: Option<FocusHandle>) {
        self.is_open = true;
        self.focus_handle = focus_handle;
    }

    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_open = false;
        if let Some(focus_handle) = self.focus_handle.as_ref() {
            focus_handle.focus(window, cx);
        };
        self.menu_stack.clear();
    }
}

impl Render for ApplicationMenu {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.is_open {
            return div();
        }

        let update_notification = update_notification(cx);

        let theme = cx.theme();
        let details = cx.global::<Details>();

        let locale = I18N_MANAGER.locale();

        div().child(
            scrim("application-menu")
                .visible(self.is_open)
                .on_click(cx.listener(|this, _, window, cx| {
                    this.close(window, cx);
                    cx.notify();
                }))
                .child(
                    div()
                        .bg(theme.background)
                        .border_color(theme.border_color)
                        .border(px(1.))
                        .rounded(theme.border_radius)
                        .occlude()
                        .flex()
                        .flex_col()
                        .w(px(300.))
                        .child(
                            div()
                                .flex()
                                .p(px(7.))
                                .gap(px(6.))
                                .child(
                                    img(ImageSource::Resource(Resource::Embedded(
                                        SharedString::new_static("contemporary-icon:/application"),
                                    )))
                                    .w(px(24.))
                                    .h(px(24.)),
                                )
                                .child(
                                    details
                                        .generatable
                                        .application_name
                                        .resolve_languages_or_default(&locale.messages),
                                ),
                        )
                        .when_else(
                            self.menu_stack.is_empty(),
                            |david| {
                                david.when_some(update_notification, |div, update_notification| {
                                    div.child(update_notification)
                                })
                            },
                            |div| {
                                div.child(
                                    button("back-button")
                                        .child(icon_text(
                                            "go-previous",
                                            tr!("MENU_GO_BACK", "Back"),
                                        ))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.menu_stack.pop();
                                            cx.notify();
                                        })),
                                )
                            },
                        )
                        .child(
                            menu_list(
                                self.menu_stack
                                    .last()
                                    .cloned()
                                    .unwrap_or(self.root_menu.clone()),
                                self.focus_handle.clone(),
                            )
                            .on_menu_click(cx.listener(|this, event: &SubmenuClickEvent, _, cx| {
                                this.menu_stack.push(event.menu.clone());
                                cx.notify();
                            }))
                            .on_menu_should_close(cx.listener(|this, _, window, cx| {
                                this.close(window, cx);
                                cx.notify();
                            })),
                        )
                        .when(self.menu_stack.is_empty(), |david| {
                            david.child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .p(px(8.))
                                    .child(
                                        button("menu-help-button")
                                            .flat()
                                            .child(icon("help-contents"))
                                            .size(px(32.))
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                let details = cx.global::<Details>();

                                                let locale = I18N_MANAGER.locale();

                                                let mut menu_items: Vec<MenuItem> = details
                                                    .links
                                                    .iter()
                                                    .flat_map(|(key, url)| {
                                                        if *key == ApplicationLink::HelpContents {
                                                            [
                                                                Some(MenuItem::action(
                                                                    tr!(
                                                                        "MENU_HELP_CONTENTS",
                                                                        application = details
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

                                                if !menu_items.is_empty() {
                                                    menu_items.push(MenuItem::separator())
                                                }

                                                menu_items.push(MenuItem::action(
                                                    tr!(
                                                        "APPLE_APP_MENU_ABOUT",
                                                        application = details
                                                            .generatable
                                                            .application_name
                                                            .resolve_languages_or_default(
                                                                &locale.messages
                                                            )
                                                    ),
                                                    About,
                                                ));

                                                this.menu_stack.push(Rc::new(
                                                    Menu {
                                                        name: "Help".into(),
                                                        items: menu_items,
                                                        disabled: false,
                                                    }
                                                    .owned(),
                                                ));
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        button("menu-exit-button")
                                            .flat()
                                            .child(icon("application-exit"))
                                            .size(px(32.))
                                            .on_click(|_, _, cx| {
                                                cx.quit();
                                            }),
                                    ),
                            )
                        }),
                ),
        )
    }
}

pub fn update_notification(cx: &mut App) -> Option<Div> {
    #[cfg(feature = "self-update")]
    {
        return cx
            .try_global::<crate::self_update::SelfUpdate>()
            .map(|self_update| self_update.state())
            .and_then(|state| {
                if state.is_visible() {
                    Some(div().child(update_notification::UpdateNotification))
                } else {
                    None
                }
            });
    }

    None
}

#[derive(Clone)]
struct SubmenuClickEvent {
    pub menu: Rc<OwnedMenu>,
}
type SubmenuClickListener = Box<dyn Fn(&SubmenuClickEvent, &mut Window, &mut App) + 'static>;
type MenuShouldCloseListener = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
struct MenuList {
    menu: Rc<OwnedMenu>,
    menu_click_listeners: Vec<SubmenuClickListener>,
    menu_should_close_listeners: Vec<MenuShouldCloseListener>,
    focus_handle: Option<FocusHandle>,
}

fn menu_list(menu: Rc<OwnedMenu>, focus_handle: Option<FocusHandle>) -> MenuList {
    MenuList {
        menu,
        menu_click_listeners: vec![],
        menu_should_close_listeners: vec![],
        focus_handle,
    }
}

impl MenuList {
    pub fn on_menu_click<F: 'static>(mut self, listener: F) -> Self
    where
        F: Fn(&SubmenuClickEvent, &mut Window, &mut App),
    {
        self.menu_click_listeners.push(Box::new(listener));
        self
    }

    pub fn on_menu_should_close<F: 'static>(mut self, listener: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App),
    {
        self.menu_should_close_listeners.push(Box::new(listener));
        self
    }
}

impl RenderOnce for MenuList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let submenu_click_listeners = Rc::new(self.menu_click_listeners);
        let menu_should_close_listeners = Rc::new(self.menu_should_close_listeners);

        self.menu.items.iter().fold(div(), |david, item| {
            david.child(match item.clone() {
                OwnedMenuItem::Separator => {
                    div().h(px(1.)).bg(theme.border_color).into_any_element()
                }
                OwnedMenuItem::Submenu(menu) => {
                    let submenu_click_listeners = submenu_click_listeners.clone();
                    button(menu.name.clone())
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .items_center()
                                .justify_between()
                                .text_ellipsis()
                                .overflow_hidden()
                                .child(menu.name.clone())
                                .child(icon("go-next")),
                        )
                        .flat()
                        .on_click(move |_event, window, cx| {
                            let submenu_click_event = SubmenuClickEvent {
                                menu: Rc::new(menu.clone()),
                            };
                            for listener in submenu_click_listeners.iter() {
                                listener(&submenu_click_event, window, cx);
                            }
                        })
                        .into_any_element()
                }
                OwnedMenuItem::Action {
                    name,
                    action,
                    disabled, checked,
                    os_action: _os_action,
                } => {
                    let button_id: SharedString = format!("menu-item-{name}").into();
                    let menu_should_close_listeners = menu_should_close_listeners.clone();
                    let focus_handle = self.focus_handle.clone();

                    let keybind = window
                        .bindings_for_action(action.as_ref())
                        .first()
                        .map(|key| key.keystrokes())
                        .map(|keystrokes| {
                            if keystrokes.len() == 1 {
                                let mut parts: Vec<String> = Vec::new();
                                let keystroke = keystrokes.first().unwrap();
                                if keystroke.modifiers().control {
                                    parts.push(tr!("KEY_CONTROL", "Ctrl", #description="Control key, as shown next to menu items").into());
                                }
                                if keystroke.modifiers().shift {
                                    parts.push(tr!("KEY_SHIFT", "Shift", #description="Shift key, as shown next to menu items").into());
                                }
                                if keystroke.modifiers().alt {
                                    parts.push(tr!("KEY_ALT", "Alt", #description="Alt key, as shown next to menu items").into());
                                }
                                if keystroke.modifiers().platform {
                                    parts.push(tr!("KEY_PLATFORM", "Super", #description="Super key, as shown next to menu items").into());
                                }
                                if keystroke.modifiers().function {
                                    parts.push(tr!("KEY_FUNCTION", "Fn", #description="Function key, as shown next to menu items").into());
                                }
                                parts.push(keystroke.key().to_uppercase().clone());
                                parts.join("+")
                            } else {
                                String::new()
                            }
                        })
                        .unwrap_or_default();

                    button(button_id)
                        .when(disabled, |button| button.disabled())
                        // TODO: when checked
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(div().text_ellipsis().overflow_hidden().child(name))
                                .child(div().text_color(theme.foreground.disabled()).child(keybind)))
                        .flat()
                        .on_click(move |event, window, cx| {
                            if let Some(focus_handle) = focus_handle.as_ref() {
                                focus_handle.focus(window, cx);
                            };

                            let action = action.boxed_clone();
                            window.on_next_frame(|window, cx| {
                                window.dispatch_action(action, cx);
                            });

                            for listener in menu_should_close_listeners.iter() {
                                listener(event, window, cx);
                            }
                        })
                        .into_any_element()
                }
                OwnedMenuItem::SystemMenu(_) => {
                    div().into_any_element()
                }
            })
        })
    }
}
