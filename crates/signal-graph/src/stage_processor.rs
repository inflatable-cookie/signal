//! Stage processor for graph execution.
//!
//! This module provides [`GraphStageProcessor`] which handles the per-sample
//! processing of individual graph stages, including parameter event application
//! and interleaved audio processing.

use crate::{GraphParameterApplicationStrategy, GraphStageSpec};
use signal_primitives::AudioBuffer;

mod event_bounds;
mod processor;

pub use event_bounds::{bounded_stage_events, StageParameterEvent};
use processor::GraphStageProcessor;

pub fn apply_stage(
    buffer: &mut AudioBuffer,
    stage: &GraphStageSpec,
    events: &[StageParameterEvent],
    strategy: Option<GraphParameterApplicationStrategy>,
) {
    let strategy = strategy.unwrap_or_default();
    let (events, _) = bounded_stage_events(events, strategy);
    let mut processor = GraphStageProcessor::new(stage);
    let mut frame_cursor = 0;
    let mut event_cursor = 0;

    while frame_cursor < buffer.frames().0 {
        while let Some(event) = events.get(event_cursor).copied() {
            if event.sample_offset != frame_cursor {
                break;
            }
            processor.set_parameter(event.value);
            event_cursor += 1;
        }

        let next_boundary = events
            .get(event_cursor)
            .map(|event| event.sample_offset)
            .unwrap_or(buffer.frames().0)
            .max(frame_cursor.saturating_add(1))
            .min(buffer.frames().0);
        let channel_count = buffer.channel_count().0;
        let sample_start = frame_cursor.saturating_mul(channel_count);
        let sample_end = next_boundary.saturating_mul(channel_count);
        processor.process_interleaved(
            &mut buffer.samples_mut()[sample_start..sample_end],
            channel_count,
        );
        frame_cursor = next_boundary;
    }
}
