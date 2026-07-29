use crate::patterns::dialog_boxes::DialogBoxes;
use crate::patterns::directories::Directories;
use crate::patterns::flyouts::Flyouts;
use crate::patterns::i18n::I18n;
use crate::patterns::jobs::Jobs;
use crate::patterns::notifications::Notifications;
use crate::patterns::popovers::Popovers;
use crate::patterns::toasts::Toasts;
use cntp_i18n::tr;
use contemporary::components::grandstand::grandstand;
use contemporary::components::layer::layer;
use contemporary::components::pager::lift_animation::LiftAnimation;
use contemporary::components::pager::pager;
use contemporary::components::pager::pager_animation::PagerAnimationDirection;
use contemporary::components::scrollbar::SelfScrollable;
use contemporary::styling::theme::ThemeStorage;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, px, uniform_list,
};

pub struct PatternsRoot {
    dialog_boxes: Entity<DialogBoxes>,
    i18n: Entity<I18n>,
    popovers: Entity<Popovers>,
    jobs: Entity<Jobs>,
    directories: Entity<Directories>,
    flyouts: Entity<Flyouts>,
    notifications: Entity<Notifications>,
    toasts: Entity<Toasts>,

    current_page: usize,
}

impl PatternsRoot {
    pub fn new(cx: &mut App) -> Entity<PatternsRoot> {
        cx.new(|cx| PatternsRoot {
            dialog_boxes: DialogBoxes::new(cx),
            i18n: I18n::new(cx),
            popovers: Popovers::new(cx),
            jobs: Jobs::new(cx),
            directories: Directories::new(cx),
            flyouts: Flyouts::new(cx),
            notifications: Notifications::new(cx),
            toasts: Toasts::new(cx),
            current_page: 0,
        })
    }
}

impl Render for PatternsRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("patterns")
            .flex()
            .w_full()
            .h_full()
            .gap(px(2.))
            .child(
                layer()
                    .w(px(300.))
                    .flex()
                    .flex_col()
                    .child(
                        grandstand("sidebar-grandstand")
                            .text(tr!("PATTERNS_TITLE", "Patterns"))
                            .pt(px(36.)),
                    )
                    .child(
                        div().flex_grow(1.).p(px(2.)).child(
                            uniform_list(
                                "sidebar-items",
                                8,
                                cx.processor(|this, range, _, cx| {
                                    let theme = cx.theme();
                                    let mut items = Vec::new();
                                    for ix in range {
                                        let item = ix + 1;

                                        items.push(
                                            div()
                                                .id(ix)
                                                .p(px(2.))
                                                .rounded(theme.border_radius)
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.current_page = ix;
                                                    cx.notify()
                                                }))
                                                .child(match ix {
                                                    0 => tr!("DIALOG_BOXES_TITLE"),
                                                    1 => tr!("I18N_TITLE"),
                                                    2 => tr!("POPOVERS_TITLE"),
                                                    3 => tr!("FLYOUTS_TITLE"),
                                                    4 => tr!("JOBS_TITLE"),
                                                    5 => tr!("DIRECTORIES_TITLE"),
                                                    6 => tr!("NOTIFICATIONS_TITLE"),
                                                    7 => tr!("TOASTS_TITLE"),
                                                    _ => format!("Item {item}").into(),
                                                })
                                                .when(this.current_page == ix, |div| {
                                                    div.bg(theme.button_background)
                                                }),
                                        );
                                    }
                                    items
                                }),
                            )
                            .self_scrollable(window, cx)
                            .h_full()
                            .w_full(),
                        ),
                    ),
            )
            .child(
                pager("main-area", self.current_page)
                    .flex_grow(1.)
                    .animation(LiftAnimation::new())
                    .animation_direction(PagerAnimationDirection::Forward)
                    .page(self.dialog_boxes.clone().into_any_element())
                    .page(self.i18n.clone().into_any_element())
                    .page(self.popovers.clone().into_any_element())
                    .page(self.flyouts.clone().into_any_element())
                    .page(self.jobs.clone().into_any_element())
                    .page(self.directories.clone().into_any_element())
                    .page(self.notifications.clone().into_any_element())
                    .page(self.toasts.clone().into_any_element()),
            )
    }
}
