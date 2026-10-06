use crate::jobs::job::{Job, JobStatus};
use gpui::{App, AppContext, AsyncApp, BorrowAppContext, Entity, Global};

use cancellation_token::CancellationTokenSource;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

pub type JobEntity = Entity<Box<dyn Job>>;

pub struct JobManager {
    jobs: Vec<JobEntity>,
    pub is_job_menu_open: bool,
    unfinished_jobs: Vec<JobEntity>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum JobButtonState {
    Hidden,
    Idle,
    HaveRequiresAttention,
    HaveFailure,
    HaveSuccess,
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            is_job_menu_open: false,
            unfinished_jobs: Vec::new(),
        }
    }

    pub fn track_job(&mut self, job: JobEntity, cx: &mut App) {
        cx.observe(&job, |job_entity, cx| {
            cx.update_global::<Self, ()>(|this, cx| {
                let job = job_entity.read(cx);

                if job.transient()
                    && job.status().is_complete()
                    && this.unfinished_jobs.contains(&job_entity)
                {
                    this.unfinished_jobs.retain(|job| *job != job_entity);
                }
            })
        })
        .detach();
        self.jobs.push(job.clone());
        self.unfinished_jobs.push(job);
    }

    pub fn track_job_delayed(&mut self, job: JobEntity, delay: Duration, cx: &mut App) {
        let cancellation_token_source = CancellationTokenSource::new();
        let cancellation_token = cancellation_token_source.token();

        let job_clone = job.clone();

        cx.spawn(async move |cx: &mut AsyncApp| {
            cx.background_executor().timer(delay).await;
            if cancellation_token.is_canceled() {
                return;
            }
            cx.update_global::<Self, ()>(|this, cx| {
                let should_track = cx.read_entity(&job_clone, |job_item, _cx| {
                    // Track the job because it's taking too long
                    job_item.status() != JobStatus::Completed
                });

                if should_track && !cancellation_token.is_canceled() {
                    this.track_job(job_clone.clone(), cx);
                }
            })
        })
        .detach();

        cx.observe(&job.clone(), move |job_entity, cx| {
            if matches!(
                job.read(cx).status(),
                JobStatus::RequiresAttention | JobStatus::Failed
            ) {
                // Immediately register the job now
                cancellation_token_source.cancel();
                cx.update_global::<Self, ()>(|this, cx| {
                    this.track_job(job_entity.clone(), cx);
                })
            }
        })
        .detach();
    }

    pub fn track_job_delayed_default(&mut self, job: JobEntity, cx: &mut App) {
        self.track_job_delayed(job, Duration::from_secs(1), cx);
    }

    fn tracked_jobs(&self, cx: &App) -> impl Iterator<Item = &JobEntity> {
        self.jobs.iter().filter(|job| {
            let job = job.read(cx);
            !job.transient()
                || matches!(
                    job.status(),
                    JobStatus::InProgress | JobStatus::RequiresAttention
                )
        })
    }

    pub fn job_len(&self, cx: &App) -> usize {
        self.tracked_jobs(cx).count()
    }

    pub fn job(&self, index: usize, cx: &App) -> Option<&JobEntity> {
        self.tracked_jobs(cx).nth(index)
    }

    pub fn aggregate_progress(&self, cx: &App) -> f32 {
        self.unfinished_jobs
            .iter()
            .map(|job| job.read(cx).progress())
            .sum::<f32>()
            / self.unfinished_jobs.len() as f32
    }

    pub fn set_job_menu_open(&mut self, open: bool, cx: &App) {
        self.is_job_menu_open = open;
        if open {
            let complete_jobs: Vec<_> = self
                .tracked_jobs(cx)
                .filter(|job| job.read(cx).status().is_complete())
                .cloned()
                .collect();
            self.unfinished_jobs
                .retain(|job| !complete_jobs.iter().any(|complete_job| complete_job == job));
        }
    }

    pub fn job_button_state(&self, cx: &App) -> JobButtonState {
        let unfinished_jobs: Vec<_> = self
            .unfinished_jobs
            .iter()
            .map(|job| job.read(cx))
            .collect();
        if unfinished_jobs.is_empty() {
            JobButtonState::Hidden
        } else if unfinished_jobs
            .iter()
            .any(|job| job.status() == JobStatus::RequiresAttention)
        {
            JobButtonState::HaveRequiresAttention
        } else if unfinished_jobs
            .iter()
            .any(|job| job.status() == JobStatus::Failed)
        {
            JobButtonState::HaveFailure
        } else if unfinished_jobs
            .iter()
            .any(|job| job.status() == JobStatus::Completed)
        {
            JobButtonState::HaveSuccess
        } else {
            JobButtonState::Idle
        }
    }
}

impl Global for JobManager {}
