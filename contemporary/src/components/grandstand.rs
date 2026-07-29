use std::rc::Rc;

use crate::components::button::button;
use crate::components::icon::icon;
use crate::components::layer::layer;
use crate::components::tooltip::simple_tooltip;
use crate::styling::theme::ThemeStorage;
use cntp_i18n::tr;
use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, Stateful, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder, px,
};

type ClickHandler = Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>;

#[derive(IntoElement)]
pub struct Grandstand {
    on_back_click: ClickHandler,
    text: SharedString,
    div: Stateful<Div>,
    button_div: Div,
}

pub fn grandstand(id: impl Into<ElementId>) -> Grandstand {
    Grandstand {
        on_back_click: None,
        text: "".into(),
        div: div().id(id),
        button_div: div().flex(),
    }
}

impl Grandstand {
    pub fn on_back_click(
        mut self,
        fun: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_back_click = Some(Rc::new(fun));
        self
    }

    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = text.into();
        self
    }
}

impl Styled for Grandstand {
    fn style(&mut self) -> &mut StyleRefinement {
        self.div.style()
    }
}

impl ParentElement for Grandstand {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.button_div.extend(elements);
    }
}

impl RenderOnce for Grandstand {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        layer().child(
            self.div.flex().flex_row().child(
                div()
                    .flex()
                    .flex_grow(1.)
                    .justify_center()
                    .gap(px(4.))
                    .when_some(self.on_back_click, move |div, on_click| {
                        div.child(
                            button("back-button")
                                .flat()
                                .child(icon("go-previous"))
                                .on_click(move |ev, window, cx| {
                                    on_click(ev, window, cx);
                                })
                                .tooltip(simple_tooltip(tr!("GRANDSTAND_BACK", "Back"))),
                        )
                    })
                    .child(
                        div()
                            .flex()
                            .flex_grow(1.)
                            .items_center()
                            .text_size(theme.heading_font_size)
                            .child(self.text)
                            .pt(px(2.))
                            .m(px(4.)),
                    )
                    .child(self.button_div),
            ),
        )
    }
}
