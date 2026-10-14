use cntp_i18n::tr;
use contemporary::application::Details;
use contemporary::components::button::button;
use contemporary::components::checkbox::{CheckState, CheckedChangeEvent, checkbox};
use contemporary::components::constrainer::constrainer;
use contemporary::components::grandstand::grandstand;
use contemporary::components::icon::icon;
use contemporary::components::icon_text::icon_text;
use contemporary::components::layer::layer;
use contemporary::components::subtitle::subtitle;
use contemporary::components::switch::{SwitchChangeEvent, switch};
use contemporary::components::text_field::TextField;
use contemporary::inhibit::{InhibitType, Inhibition};
use contemporary::styling::theme::ThemeStorage;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, ClickEvent, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, Styled, Window, div, px,
};

pub struct Inhibitions {
    reason_field: Entity<TextField>,
    inhibit_idle_sleep: bool,
    inhibit_display_sleep: bool,

    inhibition: Option<Inhibition>,
}

impl Inhibitions {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            reason_field: cx.new(|cx| {
                let mut text_field = TextField::new("reason-field", cx);
                text_field
                    .set_placeholder(tr!("INHIBITIONS_REASON", "Reason").to_string().as_str());
                text_field
            }),
            inhibit_idle_sleep: true,
            inhibit_display_sleep: true,
            inhibition: None,
        }
    }

    fn inhibit(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let mut inhibitions = Vec::new();
        if self.inhibit_idle_sleep {
            inhibitions.push(InhibitType::IdleSleep);
        }
        if self.inhibit_display_sleep {
            inhibitions.push(InhibitType::DisplaySleep);
        }

        self.inhibition
            .replace(Inhibition::new(self.reason_field.read(cx).text(), &inhibitions).unwrap());
        cx.notify()
    }

    fn clear_inhibit(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.inhibition.take();
        cx.notify()
    }
}

impl Render for Inhibitions {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .bg(theme.background)
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .child(
                grandstand("inhibitions-grandstand")
                    .text(tr!("INHIBITIONS_TITLE", "Inhibitions"))
                    .pt(px(36.)),
            )
            .child(
                constrainer("inhibitions")
                    .flex()
                    .flex_col()
                    .w_full()
                    .p(px(8.))
                    .gap(px(8.))
                    .child(
                        layer()
                            .flex()
                            .flex_col()
                            .p(px(8.))
                            .w_full()
                            .child(subtitle(tr!("INHIBITIONS_TITLE")))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .child(div().flex().flex_col().gap(px(8.)).child(tr!(
                                        "INHIBITIONS_DESCRIPTION",
                                        "Inhibitions allow you to request that power-related \
                                        actions are deferred. Any inhibitions not supported \
                                        by the current platform are ignored."
                                    )))
                                    .child(self.reason_field.clone())
                                    .child(
                                        checkbox("idle-sleep")
                                            .label(tr!("INHIBITIONS_IDLE_SLEEP", "Idle Sleep"))
                                            .when(self.inhibit_idle_sleep, |david| david.checked())
                                            .on_checked_changed(cx.listener(
                                                |this, event: &CheckedChangeEvent, _, cx| {
                                                    this.inhibit_idle_sleep =
                                                        event.check_state == CheckState::On;
                                                    cx.notify()
                                                },
                                            )),
                                    )
                                    .child(
                                        checkbox("display-sleep")
                                            .label(tr!(
                                                "INHIBITIONS_DISPLAY_SLEEP",
                                                "Display Sleep"
                                            ))
                                            .when(self.inhibit_display_sleep, |david| {
                                                david.checked()
                                            })
                                            .on_checked_changed(cx.listener(
                                                |this, event: &CheckedChangeEvent, _, cx| {
                                                    this.inhibit_display_sleep =
                                                        event.check_state == CheckState::On;
                                                    cx.notify()
                                                },
                                            )),
                                    )
                                    .child(
                                        button("inhibit-button")
                                            .child(icon_text(
                                                "media-playback-pause",
                                                tr!("INHIBITION_SET", "Inhibit"),
                                            ))
                                            .on_click(cx.listener(Self::inhibit)),
                                    ),
                            ),
                    )
                    .when(self.inhibition.is_some(), |david| {
                        david.child(
                            layer()
                                .flex()
                                .flex_col()
                                .p(px(8.))
                                .w_full()
                                .child(subtitle(tr!("INHIBITION_ACTIVE", "Inhibition Active")))
                                .child(div().flex().flex_col().gap(px(8.)).child(tr!(
                                    "INHIBITION_ACTIVE_DESCRIPTION",
                                    "You have set an inhibition. The device should not perform the \
                                    specified power related events."
                                )))
                                .child(
                                    button("inhibit-stop-button")
                                        .child(icon_text(
                                            "media-playback-start",
                                            tr!("INHIBITION_STOP", "Clear Inhibition"),
                                        ))
                                        .on_click(cx.listener(Self::clear_inhibit)),
                                ),
                        )
                    }),
            )
    }
}
