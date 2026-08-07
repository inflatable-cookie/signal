use std::sync::atomic::Ordering;

use crate::{PluginBlockProcessor, RenderBlockPluginEvent};

use super::bank::BinauralVoiceBank;

impl PluginBlockProcessor for BinauralVoiceBank {
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

    fn unsupported_event_count(&self) -> u64 {
        self.unsupported_events.load(Ordering::Relaxed)
    }
}
