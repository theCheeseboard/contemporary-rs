use cntp_i18n::tr;
use contemporary::application::Details;
use contemporary::components::admonition::AdmonitionSeverity;
use contemporary::components::button::button;
use contemporary::components::checkbox::{CheckState, CheckedChangeEvent, radio_button};
use contemporary::components::constrainer::constrainer;
use contemporary::components::grandstand::grandstand;
use contemporary::components::icon_text::icon_text;
use contemporary::components::layer::layer;
use contemporary::components::subtitle::subtitle;
use contemporary::components::text_field::TextField;
use contemporary::components::toast::Toast;
use contemporary::notification::PostedNotification;
use contemporary::styling::theme::ThemeStorage;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, ClickEvent, Context, Entity, IntoElement, ParentElement, Render, Styled,
    Window, div, px,
};

pub struct Toasts {
    title_field: Entity<TextField>,
    body_field: Entity<TextField>,
    severity: AdmonitionSeverity,
}

impl Toasts {
    pub fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            title_field: cx.new(|cx| {
                let mut text_field = TextField::new("title-field", cx);
                text_field.set_placeholder(tr!("TOAST_TITLE", "Title").to_string().as_str());
                text_field
            }),
            body_field: cx.new(|cx| {
                let mut text_field = TextField::new("body-field", cx);
                text_field.set_placeholder(tr!("TOAST_BODY", "Body").to_string().as_str());
                text_field
            }),
            severity: AdmonitionSeverity::Info,
        })
    }

    fn trigger_toast(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let title = self.title_field.read(cx).text();
        let body = self.body_field.read(cx).text();

        Toast::new()
            .title(title)
            .body(body)
            .severity(self.severity)
            .post(window, cx);
    }
}

impl Render for Toasts {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let details = cx.global::<Details>();

        let _directories = details.standard_dirs().unwrap();

        div()
            .bg(theme.background)
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .child(
                grandstand("toasts-grandstand")
                    .text(tr!("TOASTS_TITLE", "Toasts"))
                    .pt(px(36.)),
            )
            .child(
                constrainer("i18n")
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
                            .child(subtitle(tr!("TOASTS_TITLE")))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .child(tr!(
                                        "TOASTS_DESCRIPTION",
                                        "Notifications throw up an alert in-app to let \
                                        the user know about something that happened in the \
                                        background."
                                    ))
                                    .child(self.title_field.clone())
                                    .child(self.body_field.clone())
                                    .child(
                                        div()
                                            .flex()
                                            .gap(px(4.))
                                            .child(tr!("TOAST_SEVERITY", "Severity"))
                                            .child(div().flex_grow(1.))
                                            .child(
                                                radio_button("severity-info")
                                                    .label(tr!("SEVERITY_INFO", "Info"))
                                                    .when(
                                                        self.severity == AdmonitionSeverity::Info,
                                                        |david| david.checked(),
                                                    )
                                                    .on_checked_changed(cx.listener(
                                                        |this, event: &CheckedChangeEvent, _, cx| {
                                                            if event.check_state == CheckState::On {
                                                                this.severity =
                                                                    AdmonitionSeverity::Info;
                                                                cx.notify();
                                                            }
                                                        },
                                                    )),
                                            )
                                            .child(
                                                radio_button("severity-warn")
                                                    .label(tr!("SEVERITY_WARN", "Warning"))
                                                    .when(
                                                        self.severity == AdmonitionSeverity::Warning,
                                                        |david| david.checked(),
                                                    )
                                                    .on_checked_changed(cx.listener(
                                                        |this, event: &CheckedChangeEvent, _, cx| {
                                                            if event.check_state == CheckState::On {
                                                                this.severity =
                                                                    AdmonitionSeverity::Warning;
                                                                cx.notify();
                                                            }
                                                        },
                                                    )),
                                            )
                                            .child(
                                                radio_button("severity-error")
                                                    .label(tr!("SEVERITY_ERROR", "Error"))
                                                    .when(
                                                        self.severity == AdmonitionSeverity::Error,
                                                        |david| david.checked(),
                                                    )
                                                    .on_checked_changed(cx.listener(
                                                        |this, event: &CheckedChangeEvent, _, cx| {
                                                            if event.check_state == CheckState::On {
                                                                this.severity =
                                                                    AdmonitionSeverity::Error;
                                                                cx.notify();
                                                            }
                                                        },
                                                    )),
                                            ),
                                    )
                                    .child(
                                        button("send-toast-button")
                                            .child(icon_text(
                                                "mail-send",
                                                tr!("TOAST_SEND", "Post Toast"),
                                            ))
                                            .on_click(cx.listener(Self::trigger_toast)),
                                    ),
                            ),
                    ),
            )
    }
}
