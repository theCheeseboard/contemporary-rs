use cntp_i18n::tr;
use contemporary::components::button::button;
use contemporary::components::constrainer::constrainer;
use contemporary::components::grandstand::grandstand;
use contemporary::components::layer::layer;
use contemporary::components::scroll_area::{scroll_area, scroll_area_cx};
use contemporary::components::scrollbar::{Scrollable, SelfScrollable};
use contemporary::components::subtitle::subtitle;
use contemporary::jobs::job::{Job, JobStatus};
use contemporary::jobs::job_manager::JobManager;
use contemporary::jobs::standard_job::StandardJob;
use contemporary::styling::theme::ThemeStorage;
use gpui::{
    App, AppContext, AsyncApp, BorrowAppContext, Context, Entity, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub struct Jobs {}

impl Jobs {
    pub fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Jobs {})
    }

    fn render_jobs_section(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        layer()
            .flex()
            .flex_col()
            .p(px(8.))
            .w_full()
            .child(subtitle(tr!("JOBS_TITLE")))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(tr!(
                        "JOBS_DESCRIPTION",
                        "Jobs represent long running processes \
                        that are currently running in the \
                        background. When a job is added, it shows \
                        in the Jobs pane in the top right corner \
                        of the window."
                    ))
                    .child(
                        button("job-normal")
                            .child(tr!("JOB_NORMAL_START", "Start Normal Job"))
                            .on_click(cx.listener(|_this, _, _, cx| {
                                let job = StandardJob::new(
                                    tr!("STANDARD_JOB_TITLE", "Standard Job"),
                                    tr!(
                                        "STANDARD_JOB_IN_PROGRESS_DESCRIPTION",
                                        "A description of the current task can go here."
                                    ),
                                );

                                cx.spawn({
                                    let mut job = job.clone();
                                    async move |_, cx: &mut AsyncApp| {
                                        let instant = Instant::now();
                                        while instant.elapsed().as_secs_f32() < 10. {
                                            job.update_job_progress(
                                                (instant.elapsed().as_secs_f32() * 1000.) as u64,
                                                10000,
                                            );
                                            cx.background_executor()
                                                .timer(Duration::from_millis(10))
                                                .await;
                                        }
                                        job.update_job_status(
                                            tr!(
                                                "STANDARD_JOB_COMPLETE_DESCRIPTION",
                                                "This job is now complete."
                                            ),
                                            JobStatus::Completed,
                                        );
                                    }
                                })
                                .detach();
                                cx.update_global::<JobManager, ()>(|job_manager, cx| {
                                    job_manager.track_job(job.make_job_entity(cx), cx);
                                });
                            })),
                    )
                    .child(
                        button("job-indeterminate")
                            .child(tr!(
                                "JOB_INDETERMINATE_START",
                                "Start Indeterminate Processing Job"
                            ))
                            .on_click(cx.listener(|_this, _, _, cx| {
                                let job = StandardJob::new_indeterminate(
                                    tr!("INDETERMINATE_JOB_TITLE", "Indeterminate Job"),
                                    tr!(
                                        "INDETERMINATE_JOB_IN_PROGRESS_DESCRIPTION",
                                        "This job has an indeterminate progress bar."
                                    ),
                                );

                                cx.spawn({
                                    let mut job = job.clone();
                                    async move |_, cx: &mut AsyncApp| {
                                        cx.background_executor()
                                            .timer(Duration::from_secs(10))
                                            .await;
                                        job.update_job_status(
                                            tr!("STANDARD_JOB_COMPLETE_DESCRIPTION"),
                                            JobStatus::Completed,
                                        );
                                    }
                                })
                                .detach();
                                cx.update_global::<JobManager, ()>(|job_manager, cx| {
                                    job_manager.track_job(job.make_job_entity(cx), cx);
                                });
                            })),
                    )
                    .child(
                        button("job-transient")
                            .child(tr!("JOB_TRANSIENT_START", "Start Transient Job"))
                            .on_click(cx.listener(|_this, _, _, cx| {
                                let job = StandardJob::new_transient(
                                    tr!("TRANSIENT_JOB_TITLE", "Transient Job"),
                                    tr!(
                                        "TRANSIENT_JOB_IN_PROGRESS_DESCRIPTION",
                                        "This job is transient and so will disappear \
                                        automatically once it is complete."
                                    ),
                                );
                                cx.spawn({
                                    let mut job = job.clone();
                                    async move |_, cx: &mut AsyncApp| {
                                        let instant = Instant::now();
                                        while instant.elapsed().as_secs_f32() < 10. {
                                            job.update_job_progress(
                                                (instant.elapsed().as_secs_f32() * 1000.) as u64,
                                                10000,
                                            );
                                            cx.background_executor()
                                                .timer(Duration::from_millis(10))
                                                .await;
                                        }
                                        job.update_job_status(
                                            tr!("STANDARD_JOB_COMPLETE_DESCRIPTION"),
                                            JobStatus::Completed,
                                        );
                                    }
                                })
                                .detach();
                                cx.update_global::<JobManager, ()>(|job_manager, cx| {
                                    job_manager.track_job(job.make_job_entity(cx), cx);
                                });
                            })),
                    )
                    .child(
                        button("job-failing")
                            .child(tr!("JOB_FAILING_START", "Start Failing Job"))
                            .on_click(cx.listener(|_this, _, _, cx| {
                                let job = StandardJob::new(
                                    tr!("FAILING_JOB_TITLE", "Failing Job"),
                                    tr!(
                                        "FAILING_JOB_IN_PROGRESS_DESCRIPTION",
                                        "This job will fail halfway through processing."
                                    ),
                                );
                                cx.spawn({
                                    let mut job = job.clone();
                                    async move |_, cx: &mut AsyncApp| {
                                        let instant = Instant::now();
                                        while instant.elapsed().as_secs_f32() < 5. {
                                            job.update_job_progress(
                                                (instant.elapsed().as_secs_f32() * 1000.) as u64,
                                                10000,
                                            );
                                            cx.background_executor()
                                                .timer(Duration::from_millis(10))
                                                .await;
                                        }
                                        job.update_job_status(
                                            tr!(
                                                "FAILING_JOB_COMPLETE_DESCRIPTION",
                                                "This job has failed."
                                            ),
                                            JobStatus::Failed,
                                        );
                                    }
                                })
                                .detach();
                                cx.update_global::<JobManager, ()>(|job_manager, cx| {
                                    job_manager.track_job(job.make_job_entity(cx), cx);
                                });
                            })),
                    ),
            )
    }

    fn render_delayed_tracking_section(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        layer()
            .flex()
            .flex_col()
            .p(px(8.))
            .w_full()
            .child(subtitle(tr!(
                "JOB_DELAYED_TRACKING_TITLE",
                "Delayed Tracking"
            )))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(tr!(
                        "JOB_DELAYED_TRACKING_DESCRIPTION",
                        "For jobs that have the potential to finish quickly, delayed tracking \
                        ensures the job isn't registered in the job manager if it is \
                        finished quickly."
                    ))
                    .child(
                        button("delayed-tracking-quick")
                            .child(tr!(
                                "JOB_DELAYED_TRACKING_QUICK",
                                "Start Quick Job with Delayed Tracking"
                            ))
                            .on_click(cx.listener(|_this, _, _, cx| {
                                let job = StandardJob::new(
                                    tr!(
                                        "JOB_DELAYED_TRACKING_QUICK_TITLE",
                                        "Quick Delayed Tracking"
                                    ),
                                    tr!(
                                        "JOB_DELAYED_TRACKING_QUICK_IN_PROGRESS_DESCRIPTION",
                                        "This job won't show up in the job menu because it \
                                        finishes too quickly."
                                    ),
                                );

                                cx.spawn({
                                    let mut job = job.clone();
                                    async move |_, cx: &mut AsyncApp| {
                                        let instant = Instant::now();
                                        while instant.elapsed().as_secs_f32() < 0.5 {
                                            job.update_job_progress(
                                                (instant.elapsed().as_secs_f32() * 1000.) as u64,
                                                500,
                                            );
                                            cx.background_executor()
                                                .timer(Duration::from_millis(10))
                                                .await;
                                        }
                                        job.update_job_status(
                                            tr!("STANDARD_JOB_COMPLETE_DESCRIPTION"),
                                            JobStatus::Completed,
                                        );
                                    }
                                })
                                .detach();
                                cx.update_global::<JobManager, ()>(|job_manager, cx| {
                                    job_manager
                                        .track_job_delayed_default(job.make_job_entity(cx), cx);
                                });
                            })),
                    ),
            )
            .child(
                button("delayed-tracking-slow")
                    .child(tr!(
                        "JOB_DELAYED_TRACKING_SLOW",
                        "Start Slow Job with Delayed Tracking"
                    ))
                    .on_click(cx.listener(|_this, _, _, cx| {
                        let job = StandardJob::new(
                            tr!("JOB_DELAYED_TRACKING_SLOW_TITLE", "Slow Delayed Tracking"),
                            tr!(
                                "JOB_DELAYED_TRACKING_SLOW_IN_PROGRESS_DESCRIPTION",
                                "This job will show up in the job menu because it takes some \
                                time to finish."
                            ),
                        );

                        cx.spawn({
                            let mut job = job.clone();
                            async move |_, cx: &mut AsyncApp| {
                                let instant = Instant::now();
                                while instant.elapsed().as_secs_f32() < 10. {
                                    job.update_job_progress(
                                        (instant.elapsed().as_secs_f32() * 1000.) as u64,
                                        10000,
                                    );
                                    cx.background_executor()
                                        .timer(Duration::from_millis(10))
                                        .await;
                                }
                                job.update_job_status(
                                    tr!("STANDARD_JOB_COMPLETE_DESCRIPTION"),
                                    JobStatus::Completed,
                                );
                            }
                        })
                        .detach();
                        cx.update_global::<JobManager, ()>(|job_manager, cx| {
                            job_manager.track_job_delayed_default(job.make_job_entity(cx), cx);
                        });
                    })),
            )
            .child(
                button("delayed-tracking-quick-fail")
                    .child(tr!(
                        "JOB_DELAYED_TRACKING_QUICK_FAIL",
                        "Start Failing Job with Delayed Tracking"
                    ))
                    .on_click(cx.listener(|_this, _, _, cx| {
                        let job = StandardJob::new(
                            tr!(
                                "JOB_DELAYED_TRACKING_QUICK_FAIL_TITLE",
                                "Failing Job with Delayed Tracking"
                            ),
                            tr!(
                                "JOB_DELAYED_TRACKING_QUICK_FAIL_IN_PROGRESS_DESCRIPTION",
                                "This job shows up in the job menu before the specified delay \
                                because it fails before then."
                            ),
                        );

                        cx.spawn({
                            let mut job = job.clone();
                            async move |_, cx: &mut AsyncApp| {
                                let instant = Instant::now();
                                while instant.elapsed().as_secs_f32() < 0.5 {
                                    job.update_job_progress(
                                        (instant.elapsed().as_secs_f32() * 1000.) as u64,
                                        500,
                                    );
                                    cx.background_executor()
                                        .timer(Duration::from_millis(10))
                                        .await;
                                }
                                job.update_job_status(
                                    tr!("FAILING_JOB_COMPLETE_DESCRIPTION"),
                                    JobStatus::Failed,
                                );
                            }
                        })
                        .detach();
                        cx.update_global::<JobManager, ()>(|job_manager, cx| {
                            job_manager.track_job_delayed_default(job.make_job_entity(cx), cx);
                        });
                    })),
            )
    }
}

