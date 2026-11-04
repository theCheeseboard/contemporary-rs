use crate::inhibit::{InhibitError, InhibitType, InternalInhibition};
use ashpd::desktop::inhibit::{InhibitFlags, InhibitProxy};
use ashpd::enumflags2::BitFlags;
use cancellation_token::{CancelOnDrop, CancellationTokenSource};
use std::thread;

pub struct InhibitResult {
    _cancel_source: CancelOnDrop,
}

pub fn inhibit(reason: &str, types: &[InhibitType]) -> Result<InhibitResult, InhibitError> {
    let cancellation_token_source = CancellationTokenSource::new();

    let flags = types.iter().fold(
        BitFlags::empty(),
        |state, inhibit_type| match inhibit_type {
            InhibitType::DisplaySleep => state | InhibitFlags::Idle,
            InhibitType::IdleSleep => state | InhibitFlags::Suspend,
            _ => state,
        },
    );

    smol::spawn({
        let reason = reason.to_string();
        let cancellation_token = cancellation_token_source.token();
        async move {
            let Ok(inhibit) = InhibitProxy::new().await else {
                return;
            };

            let Ok(inhibit_result) = inhibit.inhibit(None, flags, &reason).await else {
                return;
            };

            cancellation_token.wait().await;

            let _ = inhibit_result.close().await;
        }
    })
    .detach();

    Ok(InhibitResult {
        _cancel_source: cancellation_token_source.cancel_defer(),
    })
}

impl InternalInhibition for InhibitResult {}
