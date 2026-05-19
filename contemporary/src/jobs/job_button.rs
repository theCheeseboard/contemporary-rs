use crate::components::button::button;
use crate::easing::ease_out_cubic;
use crate::jobs::job_manager::{JobButtonState, JobManager};
use crate::jobs::jobs_menu::jobs_menu;
use crate::styling::theme::ThemeStorage;
use gpui::prelude::FluentBuilder;
use gpui::{
    Anchor, App, BorderStyle, BorrowAppContext, Bounds, Context, Corners, Entity, IntoElement,
    ParentElement, Path, PathBuilder, Pixels, Point, Render, Rgba, Size, Styled, Window, canvas,
    div, px, quad, rgb, transparent_black,
};
use std::ops::Rem;
use std::time::Instant;

pub struct JobButton {
    is_first_update: bool,
    old_job_button_state: JobButtonState,
    old_job_button_state_since: Instant,
}

struct JobButtonProgressPrepaintState {
    progress_bounds: Bounds<Pixels>,
    ping_bounds: Bounds<Pixels>,
    ping_color: Rgba,
    have_ping: bool,
    progress_path: Path<Pixels>,
}

impl JobButton {
    pub fn use_job_button(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let entity = window.use_state(cx, |_window, _cx| JobButton {
            is_first_update: true,
            old_job_button_state: JobButtonState::Hidden,
            old_job_button_state_since: Instant::now(),
        });

        entity.update(cx, |job_button, cx| {
            if job_button.is_first_update {
                cx.observe_global::<JobManager>(|_, cx| cx.notify())
                    .detach();
                job_button.is_first_update = false;
            }
        });

        entity
    }
}

impl Render for JobButton {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let job_manager = cx.global::<JobManager>();

        let job_button_state = job_manager.job_button_state(cx);
        if self.old_job_button_state != job_button_state {
            self.old_job_button_state = job_button_state;
            self.old_job_button_state_since = Instant::now();
        }

        if matches!(
            job_button_state,
            JobButtonState::HaveRequiresAttention
                | JobButtonState::HaveFailure
                | JobButtonState::HaveSuccess
        ) {
            window.request_animation_frame()
        }

        let time_since_button_state_change = self.old_job_button_state_since.elapsed();

        div()
            .child(jobs_menu())
            .when(job_button_state != JobButtonState::Hidden, move |david| {
                david.child(
                    button("jobs")
                        .flat()
                        .child(
                            canvas(
                                move |bounds, _window, cx| {
                                    let job_manager = cx.global::<JobManager>();
                                    let progress_bounds = Bounds::centered_at(
                                        bounds.center(),
                                        Size::new(px(20.), px(20.)),
                                    );
                                    let ping_bounds = Bounds::from_anchor_and_size(
                                        Anchor::BottomRight,
                                        progress_bounds.bottom_right(),
                                        Size::new(px(8.), px(8.)),
                                    );

                                    let angle_point = |angle: f32| -> Point<Pixels> {
                                        Point::new(
                                            progress_bounds.center().x
                                                + px(8.
                                                    * (angle - std::f32::consts::FRAC_PI_2).cos()),
                                            progress_bounds.center().y
                                                + px(8.
                                                    * (angle - std::f32::consts::FRAC_PI_2).sin()),
                                        )
                                    };

                                    let arc_length = job_manager.aggregate_progress(cx) * 360.;
                                    let mut progress_path = PathBuilder::stroke(px(4.));
                                    progress_path.move_to(angle_point(0.));
                                    if arc_length == 360. {
                                        // Go halfway around the circle first, otherwise we'll go nowhere!
                                        progress_path.arc_to(
                                            Point::new(px(8.), px(8.)),
                                            px(180. * std::f32::consts::PI / 180.),
                                            true,
                                            true,
                                            angle_point(180. * std::f32::consts::PI / 180.),
                                        );
                                    }

                                    progress_path.arc_to(
                                        Point::new(px(8.), px(8.)),
                                        px(arc_length * std::f32::consts::PI / 180.),
                                        arc_length > 180.,
                                        true,
                                        angle_point(arc_length * std::f32::consts::PI / 180.),
                                    );

                                    JobButtonProgressPrepaintState {
                                        progress_bounds,
                                        ping_bounds,
                                        progress_path: progress_path.build().unwrap(),
                                        ping_color: match job_button_state {
                                            JobButtonState::Hidden => transparent_black().into(),
                                            JobButtonState::Idle => transparent_black().into(),
                                            JobButtonState::HaveRequiresAttention => rgb(0xFFFF00),
                                            JobButtonState::HaveFailure => rgb(0xFF0000),
                                            JobButtonState::HaveSuccess => rgb(0x00FF00),
                                        },
                                        have_ping: match job_button_state {
                                            JobButtonState::Hidden => false,
                                            JobButtonState::Idle => false,
                                            JobButtonState::HaveRequiresAttention => true,
                                            JobButtonState::HaveFailure => true,
                                            JobButtonState::HaveSuccess => true,
                                        },
                                    }
                                },
                                move |_bounds, prepaint_state, window, cx| {
                                    let theme = cx.theme();
                                    window.paint_quad(quad(
                                        prepaint_state.progress_bounds,
                                        Corners::all(
                                            prepaint_state.progress_bounds.size.height / 2.,
                                        ),
                                        transparent_black(),
                                        px(1.),
                                        theme.foreground,
                                        BorderStyle::Solid,
                                    ));
                                    window
                                        .paint_path(prepaint_state.progress_path, theme.foreground);

                                    if prepaint_state.have_ping {
                                        window.paint_quad(quad(
                                            prepaint_state.ping_bounds,
                                            Corners::all(
                                                prepaint_state.ping_bounds.size.height / 2.,
                                            ),
                                            prepaint_state.ping_color,
                                            px(0.),
                                            transparent_black(),
                                            BorderStyle::Solid,
                                        ));

                                        let animation_progress = ease_out_cubic(
                                            time_since_button_state_change.as_secs_f32().rem(2.)
                                                / 2.,
                                        );
                                        let dilated_bounds = prepaint_state.ping_bounds.dilate(
                                            prepaint_state.ping_bounds.size.width
                                                * animation_progress,
                                        );
                                        let dilated_ping_color = Rgba {
                                            a: 1. - animation_progress,
                                            ..prepaint_state.ping_color
                                        };

                                        window.paint_quad(quad(
                                            dilated_bounds,
                                            Corners::all(dilated_bounds.size.width / 2.),
                                            transparent_black(),
                                            px(1.),
                                            dilated_ping_color,
                                            BorderStyle::Solid,
                                        ));
                                    }
                                },
                            )
                            .size(px(24.)),
                        )
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.update_global::<JobManager, ()>(|manager, cx| {
                                manager.set_job_menu_open(true, cx);
                            });
                            cx.notify()
                        })),
                )
            })
    }
}
