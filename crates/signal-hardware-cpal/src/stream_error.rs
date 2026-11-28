//! Stream-error handling shared by the output and input backends.
//!
//! Since cpal 0.18 the error callback can run on the OS audio thread:
//! CoreAudio reports `Xrun` from its processor-overload listener, which fires
//! on the realtime thread, and other backends report from their audio worker.
//! The callback built here therefore follows the realtime contract. It
//! classifies the error by kind, stores an atomic, and parks a fatal error in
//! a slot through a non-blocking `try_lock`. It never formats, allocates, or
//! blocks; the handle formats the detail on the caller's thread.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

/// Most recent fatal stream error, formatted on demand by the handle.
pub(crate) type ErrorSlot = Arc<Mutex<Option<cpal::Error>>>;

/// Whether `kind` ends the stream. Xruns, refused realtime promotion, and an
/// automatic reroute to the new default device leave the stream running, so
/// they do not fault it. Everything else, including device loss and
/// invalidation after a format change, faults the stream.
fn is_fatal(kind: cpal::ErrorKind) -> bool {
    !matches!(
        kind,
        cpal::ErrorKind::Xrun | cpal::ErrorKind::RealtimeDenied | cpal::ErrorKind::DeviceChanged
    )
}

/// Build the cpal error callback for one stream. A fatal error stores
/// `faulted` into `state` and records the error in `slot`.
pub(crate) fn error_callback(
    state: Arc<AtomicU8>,
    faulted: u8,
    slot: ErrorSlot,
) -> impl FnMut(cpal::Error) + Send + 'static {
    move |error| {
        if !is_fatal(error.kind()) {
            return;
        }
        state.store(faulted, Ordering::Relaxed);
        // `try_lock` never blocks. If the handle is reading the slot at this
        // instant the detail is dropped, but the fault still stands. Moving
        // the error in does not allocate; replacing an earlier detail can
        // free its owned message, which only happens after the stream has
        // already faulted.
        if let Ok(mut detail) = slot.try_lock() {
            *detail = Some(error);
        }
    }
}

/// Format the recorded fatal error, if any. Runs on the caller's thread.
pub(crate) fn last_error(slot: &ErrorSlot) -> Option<String> {
    slot.lock()
        .ok()
        .and_then(|detail| detail.as_ref().map(ToString::to_string))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUNNING: u8 = 0;
    const FAULTED: u8 = 2;

    fn callback_with_state() -> (Arc<AtomicU8>, ErrorSlot, impl FnMut(cpal::Error)) {
        let state = Arc::new(AtomicU8::new(RUNNING));
        let slot: ErrorSlot = Arc::new(Mutex::new(None));
        let callback = error_callback(Arc::clone(&state), FAULTED, Arc::clone(&slot));
        (state, slot, callback)
    }

    #[test]
    fn non_fatal_kinds_leave_the_stream_running() {
        let (state, slot, mut callback) = callback_with_state();
        for kind in [
            cpal::ErrorKind::Xrun,
            cpal::ErrorKind::RealtimeDenied,
            cpal::ErrorKind::DeviceChanged,
        ] {
            callback(kind.into());
        }
        assert_eq!(state.load(Ordering::Relaxed), RUNNING);
        assert_eq!(last_error(&slot), None);
    }

    #[test]
    fn device_loss_faults_with_detail() {
        let (state, slot, mut callback) = callback_with_state();
        callback(cpal::Error::with_message(
            cpal::ErrorKind::DeviceNotAvailable,
            "Device disconnected",
        ));
        assert_eq!(state.load(Ordering::Relaxed), FAULTED);
        assert_eq!(last_error(&slot).as_deref(), Some("Device disconnected"));
    }

    #[test]
    fn stream_invalidation_faults() {
        let (state, slot, mut callback) = callback_with_state();
        callback(cpal::ErrorKind::StreamInvalidated.into());
        assert_eq!(state.load(Ordering::Relaxed), FAULTED);
        assert!(last_error(&slot).is_some());
    }
}
