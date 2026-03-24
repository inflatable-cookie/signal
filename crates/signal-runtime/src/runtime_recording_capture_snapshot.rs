use super::*;

impl RuntimeRecordingCaptureStateModel {
    pub(crate) fn capture_ready(&self, configured: bool, readiness: &RuntimeReadiness) -> bool {
        configured
            && !matches!(
                readiness,
                RuntimeReadiness::Stopped | RuntimeReadiness::Failed { .. }
            )
    }

    pub(crate) fn snapshot(
        &self,
        configured: bool,
        readiness: &RuntimeReadiness,
    ) -> RuntimeRecordingCaptureSnapshot {
        let active_checkpoint = self
            .active
            .as_ref()
            .map(|active| self.active_checkpoint(active, readiness));
        let state = if self.last_error.is_some() {
            Some(RuntimeRecordingCaptureState::Failed)
        } else if self.active.is_some() {
            Some(RuntimeRecordingCaptureState::Capturing)
        } else {
            Some(RuntimeRecordingCaptureState::Idle)
        };
        let summary = if let Some(checkpoint) = active_checkpoint.as_ref() {
            format!(
                "state=capturing ready={} kind={:?} checkpoint={:?}/{:?} take={} track={} frames={} events={} blocks={} pressure={} path={}",
                self.capture_ready(configured, readiness),
                checkpoint.capture_kind,
                checkpoint.checkpoint_class,
                checkpoint.interruption_class,
                checkpoint.take_id,
                checkpoint.track_id,
                checkpoint.buffered_frame_count,
                checkpoint.buffered_event_count,
                checkpoint.buffered_block_count,
                checkpoint.pressure_event_count,
                checkpoint.capture_path
            )
        } else {
            format!(
                "state={} ready={} last_take={} last_path={} duration={} last_checkpoint={:?}/{:?} error={}",
                if self.last_error.is_some() {
                    "failed"
                } else {
                    "idle"
                },
                self.capture_ready(configured, readiness),
                self.last_committed_take_id.as_deref().unwrap_or("none"),
                self.last_committed_path.as_deref().unwrap_or("none"),
                self.last_committed_duration_samples.unwrap_or(0),
                self.last_checkpoint.as_ref().map(|checkpoint| checkpoint.checkpoint_class),
                self.last_checkpoint
                    .as_ref()
                    .map(|checkpoint| checkpoint.interruption_class),
                self.last_error.as_deref().unwrap_or("none"),
            )
        };

        RuntimeRecordingCaptureSnapshot {
            capture_ready: self.capture_ready(configured, readiness),
            state,
            capture_kind: self
                .active
                .as_ref()
                .map(|active| active.capture_kind)
                .or_else(|| {
                    self.last_checkpoint
                        .as_ref()
                        .map(|checkpoint| checkpoint.capture_kind)
                }),
            active_take_id: self.active.as_ref().map(|active| active.take_id.clone()),
            active_track_id: self.active.as_ref().map(|active| active.track_id.clone()),
            capture_start_samples: self.active.as_ref().map(|active| active.start_samples),
            active_capture_path: self
                .active
                .as_ref()
                .map(|active| active.capture_path.clone()),
            buffered_block_count: self
                .active
                .as_ref()
                .map(|active| active.buffered_block_count)
                .unwrap_or(0),
            buffered_frame_count: self
                .active
                .as_ref()
                .map(|active| active.buffered_frame_count)
                .unwrap_or(0),
            buffered_event_count: self
                .active
                .as_ref()
                .map(|active| active.buffered_event_count)
                .unwrap_or(0),
            captured_channel_count: self
                .active
                .as_ref()
                .map(|active| active.channel_count)
                .unwrap_or(0),
            peak_level: self.active.as_ref().map(|active| active.peak_level),
            pressure_event_count: self
                .active
                .as_ref()
                .map(|active| active.pressure_event_count)
                .unwrap_or(0),
            active_checkpoint,
            last_checkpoint: self.last_checkpoint.clone(),
            last_committed_take_id: self.last_committed_take_id.clone(),
            last_committed_path: self.last_committed_path.clone(),
            last_committed_duration_samples: self.last_committed_duration_samples,
            last_error: self.last_error.clone(),
            summary,
        }
    }

    pub(crate) fn active_checkpoint(
        &self,
        active: &RuntimeRecordingCaptureActiveSession,
        readiness: &RuntimeReadiness,
    ) -> RuntimeRecordingCaptureCheckpointSnapshot {
        let checkpoint_class = if self.last_error.is_some() {
            RuntimeRecordingCaptureCheckpointClass::Failed
        } else if active.buffered_frame_count > 0 || active.buffered_event_count > 0 {
            RuntimeRecordingCaptureCheckpointClass::Streaming
        } else {
            RuntimeRecordingCaptureCheckpointClass::Armed
        };
        let interruption_class = if self.last_error.is_some() {
            RuntimeInterruptionClass::Terminal
        } else if matches!(readiness, RuntimeReadiness::Degraded { .. }) {
            RuntimeInterruptionClass::Resumable
        } else {
            RuntimeInterruptionClass::Steady
        };
        self.checkpoint_from_active(
            active,
            checkpoint_class,
            interruption_class,
            self.last_error.clone(),
            if self.last_error.is_some() {
                "active capture failed"
            } else {
                "active capture checkpoint"
            },
        )
    }

    pub(crate) fn checkpoint_from_active(
        &self,
        active: &RuntimeRecordingCaptureActiveSession,
        checkpoint_class: RuntimeRecordingCaptureCheckpointClass,
        interruption_class: RuntimeInterruptionClass,
        last_error: Option<String>,
        reason: &str,
    ) -> RuntimeRecordingCaptureCheckpointSnapshot {
        RuntimeRecordingCaptureCheckpointSnapshot {
            capture_kind: active.capture_kind,
            checkpoint_class,
            interruption_class,
            take_id: active.take_id.clone(),
            track_id: active.track_id.clone(),
            capture_start_samples: active.start_samples,
            capture_path: active.capture_path.clone(),
            buffered_block_count: active.buffered_block_count,
            buffered_frame_count: active.buffered_frame_count,
            buffered_event_count: active.buffered_event_count,
            captured_channel_count: active.channel_count,
            peak_level: (active.channel_count > 0).then_some(active.peak_level),
            pressure_event_count: active.pressure_event_count,
            last_error,
            summary: format!(
                "kind={:?} checkpoint={:?} interruption={:?} take={} track={} frames={} events={} blocks={} pressure={} reason={} path={}",
                active.capture_kind,
                checkpoint_class,
                interruption_class,
                active.take_id,
                active.track_id,
                active.buffered_frame_count,
                active.buffered_event_count,
                active.buffered_block_count,
                active.pressure_event_count,
                reason,
                active.capture_path,
            ),
        }
    }
}
