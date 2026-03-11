use crate::flush_denormal;
use signal_primitives::Sample;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearRamp {
    current: Sample,
    target: Sample,
    step: Sample,
    remaining_samples: usize,
}

impl LinearRamp {
    pub fn new(initial: Sample) -> Self {
        Self {
            current: initial,
            target: initial,
            step: 0.0,
            remaining_samples: 0,
        }
    }

    pub fn reset(&mut self, value: Sample) {
        self.current = value;
        self.target = value;
        self.step = 0.0;
        self.remaining_samples = 0;
    }

    pub fn set_target(&mut self, target: Sample, samples: usize) {
        self.target = target;
        if samples == 0 {
            self.reset(target);
            return;
        }

        self.remaining_samples = samples;
        self.step = (target - self.current) / samples as Sample;
    }

    pub fn current(&self) -> Sample {
        self.current
    }

    pub fn target(&self) -> Sample {
        self.target
    }

    pub fn is_active(&self) -> bool {
        self.remaining_samples > 0
    }

    pub fn next_value(&mut self) -> Sample {
        if self.remaining_samples == 0 {
            self.current = self.target;
            return self.target;
        }

        self.remaining_samples -= 1;
        if self.remaining_samples == 0 {
            self.current = self.target;
        } else {
            self.current = flush_denormal(self.current + self.step);
        }

        self.current
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExponentialRamp {
    current: Sample,
    target: Sample,
    multiplier: Sample,
    remaining_samples: usize,
}

impl ExponentialRamp {
    const MIN_MAGNITUDE: Sample = 1.0e-6;

    pub fn new(initial: Sample) -> Self {
        Self {
            current: initial,
            target: initial,
            multiplier: 1.0,
            remaining_samples: 0,
        }
    }

    pub fn reset(&mut self, value: Sample) {
        self.current = value;
        self.target = value;
        self.multiplier = 1.0;
        self.remaining_samples = 0;
    }

    pub fn set_target(&mut self, target: Sample, samples: usize) {
        let current = self.current.abs().max(Self::MIN_MAGNITUDE);
        let target = target.abs().max(Self::MIN_MAGNITUDE);
        self.target = target.copysign(self.target.signum().max(1.0));

        if samples == 0 {
            self.reset(target);
            return;
        }

        self.remaining_samples = samples;
        self.multiplier = (target / current).powf(1.0 / samples as Sample);
        if !self.multiplier.is_finite() || self.multiplier == 0.0 {
            self.multiplier = 1.0;
            self.remaining_samples = 0;
            self.current = target;
            self.target = target;
        }
    }

    pub fn current(&self) -> Sample {
        self.current
    }

    pub fn target(&self) -> Sample {
        self.target
    }

    pub fn is_active(&self) -> bool {
        self.remaining_samples > 0
    }

    pub fn next_value(&mut self) -> Sample {
        if self.remaining_samples == 0 {
            self.current = self.target;
            return self.target;
        }

        self.remaining_samples -= 1;
        if self.remaining_samples == 0 {
            self.current = self.target;
        } else {
            self.current = flush_denormal(self.current * self.multiplier);
        }

        self.current
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RampKind {
    Idle,
    Linear(LinearRamp),
    Exponential(ExponentialRamp),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmoothedValue {
    current: Sample,
    target: Sample,
    ramp: RampKind,
}

impl SmoothedValue {
    pub fn new(initial: Sample) -> Self {
        Self {
            current: initial,
            target: initial,
            ramp: RampKind::Idle,
        }
    }

    pub fn reset(&mut self, value: Sample) {
        self.current = value;
        self.target = value;
        self.ramp = RampKind::Idle;
    }

    pub fn current(&self) -> Sample {
        self.current
    }

    pub fn target(&self) -> Sample {
        self.target
    }

    pub fn is_smoothing(&self) -> bool {
        !matches!(self.ramp, RampKind::Idle)
    }

    pub fn set_immediate(&mut self, value: Sample) {
        self.reset(value);
    }

    pub fn set_linear_target(&mut self, target: Sample, samples: usize) {
        self.target = target;
        if samples == 0 {
            self.set_immediate(target);
            return;
        }

        let mut ramp = LinearRamp::new(self.current);
        ramp.set_target(target, samples);
        self.ramp = RampKind::Linear(ramp);
    }

    pub fn set_exponential_target(&mut self, target: Sample, samples: usize) {
        self.target = target.max(ExponentialRamp::MIN_MAGNITUDE);
        if samples == 0 {
            self.set_immediate(self.target);
            return;
        }

        let mut ramp = ExponentialRamp::new(self.current.max(ExponentialRamp::MIN_MAGNITUDE));
        ramp.set_target(self.target, samples);
        self.ramp = RampKind::Exponential(ramp);
    }

    pub fn next_value(&mut self) -> Sample {
        self.current = match &mut self.ramp {
            RampKind::Idle => self.target,
            RampKind::Linear(ramp) => {
                let next = ramp.next_value();
                if !ramp.is_active() {
                    self.ramp = RampKind::Idle;
                }
                next
            }
            RampKind::Exponential(ramp) => {
                let next = ramp.next_value();
                if !ramp.is_active() {
                    self.ramp = RampKind::Idle;
                }
                next
            }
        };

        self.current
    }

    pub fn fill_block(&mut self, block: &mut [Sample]) {
        for sample in block {
            *sample = self.next_value();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlSegmentShape {
    Step,
    Linear,
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
    pub fn step(start_frame: usize, target: Sample) -> Self {
        Self {
            start_frame,
            target,
            duration_samples: 0,
            shape: ControlSegmentShape::Step,
        }
    }

    pub fn linear(start_frame: usize, duration_samples: usize, target: Sample) -> Self {
        Self {
            start_frame,
            target,
            duration_samples,
            shape: ControlSegmentShape::Linear,
        }
    }

    pub fn exponential(start_frame: usize, duration_samples: usize, target: Sample) -> Self {
        Self {
            start_frame,
            target,
            duration_samples,
            shape: ControlSegmentShape::Exponential,
        }
    }

    pub fn start_frame(self) -> usize {
        self.start_frame
    }

    pub fn target(self) -> Sample {
        self.target
    }

    pub fn duration_samples(self) -> usize {
        self.duration_samples
    }

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
    pub fn new(initial_value: Sample, segments: &'a [ControlSegment]) -> Self {
        Self {
            smoothed: SmoothedValue::new(initial_value),
            segments,
            next_segment_index: 0,
            frame_cursor: 0,
        }
    }

    pub fn current_frame(&self) -> usize {
        self.frame_cursor
    }

    pub fn current_value(&self) -> Sample {
        self.smoothed.current()
    }

    pub fn reset(&mut self, initial_value: Sample) {
        self.smoothed.reset(initial_value);
        self.next_segment_index = 0;
        self.frame_cursor = 0;
    }

    pub fn render_block(&mut self, output: &mut [Sample]) {
        for sample in output {
            self.apply_due_segments();
            *sample = self.smoothed.next_value();
            self.frame_cursor += 1;
        }
    }

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
    pub fn new(initial_value: Sample, segments: &'a [ControlSegment]) -> Self {
        Self {
            initial_value,
            segments,
        }
    }

    pub fn initial_value(&self) -> Sample {
        self.initial_value
    }

    pub fn segments(&self) -> &'a [ControlSegment] {
        self.segments
    }

    pub fn player(&self) -> ControlSegmentPlayer<'a> {
        ControlSegmentPlayer::new(self.initial_value, self.segments)
    }

    pub fn render_block(&self, block_start_frame: usize, output: &mut [Sample]) {
        let mut player = self.player();
        player.skip(block_start_frame);
        player.render_block(output);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ControlPlan, ControlSegment, ControlSegmentPlayer, ExponentialRamp, LinearRamp,
        SmoothedValue,
    };

    #[test]
    fn linear_ramp_hits_target_exactly() {
        let mut ramp = LinearRamp::new(0.0);
        ramp.set_target(1.0, 4);

        let values = [
            ramp.next_value(),
            ramp.next_value(),
            ramp.next_value(),
            ramp.next_value(),
        ];

        assert_eq!(values, [0.25, 0.5, 0.75, 1.0]);
        assert!(!ramp.is_active());
    }

    #[test]
    fn exponential_ramp_converges_monotonically() {
        let mut ramp = ExponentialRamp::new(1.0);
        ramp.set_target(16.0, 4);

        let values = [
            ramp.next_value(),
            ramp.next_value(),
            ramp.next_value(),
            ramp.next_value(),
        ];

        assert!(values[0] > 1.0);
        assert!(values[1] > values[0]);
        assert!(values[2] > values[1]);
        assert_eq!(values[3], 16.0);
    }

    #[test]
    fn smoothed_value_fills_block_sample_accurately() {
        let mut value = SmoothedValue::new(0.0);
        value.set_linear_target(1.0, 4);
        let mut block = [0.0; 4];

        value.fill_block(&mut block);

        assert_eq!(block, [0.25, 0.5, 0.75, 1.0]);
        assert!(!value.is_smoothing());
    }

    #[test]
    fn control_segment_player_renders_sample_accurate_segments_across_blocks() {
        let segments = [
            ControlSegment::step(0, 0.0),
            ControlSegment::linear(2, 4, 1.0),
            ControlSegment::step(8, 0.25),
        ];
        let mut player = ControlSegmentPlayer::new(0.0, &segments);
        let mut first = [0.0; 5];
        let mut second = [0.0; 5];

        player.render_block(&mut first);
        player.render_block(&mut second);

        assert_eq!(first, [0.0, 0.0, 0.25, 0.5, 0.75]);
        assert_eq!(second, [1.0, 1.0, 1.0, 0.25, 0.25]);
        assert_eq!(player.current_frame(), 10);
        assert_eq!(player.current_value(), 0.25);
    }

    #[test]
    fn control_plan_renders_same_values_for_offset_block() {
        let segments = [
            ControlSegment::step(0, 0.0),
            ControlSegment::linear(2, 4, 1.0),
            ControlSegment::step(8, 0.25),
        ];
        let plan = ControlPlan::new(0.0, &segments);
        let mut sequential_player = plan.player();
        let mut first = [0.0; 5];
        let mut expected_second = [0.0; 5];
        let mut actual_second = [0.0; 5];

        sequential_player.render_block(&mut first);
        sequential_player.render_block(&mut expected_second);
        plan.render_block(5, &mut actual_second);

        assert_eq!(expected_second, actual_second);
    }
}
