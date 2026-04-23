use signal_primitives::Sample;

use super::SmoothedValue;

/// Shape of a control segment transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlSegmentShape {
    /// Jump immediately to the target value.
    Step,
    /// Ramp linearly from the current value to the target over the segment duration.
    Linear,
    /// Ramp exponentially from the current value to the target over the segment duration.
    Exponential,
}

/// One scheduled control change in sample frames.
///
/// Segments are interpreted relative to the player's current frame cursor.
/// `Step` applies immediately at `start_frame`, while `Linear` and
/// `Exponential` begin a ramp toward `target` over `duration_samples`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlSegment {
    start_frame: usize,
    target: Sample,
    duration_samples: usize,
    shape: ControlSegmentShape,
}

impl ControlSegment {
    /// Create a step segment that applies immediately.
    pub fn step(start_frame: usize, target: Sample) -> Self {
        Self {
            start_frame,
            target,
            duration_samples: 0,
            shape: ControlSegmentShape::Step,
        }
    }

    /// Create a linear ramp segment.
    pub fn linear(start_frame: usize, duration_samples: usize, target: Sample) -> Self {
        Self {
            start_frame,
            target,
            duration_samples,
            shape: ControlSegmentShape::Linear,
        }
    }

    /// Create an exponential ramp segment.
    pub fn exponential(start_frame: usize, duration_samples: usize, target: Sample) -> Self {
        Self {
            start_frame,
            target,
            duration_samples,
            shape: ControlSegmentShape::Exponential,
        }
    }

    /// Return the start frame for this segment.
    pub fn start_frame(self) -> usize {
        self.start_frame
    }

    /// Return the target value.
    pub fn target(self) -> Sample {
        self.target
    }

    /// Return the duration in samples.
    pub fn duration_samples(self) -> usize {
        self.duration_samples
    }

    /// Return the segment shape.
    pub fn shape(self) -> ControlSegmentShape {
        self.shape
    }
}

/// Sample-accurate player for an ordered list of [`ControlSegment`] values.
///
/// This is the main bridge between coarse automation intent and per-sample
/// control buffers that block DSP helpers can consume.
#[derive(Clone, Debug, PartialEq)]
pub struct ControlSegmentPlayer<'a> {
    smoothed: SmoothedValue,
    segments: &'a [ControlSegment],
    next_segment_index: usize,
    frame_cursor: usize,
}

impl<'a> ControlSegmentPlayer<'a> {
    /// Create a new player with the given initial value and segments.
    pub fn new(initial_value: Sample, segments: &'a [ControlSegment]) -> Self {
        Self {
            smoothed: SmoothedValue::new(initial_value),
            segments,
            next_segment_index: 0,
            frame_cursor: 0,
        }
    }

    /// Return the current frame position.
    pub fn current_frame(&self) -> usize {
        self.frame_cursor
    }

    /// Return the current value.
    pub fn current_value(&self) -> Sample {
        self.smoothed.current()
    }

    /// Reset to the given initial value and start from the first segment.
    pub fn reset(&mut self, initial_value: Sample) {
        self.smoothed.reset(initial_value);
        self.next_segment_index = 0;
        self.frame_cursor = 0;
    }

    /// Render a block of samples, applying any due segments.
    pub fn render_block(&mut self, output: &mut [Sample]) {
        for sample in output {
            self.apply_due_segments();
            *sample = self.smoothed.next_value();
            self.frame_cursor += 1;
        }
    }

    /// Skip ahead by the given number of frames.
    pub fn skip(&mut self, frames: usize) {
        for _ in 0..frames {
            self.apply_due_segments();
            self.smoothed.next_value();
            self.frame_cursor += 1;
        }
    }

    fn apply_due_segments(&mut self) {
        while let Some(segment) = self.segments.get(self.next_segment_index).copied() {
            if segment.start_frame() != self.frame_cursor {
                break;
            }

            match segment.shape() {
                ControlSegmentShape::Step => self.smoothed.set_immediate(segment.target()),
                ControlSegmentShape::Linear => self
                    .smoothed
                    .set_linear_target(segment.target(), segment.duration_samples()),
                ControlSegmentShape::Exponential => self
                    .smoothed
                    .set_exponential_target(segment.target(), segment.duration_samples()),
            }
            self.next_segment_index += 1;
        }
    }
}

/// Reusable control schedule consisting of an initial value plus ordered
/// segments.
///
/// Callers can create a player for incremental block rendering or ask the plan
/// to render a control block directly from an arbitrary start frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlPlan<'a> {
    initial_value: Sample,
    segments: &'a [ControlSegment],
}

impl<'a> ControlPlan<'a> {
    /// Create a new control plan.
    pub fn new(initial_value: Sample, segments: &'a [ControlSegment]) -> Self {
        Self {
            initial_value,
            segments,
        }
    }

    /// Return the initial value.
    pub fn initial_value(&self) -> Sample {
        self.initial_value
    }

    /// Return the segments slice.
    pub fn segments(&self) -> &'a [ControlSegment] {
        self.segments
    }

    /// Create a player for this plan.
    pub fn player(&self) -> ControlSegmentPlayer<'a> {
        ControlSegmentPlayer::new(self.initial_value, self.segments)
    }

    /// Render a block starting from the given frame offset.
    pub fn render_block(&self, block_start_frame: usize, output: &mut [Sample]) {
        let mut player = self.player();
        player.skip(block_start_frame);
        player.render_block(output);
    }
}
