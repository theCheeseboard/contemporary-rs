use crate::components::anchorer::WithAnchorer;
use crate::components::application_menu::{ApplicationMenu, update_notification};
use crate::components::button::button;
use crate::components::flyout::flyout;
use crate::components::icon::icon;
use crate::jobs::job_button::JobButton;
use crate::styling::theme::ThemeStorage;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, Entity, ImageSource, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, Resource, SharedString, Styled, Window, WindowControlArea, div, img,
    px, rgb, svg,
};

#[derive(IntoElement)]
pub struct Surface {
    child: AnyElement,
    actions: AnyElement,
    application_menu: Option<Entity<ApplicationMenu>>,
}

pub fn surface() -> Surface {
    Surface {
        child: div().into_any_element(),
        actions: div().into_any_element(),
        application_menu: None,
    }
}

impl Surface {
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = child.into_any_element();
        self
    }

    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = actions.into_any_element();
        self
    }

    pub fn application_menu(mut self, application_menu: Entity<ApplicationMenu>) -> Self {
        self.application_menu = Some(application_menu);
        self
    }
}

impl RenderOnce for Surface {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .absolute()
            .top(px(0.))
            .left(px(0.))
            .w_full()
            .h_full()
            .occlude()
            .bg(theme.background)
            .child(self.child)
            .child(window_controls(self.actions, self.application_menu))
    }
}

#[derive(IntoElement)]
struct WindowTitle {
    actions: AnyElement,
    application_menu: Option<Entity<ApplicationMenu>>,
}

fn window_controls(
    actions: AnyElement,
    application_menu: Option<Entity<ApplicationMenu>>,
) -> WindowTitle {
    WindowTitle {
        actions,
        application_menu,
    }
}

#[allow(unreachable_code)]
impl RenderOnce for WindowTitle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let job_button = JobButton::use_job_button(window, cx);

        let is_update_available = is_update_not_idle(cx);
        let update_flyout_open_state = window.use_state(cx, |_, _| false);

        let theme = cx.theme();

        div()
            .id("contemporary-window-title")
            .absolute()
            .top(px(0.))
            .left(px(0.))
            .w_full()
            .h(px(40.))
            .flex()
            .gap(px(6.))
            .child(if cfg!(target_os = "macos") {
                // Make space for the window controls
                div().when(!window.is_fullscreen(), |div| div.w(px(70.)))
            } else {
                div()
                    .child(
                        button("window-menu")
                            .not_focusable()
                            .flat()
                            .w(px(40.))
                            .h(px(40.))
                            .child(
                                img(ImageSource::Resource(Resource::Embedded(
                                    SharedString::new_static("contemporary-icon:/application"),
                                )))
                                .w(px(24.))
                                .h(px(24.)),
                            )
                            .when(is_update_available, |david| {
                                david.child(
                                    div()
                                        .absolute()
                                        .rounded(px(4.))
                                        .size(px(8.))
                                        .right(px(8.))
                                        .bottom(px(8.))
                                        .bg(rgb(0x0064c8)),
                                )
                            })
                            .when_some(self.application_menu, |this, application_menu| {
                                this.child(application_menu.clone()).on_click(
                                    move |_, window, cx| {
                                        let focus_handle = window.focused(cx);
                                        application_menu.update(cx, |this, cx| {
                                            println!("Opening application menu");
                                            this.open(focus_handle);
                                            cx.notify();
                                        })
                                    },
                                )
                            }),
                    )
                    .occlude()
            })
            .child(
                div()
                    .flex()
                    .h(px(40.))
                    .flex_grow(1.)
                    .content_stretch()
                    .child(self.actions),
            )
            .window_control_area(WindowControlArea::Drag)
            .child(
                div()
                    .flex()
                    .occlude()
                    .when(
                        cfg!(target_os = "macos") && is_update_available,
                        move |david| {
                            david
                                .child(
                                    button("update-available")
                                        .flat()
                                        .w(px(40.))
                                        .h(px(40.))
                                        .child(icon("cloud-download").size(24.))
                                        .on_click({
                                            let update_flyout_open_state =
                                                update_flyout_open_state.clone();
                                            move |_, _, cx| {
                                                update_flyout_open_state.write(cx, true);
                                            }
                                        }),
                                )
                                .with_anchorer(move |david, bounds, _, cx| {
                                    let update_notification = update_notification(cx);
                                    david.child(
                                        flyout(bounds)
                                            .visible(*update_flyout_open_state.read(cx))
                                            .anchor_bottom_right()
                                            .w(px(400.))
                                            .child(
                                                div()
                                                    .occlude()
                                                    .p(px(4.))
                                                    .child(update_notification.unwrap_or(div())),
                                            )
                                            .on_close(move |_, _, cx| {
                                                update_flyout_open_state.write(cx, false);
                                            }),
                                    )
                                })
                        },
                    )
                    .child(job_button.clone())
                    .when(!cfg!(target_os = "macos"), |david| {
                        david
                            .child(
                                button("window-minimise")
                                    .flat()
                                    .w(px(40.))
                                    .h(px(40.))
                                    .child(
                                        svg()
                                            .w(px(24.))
                                            .h(px(24.))
                                            .text_color(theme.foreground)
                                            .path("window-controls:/min"),
                                    )
                                    .on_click(move |_, window, _| window.minimize_window())
                                    .window_control_area(WindowControlArea::Min),
                            )
                            .child(
                                button("window-maximise")
                                    .flat()
                                    .w(px(40.))
                                    .h(px(40.))
                                    .child(
                                        svg()
                                            .w(px(24.))
                                            .h(px(24.))
                                            .text_color(theme.foreground)
                                            .path(if window.is_maximized() {
                                                "window-controls:/res"
                                            } else {
                                                "window-controls:/max"
                                            }),
                                    )
                                    .on_click(move |_, window, _| window.zoom_window())
                                    .window_control_area(WindowControlArea::Max),
                            )
                            .child(
                                button("window-close")
                                    .flat()
                                    .w(px(40.))
                                    .h(px(40.))
                                    .child(
                                        svg()
                                            .w(px(24.))
                                            .h(px(24.))
                                            .text_color(theme.foreground)
                                            .path("window-controls:/close"),
                                    )
                                    .on_click(move |_, _, cx| cx.quit())
                                    .window_control_area(WindowControlArea::Close),
                            )
                    }),
            )
            .on_mouse_down(MouseButton::Left, move |_, window, _| {
                window.start_window_move()
            })
    }
}

fn is_update_not_idle(cx: &mut App) -> bool {
    #[cfg(feature = "self-update")]
    {
        return cx
            .try_global::<crate::self_update::SelfUpdate>()
            .map(|self_update| self_update.state().is_visible())
            .unwrap_or(false);
    }

    false
}
