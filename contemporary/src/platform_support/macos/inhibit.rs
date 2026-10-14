use crate::inhibit::{InhibitError, InhibitType, InternalInhibition};
use objc2_core_foundation::CFString;
use objc2_io_kit::{
    IOPMAssertionCreateWithName, IOPMAssertionID, IOPMAssertionRelease, kIOPMAssertionLevelOn,
    kIOReturnSuccess,
};

struct Inhibition(IOPMAssertionID);

pub struct InhibitResult {
    _inhibitions: Vec<Inhibition>,
}

pub fn inhibit(reason: &str, types: &[InhibitType]) -> Result<InhibitResult, InhibitError> {
    Ok(InhibitResult {
        _inhibitions: types
            .iter()
            .map(|inhibit_type| inhibit_internal(reason, inhibit_type))
            .try_fold(Vec::new(), |mut acc, inhibition| match inhibition {
                Ok(Some(result)) => {
                    acc.push(result);
                    Ok(acc)
                }
                Ok(None) => Ok(acc),
                Err(err) => Err(err),
            })?,
    })
}

impl Drop for Inhibition {
    fn drop(&mut self) {
        IOPMAssertionRelease(self.0);
    }
}

impl InternalInhibition for InhibitResult {}

fn inhibit_internal(
    reason: &str,
    inhibit_type: &InhibitType,
) -> Result<Option<Inhibition>, InhibitError> {
    let mut assertion_id = IOPMAssertionID::default();
    let reason = CFString::from_str(reason);
    let assertion_type = match inhibit_type {
        InhibitType::DisplaySleep => CFString::from_static_str("PreventUserIdleDisplaySleep"),
        InhibitType::IdleSleep => CFString::from_static_str("PreventUserIdleSystemSleep"),

        // Other inhibitions are not supported
        _ => return Ok(None),
    };

    let result = unsafe {
        IOPMAssertionCreateWithName(
            Some(&assertion_type),
            kIOPMAssertionLevelOn,
            Some(&reason),
            &mut assertion_id,
        )
    };

    if result == kIOReturnSuccess {
        Ok(Some(Inhibition(assertion_id)))
    } else {
        Err(InhibitError::InternalError)
    }
}
