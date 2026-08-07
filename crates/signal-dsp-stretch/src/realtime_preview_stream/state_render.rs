use signal_primitives::Sample;

use crate::align_to_next_grid;

use super::constants::{REALTIME_PREVIEW_STREAM_MAX_RATIO, REALTIME_PREVIEW_STREAM_MIN_RATIO};
use super::types::{
    RealtimePreviewStreamError, RealtimePreviewStreamRenderReport, RealtimePreviewStreamState,
};

impl RealtimePreviewStreamState {
    /// Audio-callback entry point: produce `frame_count` output frames,
    /// consuming however much source `ratio` demands.
    ///
    /// Allocation-free, lock-free, and I/O-free. Unlike the kernel it replaces
    /// it takes no input slice: source arrives through [`Self::push_source`].
    pub fn render(
        &mut self,
        output: &mut [Sample],
        frame_count: usize,
        ratio: f64,
    ) -> Result<RealtimePreviewStreamRenderReport, RealtimePreviewStreamError> {
        if frame_count > self.config.max_block_frames {
            return Err(RealtimePreviewStreamError::FrameCountExceedsConfig {
                requested: frame_count,
                max: self.config.max_block_frames,
            });
        }
        let required_samples = frame_count * self.config.channel_count;
        if output.len() < required_samples {
            return Err(RealtimePreviewStreamError::OutputTooSmall {
                required_samples,
                output_samples: output.len(),
            });
        }
        // Rejected, not clamped. A silently clamped ratio would make the
        // reported and actual source advance disagree, which is the class of
        // defect this lane exists to remove.
        if !ratio.is_finite()
            || !(REALTIME_PREVIEW_STREAM_MIN_RATIO..=REALTIME_PREVIEW_STREAM_MAX_RATIO)
                .contains(&ratio)
        {
            return Err(RealtimePreviewStreamError::RatioOutOfRange {
                requested: ratio,
                min: REALTIME_PREVIEW_STREAM_MIN_RATIO,
                max: REALTIME_PREVIEW_STREAM_MAX_RATIO,
            });
        }

        self.schedule_ratio_change(ratio);

        let analysis_start = self.next_analysis_frame;
        let target_output_frame = self.output_read_frame + frame_count as u64;
        let mut spectral_frames = 0usize;

        while self.next_synthesis_frame < target_output_frame as f64 {
            let window_end = self.next_analysis_frame + self.config.window_size as u64;
            if self.source_write_frame < window_end {
                // Underrun. Stop rather than advancing past source the producer
                // has not delivered: skipping is the defect this replaces.
                break;
            }
            let synthesis_start = self.next_synthesis_frame.round() as u64;
            self.apply_ratio_change_if_due(synthesis_start);
            let active = self.active_ratio;
            for channel in 0..self.config.channel_count {
                self.analyze(channel);
                self.propagate_phase(channel, active);
                self.synthesize(channel, synthesis_start);
            }
            self.next_analysis_frame = self
                .next_analysis_frame
                .saturating_add(self.config.analysis_hop as u64);
            self.next_synthesis_frame += self.config.analysis_hop as f64 * active;
            self.spectral_frame_index = self.spectral_frame_index.saturating_add(1);
            spectral_frames += 1;
        }

        // Frames past the synthesis frontier were never accumulated, so their
        // normalization weight is zero and `read_output` emits silence. The
        // count is arithmetic, not a second source of truth.
        let covered = (self.next_synthesis_frame.floor() as u64).min(target_output_frame);
        let underrun_frames = target_output_frame.saturating_sub(covered) as usize;

        self.read_output(output, frame_count);

        let consumed = self.next_analysis_frame.saturating_sub(analysis_start);
        self.total_source_frames_consumed =
            self.total_source_frames_consumed.saturating_add(consumed);

        Ok(RealtimePreviewStreamRenderReport {
            output_frames: frame_count,
            underrun_frames,
            source_frames_consumed: consumed,
            total_source_frames_consumed: self.total_source_frames_consumed,
            spectral_frames,
            requested_ratio: ratio,
            active_ratio: self.active_ratio,
            ratio_change_count: self.ratio_change_count,
            ratio_change_alignment_error_frames: self.last_alignment_error_frames,
            source_demand_frame: self.source_demand_frame(),
        })
    }

    pub(super) fn schedule_ratio_change(&mut self, ratio: f64) {
        if (ratio - self.current_ratio).abs() <= f64::EPSILON {
            return;
        }
        self.current_ratio = ratio;
        self.pending_ratio = ratio;
        self.pending_request_frame = self.output_read_frame;
        self.pending_apply_frame =
            align_to_next_grid(self.output_read_frame, self.config.analysis_hop as u64);
        self.pending_change = true;
    }

    pub(super) fn apply_ratio_change_if_due(&mut self, synthesis_start: u64) {
        if !self.pending_change || synthesis_start < self.pending_apply_frame {
            return;
        }
        self.active_ratio = self.pending_ratio;
        // Bounded by `analysis_hop` by construction: changes align to the hop
        // grid, so the request and application cannot be further apart.
        self.last_alignment_error_frames = synthesis_start.abs_diff(self.pending_request_frame);
        self.pending_change = false;
        self.ratio_change_count = self.ratio_change_count.saturating_add(1);
    }
}
