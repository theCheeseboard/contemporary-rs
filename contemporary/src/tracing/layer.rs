use crate::tracing::application_log_entry::ApplicationLogEntry;
use crate::tracing::visitor::ContemporaryVisitor;
use async_channel::Sender;
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

pub struct ContemporaryLayer {
    channel_tx: Sender<ApplicationLogEntry>,
}

impl ContemporaryLayer {
    pub fn new(channel_tx: Sender<ApplicationLogEntry>) -> Self {
        Self { channel_tx }
    }
}

impl<S> Layer<S> for ContemporaryLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let level = *event.metadata().level();
        let target = event.metadata().target().to_string();
        let mut visitor = ContemporaryVisitor::new();
        event.record(&mut visitor);

        let channel_tx = self.channel_tx.clone();
        smol::spawn(async move {
            channel_tx
                .send(ApplicationLogEntry {
                    level,
                    timestamp: chrono::Local::now(),
                    target: target.into(),
                    message: visitor.message().into(),
                })
                .await
        })
        .detach()
    }
}
