use crate::tracing::application_log_entry::ApplicationLogEntry;
use async_channel::Receiver;
use gpui::{App, AsyncApp, Global};
use std::collections::VecDeque;

pub struct ApplicationLog {
    log_entries: VecDeque<ApplicationLogEntry>,
}

impl ApplicationLog {
    pub fn new(cx: &mut App, receiver: Receiver<ApplicationLogEntry>) -> Self {
        cx.spawn(async move |cx: &mut AsyncApp| {
            loop {
                let Ok(entry) = receiver.recv().await else {
                    return;
                };
                cx.update_global::<ApplicationLog, ()>(|application_log, _| {
                    while application_log.log_entries.len() > 9999 {
                        application_log.log_entries.pop_front();
                    }
                    application_log.log_entries.push_back(entry);
                })
            }
        })
        .detach();

        Self {
            log_entries: VecDeque::new(),
        }
    }

    pub fn entries(&self) -> &VecDeque<ApplicationLogEntry> {
        &self.log_entries
    }
}

impl Global for ApplicationLog {}
