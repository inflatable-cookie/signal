use crate::{DelayLine, OnePoleLowPass};
use signal_primitives::{FrequencyHz, Sample};

/// Apply a sample-accurate gain control buffer to an audio block.
///
/// Each sample in `block` is multiplied by the corresponding value in
/// `gain_control`, zip-truncating to the shorter slice.
pub fn apply_gain_control(block: &mut [Sample], gain_control: &[Sample]) {
    for (sample, gain) in block.iter_mut().zip(gain_control.iter().copied()) {
        *sample *= gain;
    }
}

/// Process an audio block through a low-pass filter driven by a per-sample cutoff control buffer.
///
/// The cutoff is updated only when it changes between consecutive samples to
/// avoid redundant coefficient recalculations.
pub fn process_low_pass_with_cutoff_control(
    filter: &mut OnePoleLowPass,
    block: &mut [Sample],
    cutoff_hz: &[Sample],
) {
    let mut previous_cutoff = None::<Sample>;
    for (sample, cutoff) in block.iter_mut().zip(cutoff_hz.iter().copied()) {
        if previous_cutoff != Some(cutoff) {
            filter.set_cutoff_hz(FrequencyHz(cutoff.max(0.0)));
            previous_cutoff = Some(cutoff);
        }
        *sample = filter.process_sample(*sample);
    }
}

/// Process an audio block through a delay line driven by a per-sample feedback control buffer.
///
/// The feedback is updated only when it changes between consecutive samples.
pub fn process_delay_with_feedback_control(
    delay: &mut DelayLine,
    block: &mut [Sample],
    feedback: &[Sample],
) {
    let mut previous_feedback = None::<Sample>;
    for (sample, feedback) in block.iter_mut().zip(feedback.iter().copied()) {
        if previous_feedback != Some(feedback) {
            delay.set_feedback(feedback);
            previous_feedback = Some(feedback);
        }
        *sample = delay.process_sample(*sample);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_gain_control, process_delay_with_feedback_control,
        process_low_pass_with_cutoff_control,
    };
    use crate::{ControlSegment, ControlSegmentPlayer, DelayLine, DspKernel, OnePoleLowPass};
    use signal_primitives::{FrequencyHz, SampleRate};

    #[test]
    fn apply_gain_control_follows_sample_accurate_segments() {
        let segments = [
            ControlSegment::step(0, 0.0),
            ControlSegment::linear(2, 4, 1.0),
        ];
        let mut player = ControlSegmentPlayer::new(0.0, &segments);
        let mut gain = [0.0; 6];
        player.render_block(&mut gain);

        let mut audio = [1.0; 6];
        apply_gain_control(&mut audio, &gain);

        assert_eq!(audio, [0.0, 0.0, 0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn low_pass_control_supports_block_boundary_bypass_and_reset() {
        let mut filter = OnePoleLowPass::new(SampleRate(48_000), FrequencyHz(200.0));
        let mut active_block = [1.0; 8];
        let cutoff = [400.0; 8];
        process_low_pass_with_cutoff_control(&mut filter, &mut active_block, &cutoff);

        filter.set_bypassed(true);
        let mut bypass_block = [0.25; 4];
        process_low_pass_with_cutoff_control(&mut filter, &mut bypass_block, &[400.0; 4]);
        assert_eq!(bypass_block, [0.25; 4]);

        filter.set_bypassed(false);
        let mut resumed_block = [0.25; 4];
        process_low_pass_with_cutoff_control(&mut filter, &mut resumed_block, &[400.0; 4]);
        assert!(resumed_block[0] >= 0.25);

        filter.reset();
        let mut reset_block = [1.0, 0.0, 0.0, 0.0];
        process_low_pass_with_cutoff_control(&mut filter, &mut reset_block, &[400.0; 4]);
        assert!(reset_block[0] > reset_block[1]);
    }

    #[test]
    fn delay_feedback_control_preserves_block_boundary_continuity() {
        let mut delay = DelayLine::with_max_delay(8);
        delay.set_delay_samples(2);

        let mut block_a = [1.0, 0.0, 0.0, 0.0];
        process_delay_with_feedback_control(&mut delay, &mut block_a, &[0.5; 4]);
        assert_eq!(block_a, [0.0, 0.0, 1.0, 0.0]);

        delay.set_bypassed(true);
        let mut bypass_block = [0.25, 0.25];
        process_delay_with_feedback_control(&mut delay, &mut bypass_block, &[0.5; 2]);
        assert_eq!(bypass_block, [0.25, 0.25]);

        delay.set_bypassed(false);
        let mut resumed_block = [0.0, 0.0, 0.0, 0.0];
        process_delay_with_feedback_control(&mut delay, &mut resumed_block, &[0.5; 4]);
        assert!(resumed_block.iter().any(|sample| sample.abs() > 0.0));

        delay.reset();
        let mut reset_block = [1.0, 0.0, 0.0, 0.0];
        process_delay_with_feedback_control(&mut delay, &mut reset_block, &[0.0; 4]);
        assert_eq!(reset_block, [0.0, 0.0, 1.0, 0.0]);
    }
}
