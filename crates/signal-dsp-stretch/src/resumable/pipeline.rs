use signal_primitives::Sample;

use crate::sanitize_ratio;

use super::engine::ResumableOfflineStretch;

impl ResumableOfflineStretch {
    pub(in crate::resumable) fn push_silence(&mut self, frames: usize) {
        for _ in 0..frames {
            let ring_frame = self.input_write_frame % self.ring_frames;
            for channel in 0..self.config.channels {
                self.input_ring[ring_frame * self.config.channels + channel] = 0.0;
            }
            self.input_write_frame += 1;
        }
    }

    pub(in crate::resumable) fn push_input(&mut self, source: &[Sample], frames: usize) {
        for frame in 0..frames {
            let ring_frame = self.input_write_frame % self.ring_frames;
            for channel in 0..self.config.channels {
                self.input_ring[ring_frame * self.config.channels + channel] =
                    source[frame * self.config.channels + channel];
            }
            self.input_write_frame += 1;
        }
    }

    /// Active ratio at one padded-source position.
    fn ratio_at(&self, padded_frame: usize) -> f64 {
        let pad = self.window_size / 2;
        let source_frame = padded_frame.saturating_sub(pad);
        let mut ratio = sanitize_ratio(self.config.fallback_ratio);
        let mut best: Option<i64> = None;
        for point in &self.config.ratio_curve {
            if point.timeline_frame < 0 || !point.ratio.is_finite() || point.ratio <= 0.0 {
                continue;
            }
            if (point.timeline_frame as usize) <= source_frame
                && best.is_none_or(|b| point.timeline_frame >= b)
            {
                best = Some(point.timeline_frame);
                ratio = point.ratio;
            }
        }
        ratio
    }

    pub(in crate::resumable) fn drain(
        &mut self,
        output: &mut Vec<Sample>,
        final_pass: bool,
    ) -> usize {
        let before = self.delivered_output_frames;
        loop {
            // A frame is computable once its whole window has arrived.
            if self.next_analysis_frame + self.window_size > self.input_write_frame {
                break;
            }
            let synthesis_start = self.next_synthesis_frame.round() as usize;
            // Do not overrun the ring: emit resolved output first.
            if synthesis_start + self.window_size >= self.output_read_frame + self.ring_frames {
                self.emit(output, synthesis_start, final_pass);
                if self.output_read_frame + self.ring_frames <= synthesis_start + self.window_size {
                    break;
                }
                continue;
            }
            let ratio = self.ratio_at(self.next_analysis_frame);
            for channel in 0..self.config.channels {
                self.analyze(channel);
                self.propagate(channel, ratio);
                self.synthesize(channel, synthesis_start);
            }
            self.next_analysis_frame += self.analysis_hop;
            self.next_synthesis_frame += self.analysis_hop as f64 * ratio;
            self.frame_index += 1;
        }
        let resolved = self.next_synthesis_frame.round() as usize;
        self.emit(output, resolved, final_pass);
        self.delivered_output_frames - before
    }

    /// Emit output frames that no future analysis frame can still touch.
    fn emit(&mut self, output: &mut Vec<Sample>, synthesis_start: usize, final_pass: bool) {
        // The frame about to be written covers [synthesis_start, +window), so
        // everything below synthesis_start is final and can be released.
        let safe_until = if final_pass {
            synthesis_start + self.window_size
        } else {
            synthesis_start
        };
        while self.output_read_frame < safe_until {
            if self.delivered_output_frames >= self.target_output_frames
                && self.pending_crop_frames == 0
            {
                // Target reached: keep draining the ring so it stays clean.
                self.clear_output_frame(self.output_read_frame);
                self.output_read_frame += 1;
                continue;
            }
            let ring_frame = self.output_read_frame % self.output_ring_frames;
            if self.pending_crop_frames > 0 {
                self.pending_crop_frames -= 1;
            } else {
                for channel in 0..self.config.channels {
                    let state = &self.channels[channel];
                    let weight = state.normalization_ring[ring_frame];
                    let sample = if weight > 1.0e-3 {
                        state.output_ring[ring_frame] / weight
                    } else {
                        0.0
                    };
                    output.push(sample);
                }
                self.delivered_output_frames += 1;
            }
            self.clear_output_frame(self.output_read_frame);
            self.output_read_frame += 1;
        }
        if final_pass {
            while self.delivered_output_frames < self.target_output_frames {
                for _ in 0..self.config.channels {
                    output.push(0.0);
                }
                self.delivered_output_frames += 1;
            }
        }
    }

    fn clear_output_frame(&mut self, frame: usize) {
        let ring_frame = frame % self.output_ring_frames;
        for state in &mut self.channels {
            state.output_ring[ring_frame] = 0.0;
            state.normalization_ring[ring_frame] = 0.0;
        }
    }
}
