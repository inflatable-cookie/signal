use std::sync::atomic::Ordering;

use signal_render_plane::{
    PluginBlockProcessor, RenderBlockPluginEvent, RenderPluginEventKind, RenderPluginEventSupport,
};

use super::super::common::convert_block_events;
use super::InProcessVst3Processor;

impl PluginBlockProcessor for InProcessVst3Processor {
    fn process(&self, scratch: &mut [f32], frame_count: usize, channels: usize) -> bool {
        self.process_with_events(scratch, frame_count, channels, &[])
    }

    fn event_support(&self) -> RenderPluginEventSupport {
        RenderPluginEventSupport {
            notes: true,
            control_change: self.midi_cc_mappings.iter().any(|mapped| *mapped),
            pitch_bend: self.pitch_bend_mapping,
            channel_pressure: self.channel_pressure_mapping,
            note_expression: false,
        }
    }

    fn unsupported_event_count(&self) -> u64 {
        self.unsupported_events.load(Ordering::Relaxed)
    }

    fn latency_frames(&self) -> u32 {
        self.refresh_latency();
        self.latency_frames.load(Ordering::Relaxed)
    }

    fn latency_revision(&self) -> u64 {
        self.refresh_latency();
        self.latency_revision.load(Ordering::Relaxed)
    }

    fn process_with_events(
        &self,
        scratch: &mut [f32],
        frame_count: usize,
        channels: usize,
        events: &[RenderBlockPluginEvent],
    ) -> bool {
        let unsupported = events
            .iter()
            .filter(|event| match event.kind {
                RenderPluginEventKind::NoteOn { .. } | RenderPluginEventKind::NoteOff { .. } => {
                    false
                }
                RenderPluginEventKind::ControlChange { controller, .. } => {
                    !self.midi_cc_mappings[usize::from(controller)]
                }
                RenderPluginEventKind::PitchBend { .. } => !self.pitch_bend_mapping,
                RenderPluginEventKind::ChannelPressure { .. } => !self.channel_pressure_mapping,
                RenderPluginEventKind::NoteExpression { .. } => true,
                RenderPluginEventKind::VoiceStart { .. }
                | RenderPluginEventKind::VoiceStop { .. }
                | RenderPluginEventKind::VoiceParam { .. } => true,
            })
            .count() as u64;
        self.unsupported_events
            .fetch_add(unsupported, Ordering::Relaxed);
        if !self.alive.load(Ordering::Relaxed)
            || channels != 2
            || frame_count > self.max_frames as usize
        {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        if self.processing_restart_pending() {
            if let Ok(mut session) = self.session.try_lock() {
                session.stop();
            }
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        // try_lock: never block the audio thread. Contention only happens
        // against teardown, which is about to mark the backend dead anyway.
        let Ok(mut session) = self.session.try_lock() else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        };
        let Ok(mut events_scratch) = self.events_scratch.try_lock() else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        };
        if !session.is_processing() && session.start().is_err() {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        convert_block_events(events, &mut events_scratch);
        let samples = frame_count * channels;
        if session.process_in_place_with_events(
            &mut scratch[..samples],
            frame_count,
            &events_scratch,
        ) {
            true
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            false
        }
    }
}
