use crate::{flush_denormal, DspKernel};
use signal_primitives::Sample;

#[derive(Clone, Debug, PartialEq)]
pub struct DelayLine {
    buffer: Vec<Sample>,
    write_index: usize,
    delay_samples: usize,
    feedback: Sample,
    bypassed: bool,
}

impl DelayLine {
    pub fn with_max_delay(max_delay_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_delay_samples.saturating_add(1)],
            write_index: 0,
            delay_samples: max_delay_samples.min(1),
            feedback: 0.0,
            bypassed: false,
        }
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len().saturating_sub(1)
    }

    pub fn delay_samples(&self) -> usize {
        self.delay_samples
    }

    pub fn set_delay_samples(&mut self, delay_samples: usize) {
        self.delay_samples = delay_samples.min(self.capacity());
    }

    pub fn set_feedback(&mut self, feedback: Sample) {
        self.feedback = feedback.clamp(-0.999, 0.999);
    }

    pub fn tap(&self, delay_samples: usize) -> Sample {
        if self.buffer.is_empty() {
            return 0.0;
        }

        let delay_samples = delay_samples.min(self.capacity());
        if delay_samples == 0 {
            return self.buffer[self.write_index];
        }

        let len = self.buffer.len();
        let read_index = (self.write_index + len - delay_samples) % len;
        self.buffer[read_index]
    }

    pub fn process_sample(&mut self, input: Sample) -> Sample {
        if self.buffer.is_empty() {
            return input;
        }

        if self.bypassed {
            self.buffer[self.write_index] = input;
            self.write_index = (self.write_index + 1) % self.buffer.len();
            return input;
        }

        if self.delay_samples == 0 {
            self.buffer[self.write_index] = input;
            self.write_index = (self.write_index + 1) % self.buffer.len();
            return input;
        }

        let delayed = self.tap(self.delay_samples);
        self.buffer[self.write_index] = flush_denormal(input + delayed * self.feedback);
        self.write_index = (self.write_index + 1) % self.buffer.len();
        delayed
    }
}

impl DspKernel for DelayLine {
    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_index = 0;
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    fn process_in_place(&mut self, block: &mut [Sample]) {
        for sample in block {
            *sample = self.process_sample(*sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DelayLine;
    use crate::DspKernel;

    #[test]
    fn delay_line_emits_impulse_at_requested_delay() {
        let mut delay = DelayLine::with_max_delay(4);
        delay.set_delay_samples(2);

        let mut block = [1.0, 0.0, 0.0, 0.0];
        delay.process_in_place(&mut block);

        assert_eq!(block, [0.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn delay_line_feedback_recirculates_signal() {
        let mut delay = DelayLine::with_max_delay(4);
        delay.set_delay_samples(1);
        delay.set_feedback(0.5);

        let mut block = [1.0, 0.0, 0.0, 0.0];
        delay.process_in_place(&mut block);

        assert_eq!(block[0], 0.0);
        assert_eq!(block[1], 1.0);
        assert_eq!(block[2], 0.5);
    }

    #[test]
    fn delay_line_bypass_keeps_audio_transparent() {
        let mut delay = DelayLine::with_max_delay(8);
        delay.set_delay_samples(4);
        delay.set_bypassed(true);

        let mut block = [0.1, 0.2, 0.3];
        delay.process_in_place(&mut block);

        assert_eq!(block, [0.1, 0.2, 0.3]);
    }
}
