use super::super::contract::{
    build_realtime_preview_dynamic_source_projection_report,
    build_realtime_preview_source_projection_report, DynamicSourceProjectionRatios,
    RealtimePreviewCallbackProcessError, RealtimePreviewCallbackState,
    RealtimePreviewDynamicSourceProjectionReport, RealtimePreviewSourceProjectionReport,
};
use crate::{abs_diff_frames, align_to_next_grid, sanitize_ratio, usize_to_u64};

impl RealtimePreviewCallbackState {
    /// Advance callback-owned source projection state for one output quantum.
    pub fn advance_source_projection(
        &mut self,
        output_frames: usize,
        ratio: f64,
    ) -> Result<RealtimePreviewSourceProjectionReport, RealtimePreviewCallbackProcessError> {
        if output_frames > self.config.max_block_frames {
            return Err(
                RealtimePreviewCallbackProcessError::FrameCountExceedsConfig {
                    requested: output_frames,
                    max: self.config.max_block_frames,
                },
            );
        }

        let ratio = sanitize_ratio(ratio);
        let output_start_frame = self.source_projection_output_frame;
        let output_end_frame = output_start_frame.saturating_add(usize_to_u64(output_frames));
        let source_start_frame = self.source_projection_source_cursor;
        let source_end_frame = source_start_frame + output_frames as f64 / ratio;
        let projection = build_realtime_preview_source_projection_report(
            ratio,
            output_start_frame,
            output_frames,
            output_end_frame,
            source_start_frame,
            source_end_frame,
        );
        self.source_projection_output_frame = output_end_frame;
        self.source_projection_source_cursor = source_end_frame;
        self.last_source_projection = projection;
        Ok(projection)
    }

    /// Advance scheduled source projection state for one output quantum.
    pub fn advance_scheduled_source_projection(
        &mut self,
        output_frames: usize,
        ratio: f64,
    ) -> Result<RealtimePreviewDynamicSourceProjectionReport, RealtimePreviewCallbackProcessError>
    {
        if output_frames > self.config.max_block_frames {
            return Err(
                RealtimePreviewCallbackProcessError::FrameCountExceedsConfig {
                    requested: output_frames,
                    max: self.config.max_block_frames,
                },
            );
        }

        let ratio = sanitize_ratio(ratio);
        self.schedule_source_projection_ratio_change(ratio);

        let output_start_frame = self.source_projection_output_frame;
        let output_end_frame = output_start_frame.saturating_add(usize_to_u64(output_frames));
        let source_start_frame = self.source_projection_source_cursor;
        let start_ratio = self.source_projection_active_ratio;
        let mut source_end_frame = source_start_frame;
        let mut active_ratio = self.source_projection_active_ratio;
        let mut ratio_change_applied = false;

        if self.source_projection_pending_ratio_change
            && self.source_projection_pending_ratio_apply_frame <= output_start_frame
        {
            self.apply_source_projection_ratio_change(output_start_frame, source_end_frame);
            active_ratio = self.source_projection_active_ratio;
            ratio_change_applied = true;
        }

        if self.source_projection_pending_ratio_change
            && self.source_projection_pending_ratio_apply_frame < output_end_frame
        {
            let ratio_change_output_frame = self.source_projection_pending_ratio_apply_frame;
            let frames_before_change =
                abs_diff_frames(ratio_change_output_frame, output_start_frame);
            source_end_frame += frames_before_change as f64 / active_ratio;
            self.apply_source_projection_ratio_change(ratio_change_output_frame, source_end_frame);
            active_ratio = self.source_projection_active_ratio;
            ratio_change_applied = true;

            let frames_after_change = abs_diff_frames(output_end_frame, ratio_change_output_frame);
            source_end_frame += frames_after_change as f64 / active_ratio;
        } else {
            source_end_frame += output_frames as f64 / active_ratio;
        }

        let projection = build_realtime_preview_dynamic_source_projection_report(
            output_start_frame,
            output_frames,
            output_end_frame,
            source_start_frame,
            source_end_frame,
            DynamicSourceProjectionRatios {
                start_ratio,
                end_ratio: active_ratio,
                ratio_change_applied,
                ratio_change_count: self.source_projection_ratio_change_count,
                ratio_change_request_output_frame: self
                    .last_source_projection_ratio_change_request_frame,
                ratio_change_output_frame: self.last_source_projection_ratio_change_output_frame,
                ratio_change_source_frame: self.last_source_projection_ratio_change_source_frame,
                ratio_change_alignment_error_frames: self
                    .last_source_projection_ratio_change_alignment_error_frames,
            },
        );

        self.source_projection_output_frame = output_end_frame;
        self.source_projection_source_cursor = source_end_frame;
        self.last_dynamic_source_projection = projection;
        Ok(projection)
    }

    fn schedule_source_projection_ratio_change(&mut self, ratio: f64) {
        if (ratio - self.source_projection_current_ratio).abs() <= f64::EPSILON {
            return;
        }
        self.source_projection_current_ratio = ratio;
        self.source_projection_pending_ratio = ratio;
        self.source_projection_pending_ratio_request_frame = self.source_projection_output_frame;
        self.source_projection_pending_ratio_apply_frame = align_to_next_grid(
            self.source_projection_output_frame,
            self.config.analysis_hop as u64,
        );
        self.source_projection_pending_ratio_change = true;
    }

    fn apply_source_projection_ratio_change(&mut self, output_frame: u64, source_frame: f64) {
        self.source_projection_active_ratio = self.source_projection_pending_ratio;
        self.last_source_projection_ratio_change_request_frame =
            self.source_projection_pending_ratio_request_frame;
        self.last_source_projection_ratio_change_output_frame = output_frame;
        self.last_source_projection_ratio_change_source_frame = source_frame;
        self.last_source_projection_ratio_change_alignment_error_frames = abs_diff_frames(
            output_frame,
            self.source_projection_pending_ratio_request_frame,
        );
        self.source_projection_pending_ratio_change = false;
        self.source_projection_ratio_change_count =
            self.source_projection_ratio_change_count.saturating_add(1);
    }
}
