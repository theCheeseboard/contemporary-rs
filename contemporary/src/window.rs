mod toast_drawer;
pub(crate) mod window_globals;

use crate::platform_support::platform_settings::PlatformSettings;
use crate::styling::theme::ThemeStorage;
use crate::window::toast_drawer::ToastDrawer;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, Bounds, CursorStyle, Decorations, Div, Hitbox, HitboxBehavior,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Point, RenderOnce,
    ResizeEdge, SharedString, Size, Styled, Tiling, TitlebarOptions, Window, WindowBounds,
    WindowDecorations, WindowOptions, canvas, div, point, px, size, transparent_black,
};

struct ResizeHitbox {
    hitbox: Hitbox,
    cursor: CursorStyle,
}

#[derive(IntoElement)]
pub struct ContemporaryWindow {
    div: Div,
}

pub fn contemporary_window() -> ContemporaryWindow {
    ContemporaryWindow { div: div() }
}

impl ParentElement for ContemporaryWindow {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.div.extend(elements);
    }
}

impl RenderOnce for ContemporaryWindow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let toast_drawer = window.use_state(cx, ToastDrawer::new);

        let theme = cx.theme();
        let platform_settings = cx.global::<PlatformSettings>();
        let decorations = window.window_decorations();

        let window_margins = (platform_settings.resize_grip_size)(window);

        window.set_client_inset(window_margins);

        div()
            .text_color(theme.foreground)
            .text_size(theme.system_font_size)
            .w_full()
            .h_full()
            .font_family(theme.system_font_family.clone())
            .map(|div| match decorations {
                Decorations::Server => div,
                Decorations::Client { tiling, .. } => div
                    .bg(transparent_black())
                    .child(
                        canvas(
                            move |_, window, _| {
                                let window_bounds = window.window_bounds().get_bounds();
                                [
                                    // Top Left
                                    ResizeHitbox {
                                        hitbox: window.insert_hitbox(
                                            Bounds::new(
                                                point(px(0.0), px(0.0)),
                                                Size::new(window_margins, window_margins),
                                            ),
                                            HitboxBehavior::Normal,
                                        ),
                                        cursor: CursorStyle::ResizeUpLeftDownRight,
                                    },
                                    // Top
                                    ResizeHitbox {
                                        hitbox: window.insert_hitbox(
                                            Bounds::new(
                                                point(window_margins, px(0.)),
                                                Size::new(
                                                    window_bounds.size.width
                                                        - window_margins
                                                        - window_margins,
                                                    window_margins,
                                                ),
                                            ),
                                            HitboxBehavior::Normal,
                                        ),
                                        cursor: CursorStyle::ResizeUpDown,
                                    },
                                    // Top Right
                                    ResizeHitbox {
                                        hitbox: window.insert_hitbox(
                                            Bounds::new(
                                                point(
                                                    window_bounds.size.width - window_margins,
                                                    px(0.),
                                                ),
                                                Size::new(window_margins, window_margins),
                                            ),
                                            HitboxBehavior::Normal,
                                        ),
                                        cursor: CursorStyle::ResizeUpRightDownLeft,
                                    },
                                    // Left
                                    ResizeHitbox {
                                        hitbox: window.insert_hitbox(
                                            Bounds::new(
                                                point(px(0.), window_margins),
                                                Size::new(
                                                    window_margins,
                                                    window_bounds.size.height
                                                        - window_margins
                                                        - window_margins,
                                                ),
                                            ),
                                            HitboxBehavior::Normal,
                                        ),
                                        cursor: CursorStyle::ResizeLeft,
                                    },
                                    // Right
                                    ResizeHitbox {
                                        hitbox: window.insert_hitbox(
                                            Bounds::new(
                                                point(
                                                    window_bounds.size.width - window_margins,
                                                    window_margins,
                                                ),
                                                Size::new(
                                                    window_margins,
                                                    window_bounds.size.height
                                                        - window_margins
                                                        - window_margins,
                                                ),
                                            ),
                                            HitboxBehavior::Normal,
                                        ),
                                        cursor: CursorStyle::ResizeLeft,
                                    },
                                    // Bottom Left
                                    ResizeHitbox {
                                        hitbox: window.insert_hitbox(
                                            Bounds::new(
                                                point(
                                                    px(0.),
                                                    window_bounds.size.height - window_margins,
                                                ),
                                                Size::new(window_margins, window_margins),
                                            ),
                                            HitboxBehavior::Normal,
                                        ),
                                        cursor: CursorStyle::ResizeUpRightDownLeft,
                                    },
                                    // Bottom
                                    ResizeHitbox {
                                        hitbox: window.insert_hitbox(
                                            Bounds::new(
                                                point(
                                                    window_margins,
                                                    window_bounds.size.height - window_margins,
                                                ),
                                                Size::new(
                                                    window_bounds.size.width
                                                        - window_margins
                                                        - window_margins,
                                                    window_margins,
                                                ),
                                            ),
                                            HitboxBehavior::Normal,
                                        ),
                                        cursor: CursorStyle::ResizeUpDown,
                                    },
                                    // Bottom Right
                                    ResizeHitbox {
                                        hitbox: window.insert_hitbox(
                                            Bounds::new(
                                                point(
                                                    window_bounds.size.width - window_margins,
                                                    window_bounds.size.height - window_margins,
                                                ),
                                                Size::new(window_margins, window_margins),
                                            ),
                                            HitboxBehavior::Normal,
                                        ),
                                        cursor: CursorStyle::ResizeUpLeftDownRight,
                                    },
                                ]
                            },
                            move |_bounds, hitboxes, window, _| {
                                for hitbox in hitboxes {
                                    window.set_cursor_style(hitbox.cursor, &hitbox.hitbox);
                                }
                            },
                        )
                        .size_full()
                        .absolute(),
                    )
                    .on_mouse_down(MouseButton::Left, move |e, window, _| {
                        let size = window.window_bounds().get_bounds().size;
                        let pos = e.position;

                        if let Some(edge) = resize_edge(pos, window_margins, size, tiling) {
                            window.start_window_resize(edge)
                        };
                    }),
            })
            .flex()
            .flex_col()
            .child(
                self.div
                    .m(window_margins)
                    .flex()
                    .flex_col()
                    .flex_grow(1.)
                    .bg(theme.background)
                    .rounded(theme.border_radius)
                    .overflow_hidden(),
            )
            .child(toast_drawer)
    }
}

