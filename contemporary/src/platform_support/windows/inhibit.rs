use crate::inhibit::{InhibitError, InhibitType, InternalInhibition};
use cancellation_token::{CancelOnDrop, CancellationTokenSource};
use std::thread;
use windows::Win32::System::Power::{
    ES_AWAYMODE_REQUIRED, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
    SetThreadExecutionState,
};

pub struct InhibitResult {
    _cancel_source: CancelOnDrop,
}

pub fn inhibit(_reason: &str, types: &[InhibitType]) -> Result<InhibitResult, InhibitError> {
    let requested_execution_state =
        types
            .iter()
            .fold(ES_CONTINUOUS, |state, inhibit_type| match inhibit_type {
                InhibitType::DisplaySleep => state | ES_DISPLAY_REQUIRED,
                InhibitType::IdleSleep => state | ES_AWAYMODE_REQUIRED | ES_SYSTEM_REQUIRED,
                _ => state,
            });

    let cancellation_token_source = CancellationTokenSource::new();
    thread::spawn({
        let cancellation_token = cancellation_token_source.token();
        move || {
            unsafe {
                SetThreadExecutionState(requested_execution_state);
            }
            smol::block_on(cancellation_token.wait());
            unsafe {
                SetThreadExecutionState(ES_CONTINUOUS);
            }
        }
    });

    Ok(InhibitResult {
        _cancel_source: cancellation_token_source.cancel_defer(),
    })
}

impl InternalInhibition for InhibitResult {}
