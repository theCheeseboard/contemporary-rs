use crate::components::scrollbar::Scrollable;
use crate::styling::theme::{ThemeStorage, VariableColor};
use crate::tracing::application_log::ApplicationLog;
use crate::window::contemporary_window;
use cntp_i18n::{tr, trf};
use gpui::{
    App, AppContext, Context, IntoElement, ParentElement, Render, Styled, TitlebarOptions,
    UniformListScrollHandle, Window, WindowBounds, WindowOptions, bounds, div, point, px, rgb,
    size, transparent_black, uniform_list,
};
use tracing::Level;

pub fn open_debug_log(cx: &mut App) {
    let _ = cx.open_window(
        WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(tr!("DEBUG_LOG", "Debug Log").into()),
                ..TitlebarOptions::default()
            }),
            window_bounds: Some(WindowBounds::Windowed(bounds(
                point(px(100.), px(100.)),
                size(px(640.), px(480.)),
            ))),
            ..WindowOptions::default()
        },
        |window, cx| cx.new(|cx| DebugLog::new(window, cx)),
    );
}

pub struct DebugLog {
    scroll_handle: UniformListScrollHandle,
}

impl DebugLog {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> DebugLog {
        let weak_this = cx.weak_entity();
        window
            .observe_global::<ApplicationLog>(cx, {
                move |window, cx| {
                    let _ = weak_this.update(cx, |this, cx| {
                        if this
                            .scroll_handle
                            .is_scrolled_to_end()
                            .is_none_or(|scrolled_to_end| scrolled_to_end)
                        {
                            window.on_next_frame({
                                let weak_this = weak_this.clone();
                                move |_, cx| {
                                    let _ = weak_this.update(cx, |this, _| {
                                        this.scroll_handle.scroll_to_bottom()
                                    });
                                }
                            })
                        }
                    });
                    window.refresh();
                }
            })
            .detach();

        DebugLog {
            scroll_handle: UniformListScrollHandle::new(),
        }
    }
}

impl Render for DebugLog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let log = cx.global::<ApplicationLog>();

        contemporary_window().child(
            div().size_full().flex().child(
                uniform_list("list", log.entries().len(), |range, window, cx| {
                    let log = cx.global::<ApplicationLog>();
                    let theme = cx.theme();
                    range
                        .map(|index| {
                            if let Some(entry) = log.entries().get(index) {
                                div()
                                    .p(px(2.))
                                    .overflow_hidden()
                                    .font_family(theme.monospaced_font_family.clone())
                                    .text_size(theme.system_font_size * 0.75)
                                    .flex()
                                    .items_center()
                                    .gap(px(4.))
                                    .child(div().w(px(4.)).rounded(theme.border_radius).bg(
                                        match entry.level {
                                            Level::ERROR => rgb(0xFF0000),
                                            Level::WARN => rgb(0xFFFF00),
                                            Level::INFO => rgb(0x00FF00),
                                            Level::DEBUG => transparent_black().into(),
                                            Level::TRACE => transparent_black().into(),
                                            _ => transparent_black().into(),
                                        },
                                    ))
                                    .child(
                                        div()
                                            .w(px(60.))
                                            .text_color(theme.foreground.disabled())
                                            .text_size(theme.system_font_size * 0.6)
                                            .child(trf!(
                                                date("T", length = "long"),
                                                entry.timestamp
                                            )),
                                    )
                                    .child(
                                        div()
                                            .w(px(200.))
                                            .text_color(theme.foreground.disabled())
                                            .text_size(theme.system_font_size * 0.6)
                                            .child(entry.target.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_ellipsis()
                                            .overflow_hidden()
                                            .flex_grow()
                                            .child(entry.message.clone()),
                                    )
                            } else {
                                div()
                            }
                        })
                        .collect()
                })
                .track_scroll(&self.scroll_handle)
                .scrollable(self.scroll_handle.clone())
                .h_full()
                .flex_grow(),
            ),
        )
    }
}
