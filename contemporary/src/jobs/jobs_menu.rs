use crate::components::grandstand::grandstand;
use crate::components::scrim::scrim;
use crate::jobs::job_manager::JobManager;
use crate::styling::theme::ThemeStorage;
use cntp_i18n::tr;
use gpui::prelude::FluentBuilder;
use gpui::{
    App, BorrowAppContext, InteractiveElement, IntoElement, ListAlignment, ListState,
    ParentElement, RenderOnce, Styled, Window, div, list, px,
};

#[derive(IntoElement)]
pub struct JobsMenu {}

pub fn jobs_menu() -> JobsMenu {
    JobsMenu {}
}

impl RenderOnce for JobsMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let job_manager = cx.global::<JobManager>();

        let list_state = ListState::new(job_manager.job_len(cx), ListAlignment::Top, px(0.));

        scrim("jobs-menu")
            .visible(job_manager.is_job_menu_open)
            .on_click(move |_, _window, cx| {
                cx.update_global::<JobManager, ()>(|manager, cx| {
                    manager.set_job_menu_open(false, cx);
                });
            })
            .when(job_manager.is_job_menu_open, |david| {
                david.child(
                    div()
                        .bg(theme.background)
                        .border_color(theme.border_color)
                        .border(px(1.))
                        .rounded(theme.border_radius)
                        .occlude()
                        .flex()
                        .flex_col()
                        .w(px(300.))
                        .h(px(500.))
                        .max_h(window.viewport_size().height)
                        .left(window.viewport_size().width - px(300.))
                        .child(grandstand("jobs-grandstand").text(tr!("JOBS_MENU_TITLE", "Jobs")))
                        .child(
                            list(list_state, |index, _, cx| {
                                let job_manager = cx.global::<JobManager>();
                                let job = job_manager.job(index, cx).unwrap();
                                job.read(cx).borrow().element()
                            })
                            .p(px(10.))
                            .flex_grow(1.)
                            .w_full(),
                        ),
                )
            })
    }
}