impl Render for Jobs {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .bg(theme.background)
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .child(
                grandstand("jobs-grandstand")
                    .text(tr!("JOBS_TITLE", "Jobs"))
                    .pt(px(36.)),
            )
            .child(
                scroll_area_cx(
                    "jobs-scrollable",
                    move |this, window, cx| {
                        constrainer("jobs")
                            .flex()
                            .flex_col()
                            .w_full()
                            .p(px(8.))
                            .gap(px(8.))
                            .child(this.render_jobs_section(window, cx))
                            .child(this.render_delayed_tracking_section(window, cx))
                            .child(
                                layer()
                                    .flex()
                                    .flex_col()
                                    .p(px(8.))
                                    .w_full()
                                    .child(subtitle(tr!("JOB_MENU_TITLE", "Job Menu")))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(8.))
                                            .child(tr!(
                                                "JOB_MENU_DESCRIPTION",
                                                "If you need to, (for example, \
                                                if a job requires attention soon after \
                                                it is started) you can programmatically \
                                                open the job menu."
                                            ))
                                            .child(
                                                button("open-job-menu")
                                                    .child(tr!("OPEN_JOB_MENU", "Open Job Menu"))
                                                    .on_click(cx.listener(|_this, _, _, cx| {
                                                        cx.update_global::<JobManager, ()>(
                                                            |job_manager, cx| {
                                                                job_manager
                                                                    .set_job_menu_open(true, cx);
                                                            },
                                                        )
                                                    })),
                                            ),
                                    ),
                            )
                    },
                    cx,
                )
                .flex_grow(1.),
            )
    }
}
