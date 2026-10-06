use crate::components::layer::layer;
use crate::components::progress_bar::progress_bar;
use crate::components::subtitle::subtitle;
use crate::jobs::job::{Job, JobStatus};
use async_channel::Sender;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, AppContext, Entity, IntoElement, ParentElement, SharedString, Styled, px,
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct StandardJob {
    inner: Rc<RefCell<StandardJobInner>>,
}

pub struct StandardJobInner {
    progress: u64,
    max_progress: u64,
    title: SharedString,
    description: SharedString,
    status: JobStatus,
    cancellation_callback: Option<Box<dyn Fn()>>,
    transient: bool,
    notify_channels: Vec<Sender<()>>,
}

impl StandardJobInner {
    pub fn new(title: impl Into<SharedString>, description: impl Into<SharedString>) -> Self {
        StandardJobInner {
            progress: 0,
            max_progress: 100,
            title: title.into(),
            description: description.into(),
            status: JobStatus::InProgress,
            cancellation_callback: None,
            transient: false,
            notify_channels: Vec::new(),
        }
    }
}

impl StandardJob {
    pub fn new(title: impl Into<SharedString>, description: impl Into<SharedString>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(StandardJobInner::new(title, description))),
        }
    }

    pub fn new_indeterminate(
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
    ) -> Self {
        Self {
            inner: Rc::new(RefCell::new(StandardJobInner {
                max_progress: 0,
                ..StandardJobInner::new(title, description)
            })),
        }
    }

    pub fn new_transient(
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
    ) -> Self {
        Self {
            inner: Rc::new(RefCell::new(StandardJobInner {
                transient: true,
                ..StandardJobInner::new(title, description)
            })),
        }
    }

    pub fn update_job_status(&mut self, description: impl Into<SharedString>, status: JobStatus) {
        self.inner.borrow_mut().description = description.into();
        self.inner.borrow_mut().status = status;
        self.notify_all_channels();
    }

    pub fn update_job_progress(&mut self, progress: u64, max_progress: u64) {
        self.inner.borrow_mut().progress = progress;
        self.inner.borrow_mut().max_progress = max_progress;
        self.notify_all_channels();
    }

    pub fn set_job_progress_indeterminate(&mut self) {
        self.inner.borrow_mut().max_progress = 0;
        self.notify_all_channels();
    }

    pub fn with_cancellation_callback(self, callback: impl Fn() + 'static) -> Self {
        self.inner.borrow_mut().cancellation_callback = Some(Box::new(callback));
        self
    }

    fn notify_all_channels(&mut self) {
        self.inner
            .borrow_mut()
            .notify_channels
            .retain(|channel| channel.try_send(()).is_ok())
    }

    pub fn make_job_entity(&self, cx: &mut App) -> Entity<Box<dyn Job>> {
        let (tx, rx) = async_channel::bounded(1);
        self.inner.borrow_mut().notify_channels.push(tx);
        cx.new::<Box<dyn Job>>(|cx| {
            cx.spawn(async move |weak_this, cx| {
                while let Ok(()) = rx.recv().await {
                    if weak_this
                        .update(cx, |_, cx| {
                            cx.notify();
                        })
                        .is_err()
                    {
                        return;
                    }
                }
            })
            .detach();

            Box::new(self.clone())
        })
    }
}

impl Job for StandardJob {
    fn progress(&self) -> f32 {
        let inner = self.inner.borrow();
        match inner.status {
            JobStatus::InProgress | JobStatus::RequiresAttention | JobStatus::Failed => {
                if inner.max_progress == 0 {
                    0.
                } else {
                    inner.progress as f32 / inner.max_progress as f32
                }
            }
            JobStatus::Completed => 1.,
        }
    }

    fn progress_indeterminate(&self) -> bool {
        let inner = self.inner.borrow();
        match inner.status {
            JobStatus::InProgress | JobStatus::RequiresAttention => inner.max_progress == 0,
            JobStatus::Completed | JobStatus::Failed => false,
        }
    }

    fn status(&self) -> JobStatus {
        let inner = self.inner.borrow();
        inner.status
    }

    fn transient(&self) -> bool {
        let inner = self.inner.borrow();
        inner.transient
    }

    fn element(&self) -> AnyElement {
        let inner = self.inner.borrow();
        layer()
            .flex()
            .flex_col()
            .w_full()
            .p(px(10.))
            .gap(px(6.))
            .child(subtitle(inner.title.clone()))
            .child(inner.description.clone())
            .when(inner.status == JobStatus::InProgress, |david| {
                if self.progress_indeterminate() {
                    david.child(progress_bar().indeterminate("indeterminate-bar"))
                } else {
                    david.child(progress_bar().value(self.progress()))
                }
            })
            .into_any_element()
    }
}
