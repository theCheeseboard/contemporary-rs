use crate::components::scrim::{Scrim, scrim};
use crate::platform_support::platform_settings::PlatformSettings;
use crate::styling::theme::ThemeStorage;
use crate::transition::float_transition_element::TransitionExt;
use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Styled, Window, div, ease_out_quint, px,
};
use std::cmp::PartialEq;

#[derive(PartialEq, Copy, Clone)]
enum Anchor {
    Top,
    Leading,
    Trailing,
    Bottom,
}

enum Size {
    Pixels(f32),
    Neg(f32),
}

#[derive(IntoElement)]
pub struct Popover {
    scrim: Scrim,
    content: AnyElement,
    size: Size,
    anchor: Anchor,
    visible: bool,
}

pub fn popover(id: impl Into<ElementId>) -> Popover {
    Popover {
        scrim: scrim(id),
        content: div().into_any_element(),
        size: Size::Pixels(300.),
        anchor: Anchor::Bottom,
        visible: false,
    }
}

impl Popover {
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = content.into_any_element();
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = Size::Pixels(size);
        self
    }

    pub fn size_neg(mut self, size: f32) -> Self {
        self.size = Size::Neg(size);
        self
    }

    pub fn anchor_bottom(mut self) -> Self {
        self.anchor = Anchor::Bottom;
        self
    }

    pub fn anchor_top(mut self) -> Self {
        self.anchor = Anchor::Top;
        self
    }

    pub fn anchor_leading(mut self) -> Self {
        self.anchor = Anchor::Leading;
        self
    }

    pub fn anchor_trailing(mut self) -> Self {
        self.anchor = Anchor::Trailing;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

impl RenderOnce for Popover {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let platform_settings = cx.global::<PlatformSettings>();

        self.scrim.visible(self.visible).child(
            div()
                .flex()
                .w_full()
                .h_full()
                .when(self.anchor == Anchor::Top, |david| {
                    david.flex_col_reverse().child(div().flex_grow(1.))
                })
                .when(self.anchor == Anchor::Bottom, |david| {
                    david.flex_col().child(div().flex_grow(1.))
                })
                .when(self.anchor == Anchor::Leading, |david| {
                    david.flex_row_reverse().child(div().flex_grow(1.))
                })
                .when(self.anchor == Anchor::Trailing, |david| {
                    david.flex_row().child(div().flex_grow(1.))
                })
                .child(
                    div()
                        .bg(theme.background)
                        .rounded(theme.border_radius)
                        .flex()
                        .flex_col()
                        .child(self.content)
                        .when(self.visible, |div| div.occlude())
                        .with_transition(
                            "side-transition",
                            if self.visible {
                                match self.size {
                                    Size::Pixels(length) => length,
                                    Size::Neg(pixels) => {
                                        if self.anchor == Anchor::Bottom
                                            || self.anchor == Anchor::Top
                                        {
                                            f32::from(window.viewport_size().height) - pixels
                                        } else {
                                            f32::from(window.viewport_size().width) - pixels
                                        }
                                    }
                                }
                            } else {
                                0.
                            },
                            Animation::new(platform_settings.animation_duration)
                                .with_easing(ease_out_quint()),
                            move |david, value| {
                                david
                                    .when(
                                        self.anchor == Anchor::Bottom || self.anchor == Anchor::Top,
                                        |div| div.h(px(value)),
                                    )
                                    .when(
                                        self.anchor == Anchor::Leading
                                            || self.anchor == Anchor::Trailing,
                                        |div| div.w(px(value)),
                                    )
                                    .when(value == 0., |div| div.invisible())
                            },
                        ),
                ),
        )
    }
}
