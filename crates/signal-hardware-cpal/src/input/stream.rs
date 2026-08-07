use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use signal_hardware::{InputStreamHandle, InputStreamState};

use super::{STATE_RUNNING, STATE_STOPPED};

pub(crate) fn copy_selected_channels(
    frames: &[f32],
    physical_channels: usize,
    channel_indices: &[u16],
    output: &mut [f32],
) -> usize {
    let mut written = 0;
    for frame in frames
        .chunks_exact(physical_channels)
        .take(output.len() / channel_indices.len())
    {
        for index in channel_indices {
            output[written] = frame[*index as usize];
            written += 1;
        }
    }
    written
}

/// Handle over a stream owned by its dedicated thread. Dropping signals the
/// owner thread, which drops the stream on the thread that built it.
pub(crate) struct CpalInputStream {
    pub(crate) state: Arc<AtomicU8>,
    /// Display detail of the most recent cpal stream error. cpal delivers
    /// stream errors on a backend worker thread — NOT the audio callback —
    /// so formatting the string (allocation) and taking this mutex there is
    /// acceptable; the audio path never touches it.
    pub(crate) last_error: Arc<Mutex<Option<String>>>,
    /// Most recent ADC→callback latency in µs, stored by the data callback
    /// from cpal's `InputCallbackInfo` timestamps. 0 = not yet observed.
    pub(crate) latency_micros: Arc<AtomicU64>,
    pub(crate) sample_rate_hz: u32,
    pub(crate) channels: u16,
    pub(crate) device_name: Option<String>,
    pub(crate) stop: Option<mpsc::Sender<()>>,
    pub(crate) owner: Option<std::thread::JoinHandle<()>>,
}

impl InputStreamHandle for CpalInputStream {
    fn state(&self) -> InputStreamState {
        match self.state.load(Ordering::Relaxed) {
            STATE_RUNNING => InputStreamState::Running,
            STATE_STOPPED => InputStreamState::Stopped,
            _ => InputStreamState::Faulted,
        }
    }

    fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn last_error(&self) -> Option<String> {
        self.last_error
            .lock()
            .ok()
            .and_then(|detail| detail.clone())
    }

    fn input_latency_micros(&self) -> Option<u64> {
        match self.latency_micros.load(Ordering::Relaxed) {
            0 => None,
            micros => Some(micros),
        }
    }

    fn device_name(&self) -> Option<String> {
        self.device_name.clone()
    }
}

impl Drop for CpalInputStream {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(owner) = self.owner.take() {
            let _ = owner.join();
        }
        if self.state.load(Ordering::Relaxed) == STATE_RUNNING {
            self.state.store(STATE_STOPPED, Ordering::Relaxed);
        }
    }
}
