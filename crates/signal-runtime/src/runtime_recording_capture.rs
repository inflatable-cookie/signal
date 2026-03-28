use super::runtime_audio_file_io::commit_recording_capture_wav;
use super::*;
#[path = "runtime_recording_capture_snapshot.rs"]
mod runtime_recording_capture_snapshot;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RuntimeRecordingCaptureActiveSession {
    pub(super) capture_kind: RuntimeRecordingCaptureKind,
    pub(super) take_id: String,
    pub(super) track_id: String,
    pub(super) start_samples: i64,
    pub(super) capture_path: String,
    pub(super) sample_rate_hz: u32,
    pub(super) channel_count: usize,
    pub(super) samples: Vec<f32>,
    pub(super) buffered_block_count: u64,
    pub(super) buffered_frame_count: u64,
    pub(super) buffered_event_count: u64,
    pub(super) peak_level: f32,
    pub(super) pressure_event_count: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct RuntimeRecordingCaptureStateModel {
    policy: RuntimeRecordingCapturePolicy,
    active: Option<RuntimeRecordingCaptureActiveSession>,
    last_committed_take_id: Option<String>,
    last_committed_path: Option<String>,
    last_committed_duration_samples: Option<u32>,
    last_checkpoint: Option<RuntimeRecordingCaptureCheckpointSnapshot>,
    last_error: Option<String>,
}

impl RuntimeRecordingCaptureStateModel {
    pub(super) fn interrupt_active_capture(
        &mut self,
        interruption_class: RuntimeInterruptionClass,
        reason: &str,
    ) {
        let Some(active) = self.active.as_ref() else {
            return;
        };
        let checkpoint_class = if interruption_class == RuntimeInterruptionClass::Terminal {
            RuntimeRecordingCaptureCheckpointClass::Failed
        } else if active.buffered_frame_count > 0 || active.buffered_event_count > 0 {
            RuntimeRecordingCaptureCheckpointClass::Buffered
        } else {
            RuntimeRecordingCaptureCheckpointClass::Armed
        };
        self.last_checkpoint = Some(self.checkpoint_from_active(
            active,
            checkpoint_class,
            interruption_class,
            self.last_error.clone(),
            reason,
        ));
        self.active = None;
    }

    pub(super) fn reset_for_runtime_reconfigure(&mut self) {
        self.policy = RuntimeRecordingCapturePolicy::default();
        self.active = None;
        self.last_error = None;
    }

    pub(super) fn start_capture(
        &mut self,
        request: RuntimeRecordingCaptureStartRequest,
        sample_rate_hz: u32,
        configured: bool,
        readiness: &RuntimeReadiness,
    ) -> Result<(), RuntimeError> {
        if !self.capture_ready(configured, readiness) {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime is not ready to begin recording capture",
            ));
        }
        if self.active.is_some() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "recording capture is already active",
            ));
        }
        self.last_error = None;
        self.active = Some(RuntimeRecordingCaptureActiveSession {
            capture_kind: request.capture_kind,
            take_id: request.take_id,
            track_id: request.track_id,
            start_samples: request.start_samples,
            capture_path: request.capture_path,
            sample_rate_hz,
            channel_count: 0,
            samples: Vec::new(),
            buffered_block_count: 0,
            buffered_frame_count: 0,
            buffered_event_count: 0,
            peak_level: 0.0,
            pressure_event_count: 0,
        });
        Ok(())
    }

    pub(super) fn record_output_block(&mut self, output: &AudioBuffer) {
        let Some(active) = self.active.as_mut() else {
            return;
        };

        let channel_count = output.channel_count().0;
        let frame_count = output.frames().0 as u64;
        if active.channel_count == 0 {
            active.channel_count = channel_count;
        } else if active.channel_count != channel_count {
            self.last_error = Some(format!(
                "capture channel-count mismatch: expected {} got {}",
                active.channel_count, channel_count
            ));
            self.interrupt_active_capture(
                RuntimeInterruptionClass::Terminal,
                "capture channel-count mismatch",
            );
            return;
        }

        active.samples.extend_from_slice(output.samples());
        active.buffered_block_count = active.buffered_block_count.saturating_add(1);
        active.buffered_frame_count = active.buffered_frame_count.saturating_add(frame_count);
        let block_peak = output
            .samples()
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        active.peak_level = active.peak_level.max(block_peak);
        if active.buffered_frame_count >= self.policy.pressure_threshold_frames {
            active.pressure_event_count = active.pressure_event_count.saturating_add(1);
        }
    }

    pub(super) fn finish_capture(
        &mut self,
    ) -> Result<RuntimeRecordingCaptureCommitReceipt, RuntimeError> {
        let active = self.active.as_ref().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "recording capture is not active",
            )
        })?;
        if active.channel_count == 0 || active.samples.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "recording capture has no buffered audio to commit",
            ));
        }

        let committed_active = active.clone();
        let capture_path = committed_active.capture_path.clone();
        let duration_samples = match commit_recording_capture_wav(&committed_active) {
            Ok(duration_samples) => duration_samples,
            Err(error) => {
                self.last_error = Some(error.message.clone());
                self.interrupt_active_capture(
                    RuntimeInterruptionClass::Terminal,
                    "capture commit failed terminally",
                );
                return Err(error);
            }
        };

        let committed_checkpoint = self.checkpoint_from_active(
            &committed_active,
            RuntimeRecordingCaptureCheckpointClass::Committed,
            RuntimeInterruptionClass::Steady,
            None,
            "capture committed",
        );
        let receipt = RuntimeRecordingCaptureCommitReceipt {
            capture_kind: committed_active.capture_kind,
            take_id: committed_active.take_id.clone(),
            track_id: committed_active.track_id.clone(),
            start_samples: committed_active.start_samples,
            duration_samples,
            channel_count: committed_active.channel_count,
            peak_level: committed_active.peak_level,
            capture_path,
            committed_checkpoint,
        };

        self.last_error = None;
        self.last_committed_take_id = Some(receipt.take_id.clone());
        self.last_committed_path = Some(receipt.capture_path.clone());
        self.last_committed_duration_samples = Some(receipt.duration_samples);
        self.last_checkpoint = Some(receipt.committed_checkpoint.clone());
        self.active = None;
        Ok(receipt)
    }

    pub(super) fn cancel_capture(&mut self) -> Result<(), RuntimeError> {
        if self.active.is_none() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "recording capture is not active",
            ));
        }
        self.last_error = None;
        self.interrupt_active_capture(
            RuntimeInterruptionClass::Restartable,
            "capture cancelled before commit",
        );
        Ok(())
    }
}