fn resize_edge(
    pos: Point<Pixels>,
    shadow_size: Pixels,
    size: Size<Pixels>,
    tiling: Tiling,
) -> Option<ResizeEdge> {
    let edge = if pos.y < shadow_size * 2 && pos.x < shadow_size * 2 && !tiling.top && !tiling.left
    {
        ResizeEdge::TopLeft
    } else if pos.y < shadow_size * 2
        && pos.x > size.width - shadow_size * 2
        && !tiling.top
        && !tiling.right
    {
        ResizeEdge::TopRight
    } else if pos.y < shadow_size && !tiling.top {
        ResizeEdge::Top
    } else if pos.y > size.height - shadow_size * 2
        && pos.x < shadow_size * 2
        && !tiling.bottom
        && !tiling.left
    {
        ResizeEdge::BottomLeft
    } else if pos.y > size.height - shadow_size * 2
        && pos.x > size.width - shadow_size * 2
        && !tiling.bottom
        && !tiling.right
    {
        ResizeEdge::BottomRight
    } else if pos.y > size.height - shadow_size && !tiling.bottom {
        ResizeEdge::Bottom
    } else if pos.x < shadow_size && !tiling.left {
        ResizeEdge::Left
    } else if pos.x > size.width - shadow_size && !tiling.right {
        ResizeEdge::Right
    } else {
        return None;
    };
    Some(edge)
}

pub fn contemporary_window_options(
    cx: &mut App,
    window_title: impl Into<SharedString>,
) -> WindowOptions {
    let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some(window_title.into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(10.0), px(10.0))),
        }),
        window_decorations: Some(WindowDecorations::Client),
        ..Default::default()
    }
}
