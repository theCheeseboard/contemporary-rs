use chrono::DateTime;
use gpui::SharedString;
use tracing::Level;

#[derive(Debug)]
pub struct ApplicationLogEntry {
    pub level: Level,
    pub target: SharedString,
    pub message: SharedString,
    pub timestamp: DateTime<chrono::Local>,
}
