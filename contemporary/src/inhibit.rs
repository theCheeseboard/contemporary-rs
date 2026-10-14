pub struct Inhibition {
    reason: String,
    _internal: Box<dyn InternalInhibition>,
}

#[derive(Debug, Clone, Copy)]
pub enum InhibitError {
    InternalError,
}

#[derive(Debug, Clone, Copy)]
pub enum InhibitType {
    DisplaySleep,
    IdleSleep,
}

pub(crate) trait InternalInhibition {}

impl InternalInhibition for () {}

impl Inhibition {
    pub fn new(reason: &str, types: &[InhibitType]) -> Result<Self, InhibitError> {
        Ok(Inhibition {
            reason: reason.to_string(),
            _internal: cfg_select! {
                target_os = "macos" => {
                    Box::new(crate::platform_support::macos::inhibit::inhibit(reason, types)?)
                }
                // Inhibitions not supported on this platform
                _ => Box::new(())
            },
        })
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}
