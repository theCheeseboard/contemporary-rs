use cntp_i18n::{Quote, tr};
use contemporary::application::Details;
use contemporary::components::button::button;
use contemporary::components::constrainer::constrainer;
use contemporary::components::grandstand::grandstand;
use contemporary::components::icon::icon;
use contemporary::components::icon_text::icon_text;
use contemporary::components::layer::layer;
use contemporary::components::subtitle::subtitle;
use contemporary::components::switch::{SwitchChangeEvent, switch};
use contemporary::components::text_field::TextField;
use contemporary::notification::{Notification, NotificationSound, PostedNotification};
use contemporary::styling::theme::ThemeStorage;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, AppContext, ClickEvent, Context, Entity, InteractiveElement, IntoElement,
    ListAlignment, ListSizingBehavior, ListState, ParentElement, Render, Styled, Window, div, list,
    px,
};
use std::rc::Rc;

pub struct Notifications {
    summary_field: Entity<TextField>,
    body_field: Entity<TextField>,

    posted_notifications: Vec<Rc<Box<dyn PostedNotification>>>,
    metadata: Vec<Entity<NotificationMetadata>>,
    list_state: ListState,
    pending_actions: Vec<Entity<TextField>>,

    has_default_action: bool,
    has_reply_action: bool,
    is_muted: bool,
}

#[derive(Default)]
struct NotificationMetadata {
    dismissed: bool,
    triggered_action: Option<NotificationTriggeredAction>,
}

#[derive(Clone)]
enum NotificationTriggeredAction {
    ActionName(String),
    DefaultAction,
    Reply(String),
}

impl Notifications {
    pub fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            summary_field: cx.new(|cx| {
                let mut text_field = TextField::new("summary-field", cx);
                text_field
                    .set_placeholder(tr!("NOTIFICATIONS_SUMMARY", "Summary").to_string().as_str());
                text_field
            }),
            body_field: cx.new(|cx| {
                let mut text_field = TextField::new("body-field", cx);
                text_field.set_placeholder(tr!("NOTIFICATIONS_BODY", "Body").to_string().as_str());
                text_field
            }),

            posted_notifications: Vec::new(),
            metadata: Vec::new(),
            list_state: ListState::new(0, ListAlignment::Top, px(200.)),
            pending_actions: Vec::new(),
            has_default_action: false,
            has_reply_action: false,
            is_muted: false,
        })
    }

    fn trigger_notification(
        &mut self,
        _: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let meta = cx.new(|_| NotificationMetadata::default());
        let meta_clone = meta.clone();
        let meta_weak = meta.downgrade();

        let summary = self.summary_field.read(cx).text();
        let body = self.body_field.read(cx).text();

        let mut notification = Notification::new()
            .summary(summary)
            .body(body)
            .sound(if self.is_muted {
                NotificationSound::Silent
            } else {
                NotificationSound::Default
            })
            .on_dismiss(move |_, cx| {
                meta_clone.update(cx, |meta, cx| {
                    meta.dismissed = true;
                    cx.notify()
                })
            });

        for action in self.pending_actions.iter() {
            let meta_weak = meta_weak.clone();
            let action_text = action.read(cx).text().to_string();
            let action_text_2 = action_text.clone();
            notification = notification.on_action(&action_text_2, move |_, cx| {
                let action_text = action_text.clone();
                let _ = meta_weak.update(cx, |meta, cx| {
                    meta.triggered_action =
                        Some(NotificationTriggeredAction::ActionName(action_text));
                    cx.notify();
                });
            })
        }

        if self.has_default_action {
            let meta_weak = meta_weak.clone();
            notification = notification.on_default_action(move |_, cx| {
                let _ = meta_weak.update(cx, |meta, cx| {
                    meta.triggered_action = Some(NotificationTriggeredAction::DefaultAction);
                    cx.notify();
                });
            })
        }

        if self.has_reply_action {
            let meta_weak = meta_weak.clone();
            notification = notification.on_reply_action(move |event, cx| {
                let _ = meta_weak.update(cx, |meta, cx| {
                    meta.triggered_action =
                        Some(NotificationTriggeredAction::Reply(event.text.clone()));
                    cx.notify();
                });
            })
        }

        let posted = Rc::new(notification.post(cx));

        self.posted_notifications.push(posted.clone());
        self.metadata.push(meta);
        self.list_state.reset(self.posted_notifications.len());
        cx.notify();
    }

    fn render_posted_notification(
        &mut self,
        i: usize,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let notification: &Rc<Box<dyn PostedNotification>> = &self.posted_notifications[i];
        let metadata: &Entity<NotificationMetadata> = &self.metadata[i];
        let notification = notification.clone();
        let metadata = metadata.read(cx);

        div()
            .id(i)
            .py(px(2.))
            .child(
                layer()
                    .flex()
                    .w_full()
                    .p(px(4.))
                    .gap(px(4.))
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_grow(1.)
                            .child(subtitle(
                                notification.summary().unwrap_or_default().to_string(),
                            ))
                            .child(notification.body().unwrap_or_default().to_string()),
                    )
                    .when_some(
                        metadata.triggered_action.clone(),
                        |david, action| match action {
                            NotificationTriggeredAction::ActionName(action_name) => {
                                david.child(tr!(
                                    "NOTIFICATION_ACTION_TRIGGERED",
                                    "Triggered with {{action}}",
                                    action:Quote = action_name
                                ))
                            }
                            NotificationTriggeredAction::DefaultAction => david.child(tr!(
                                "NOTIFICATION_DEFAULT_ACTION_TRIGGERED",
                                "Triggered default action",
                            )),
                            NotificationTriggeredAction::Reply(reply) => david.child(tr!(
                                "NOTIFICATION_REPLY_ACTION_TRIGGERED",
                                "Replied with {{reply}}",
                                reply:Quote = reply
                            )),
                        },
                    )
                    .child(
                        button("dismiss-button")
                            .destructive()
                            .child(icon("edit-delete"))
                            .when(metadata.dismissed, |button| button.disabled())
                            .on_click(move |_, _, cx| {
                                notification.dismiss(cx);
                            }),
                    ),
            )
            .into_any_element()
    }
}

