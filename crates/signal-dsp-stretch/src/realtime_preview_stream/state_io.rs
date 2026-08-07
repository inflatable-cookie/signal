use signal_primitives::Sample;

use super::types::RealtimePreviewStreamState;

impl RealtimePreviewStreamState {
    /// Non-realtime producer entry point. Returns frames accepted, which is
    /// fewer than offered when the ring is full.
    ///
    /// Never called from the audio thread.
    pub fn push_source(&mut self, interleaved: &[Sample]) -> usize {
        let channel_count = self.config.channel_count;
        let offered = interleaved.len() / channel_count;
        let in_flight = self
            .source_write_frame
            .saturating_sub(self.next_analysis_frame) as usize;
        let free = self.source_ring_frames.saturating_sub(in_flight);
        let accepted = offered.min(free);
        for frame in 0..accepted {
            let ring_frame = (self.source_write_frame as usize + frame) % self.source_ring_frames;
            for channel in 0..channel_count {
                self.source_ring[ring_frame * channel_count + channel] =
                    interleaved[frame * channel_count + channel];
            }
        }
        self.source_write_frame = self.source_write_frame.saturating_add(accepted as u64);
        accepted
    }

    pub(super) fn read_output(&mut self, output: &mut [Sample], frame_count: usize) {
        let channel_count = self.config.channel_count;
        for frame_offset in 0..frame_count {
            let ring_frame =
                (self.output_read_frame as usize + frame_offset) % self.output_ring_frames;
            for channel in 0..channel_count {
                let ring_index = ring_frame * channel_count + channel;
                let output_index = frame_offset * channel_count + channel;
                let weight = self.normalization_ring[ring_index];
                output[output_index] = if weight > 1.0e-3 {
                    self.output_ring[ring_index] / weight
                } else {
                    0.0
                };
                self.output_ring[ring_index] = 0.0;
                self.normalization_ring[ring_index] = 0.0;
            }
        }
        self.output_read_frame = self.output_read_frame.saturating_add(frame_count as u64);
    }
}
