use super::super::*;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

pub(in crate::tests) struct FakeGainProcessor {
    pub(in crate::tests) gain: f32,
    pub(in crate::tests) calls: AtomicU64,
}

impl PluginBlockProcessor for FakeGainProcessor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.calls.fetch_add(1, Ordering::Relaxed);
        for sample in &mut scratch[..frame_count * channels] {
            *sample *= self.gain;
        }
        true
    }
}

/// Fake backend that always misses: returns `false` and must leave the
/// scratch untouched (the bypass contract under test).
pub(in crate::tests) struct AlwaysMissProcessor {
    pub(in crate::tests) misses: AtomicU64,
}

impl PluginBlockProcessor for AlwaysMissProcessor {
    fn process(&self, _scratch: &mut [f32], _frames: usize, _channels: usize) -> bool {
        self.misses.fetch_add(1, Ordering::Relaxed);
        false
    }
}

/// Minimal alloc-free instrument backend: note-on starts a constant
/// signal at the event velocity; note-off returns to silence.
pub(in crate::tests) struct EventInstrumentProcessor {
    pub(in crate::tests) amplitude_bits: AtomicU32,
}

impl EventInstrumentProcessor {
    fn render(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        let mut amplitude = f32::from_bits(self.amplitude_bits.load(Ordering::Relaxed));
        let mut event_index = 0;
        for frame in 0..frame_count {
            while event_index < events.len() && events[event_index].offset_frames as usize == frame
            {
                amplitude = match events[event_index].kind {
                    RenderPluginEventKind::NoteOn { velocity, .. } => velocity,
                    RenderPluginEventKind::NoteOff { .. } => 0.0,
                    RenderPluginEventKind::ControlChange { .. }
                    | RenderPluginEventKind::PitchBend { .. }
                    | RenderPluginEventKind::ChannelPressure { .. }
                    | RenderPluginEventKind::NoteExpression { .. }
                    | RenderPluginEventKind::VoiceStart { .. }
                    | RenderPluginEventKind::VoiceStop { .. }
                    | RenderPluginEventKind::VoiceParam { .. } => amplitude,
                };
                event_index += 1;
            }
            for channel in 0..channels {
                scratch[frame * channels + channel] = amplitude;
            }
        }
        self.amplitude_bits
            .store(amplitude.to_bits(), Ordering::Relaxed);
        true
    }
}

impl PluginBlockProcessor for EventInstrumentProcessor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.render(scratch, frame_count, channels, &[])
    }

    fn process_with_events(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        self.render(scratch, frame_count, channels, events)
    }
}

pub(in crate::tests) struct RecordingEventProcessor {
    calls: std::sync::Mutex<Vec<Vec<RenderBlockPluginEvent>>>,
}

impl RecordingEventProcessor {
    pub(in crate::tests) fn new() -> Arc<Self> {
        Arc::new(Self {
            calls: std::sync::Mutex::new(Vec::new()),
        })
    }

    pub(in crate::tests) fn calls(&self) -> Vec<Vec<RenderBlockPluginEvent>> {
        self.calls.lock().unwrap().clone()
    }
}

impl PluginBlockProcessor for RecordingEventProcessor {
    fn process(&self, _scratch: &mut [f32], _frames: usize, _channels: usize) -> bool {
        self.calls
            .lock()
            .unwrap()
            .push(vec![RenderBlockPluginEvent {
                offset_frames: u32::MAX,
                channel: 0,
                kind: RenderPluginEventKind::NoteOff { key: 0 },
            }]);
        true
    }

    fn process_with_events(
        &self,
        _scratch: &mut [f32],
        _frames: usize,
        _channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        self.calls.lock().unwrap().push(events.to_vec());
        true
    }
}