impl Render for Notifications {
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
                grandstand("notifications-grandstand")
                    .text(tr!("NOTIFICATIONS_TITLE", "Notifications"))
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
                            .child(subtitle(tr!("NOTIFICATIONS_TITLE")))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .child(tr!(
                                        "NOTIFICATIONS_DESCRIPTION",
                                        "Notifications throw up an alert on the desktop to let \
                                        the user know about something that happened in the \
                                        background."
                                    ))
                                    .child(self.summary_field.clone())
                                    .child(self.body_field.clone())
                                    .child(self.pending_actions.iter().enumerate().fold(
                                        div().flex().flex_col().gap(px(8.)),
                                        |david, (i, text_field)| {
                                            david.child(
                                                div()
                                                    .id(i)
                                                    .flex()
                                                    .gap(px(4.))
                                                    .child(text_field.clone())
                                                    .child(
                                                        button("delete-button")
                                                            .destructive()
                                                            .child(icon("list-remove"))
                                                            .on_click(cx.listener(
                                                                move |this, _, _, cx| {
                                                                    this.pending_actions.remove(i);
                                                                    cx.notify();
                                                                },
                                                            )),
                                                    ),
                                            )
                                        },
                                    ))
                                    .child(
                                        button("add-action-button")
                                            .child(icon_text(
                                                "list-add",
                                                tr!("NOTIFICATION_ADD_ACTION", "Add Action"),
                                            ))
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.pending_actions.push(
                                                    cx.new(|cx| TextField::new("action", cx)),
                                                );
                                                cx.notify()
                                            })),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .child(tr!(
                                                "NOTIFICATION_DEFAULT_ACTION_PROMPT",
                                                "Enable Default Action"
                                            ))
                                            .child(div().flex_grow(1.))
                                            .child(
                                                switch("default-action-switch")
                                                    .when(self.has_default_action, |david| {
                                                        david.checked()
                                                    })
                                                    .on_change(cx.listener(
                                                        |this, event: &SwitchChangeEvent, _, cx| {
                                                            this.has_default_action = event.checked;
                                                            cx.notify();
                                                        },
                                                    )),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .child(tr!(
                                                "NOTIFICATION_REPLY_ACTION_PROMPT",
                                                "Enable Reply Action"
                                            ))
                                            .child(div().flex_grow(1.))
                                            .child(
                                                switch("reply-action-switch")
                                                    .when(self.has_reply_action, |david| {
                                                        david.checked()
                                                    })
                                                    .on_change(cx.listener(
                                                        |this, event: &SwitchChangeEvent, _, cx| {
                                                            this.has_reply_action = event.checked;
                                                            cx.notify();
                                                        },
                                                    )),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .child(tr!(
                                                "NOTIFICATION_MUTED_PROMPT",
                                                "Notify Silently"
                                            ))
                                            .child(div().flex_grow(1.))
                                            .child(
                                                switch("muted-action-switch")
                                                    .when(self.is_muted, |david| david.checked())
                                                    .on_change(cx.listener(
                                                        |this, event: &SwitchChangeEvent, _, cx| {
                                                            this.is_muted = event.checked;
                                                            cx.notify();
                                                        },
                                                    )),
                                            ),
                                    )
                                    .child(
                                        button("send-notification-button")
                                            .child(icon_text(
                                                "mail-send",
                                                tr!("NOTIFICATION_SEND", "Post Notification"),
                                            ))
                                            .on_click(cx.listener(Self::trigger_notification)),
                                    ),
                            ),
                    )
                    .child(
                        layer()
                            .flex()
                            .flex_col()
                            .p(px(8.))
                            .w_full()
                            .child(subtitle(tr!(
                                "POSTED_NOTIFICATIONS_TITLE",
                                "Posted Notifications"
                            )))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.))
                                    .child(tr!(
                                        "POSTED_NOTIFICATIONS_DESCRIPTION",
                                        "Notifications that you have posted will appear below."
                                    ))
                                    .child(
                                        list(
                                            self.list_state.clone(),
                                            cx.processor(Self::render_posted_notification),
                                        )
                                        .with_sizing_behavior(ListSizingBehavior::Infer),
                                    ),
                            ),
                    ),
            )
    }
}
