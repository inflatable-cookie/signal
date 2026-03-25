use super::*;

impl SignalRuntime {
    pub(crate) fn transport_observation_snapshot(&self) -> RuntimeTransportObservationSnapshot {
        let timeline = self.timeline.snapshot();
        RuntimeTransportObservationSnapshot {
            transport_epoch: timeline.transport_epoch,
            projected_playing: self.applied_transport.map(|transport| transport.playing),
            projected_tempo_bpm: self.applied_transport.map(|transport| transport.tempo_bpm),
            projected_timeline_position_samples: self
                .applied_transport
                .map(|transport| transport.timeline_position_samples),
            projected_loop_start_samples: self.applied_transport.and_then(|transport| {
                transport
                    .loop_state
                    .map(|loop_state| loop_state.start_samples)
            }),
            projected_loop_end_samples: self.applied_transport.and_then(|transport| {
                transport
                    .loop_state
                    .map(|loop_state| loop_state.end_samples)
            }),
            observed_playing: timeline.last_transport_playing,
            observed_tempo_bpm: timeline.last_transport_tempo_bpm,
            observed_timeline_position_samples: timeline.last_transport_timeline_position_samples,
            observed_loop_start_samples: timeline.last_transport_loop_start_samples,
            observed_loop_end_samples: timeline.last_transport_loop_end_samples,
            last_transition: timeline.last_transport_transition,
            last_transition_processing_epoch: timeline.last_transport_transition_processing_epoch,
            last_transition_block_sequence: timeline.last_transport_transition_block_sequence,
            last_engine_block_start_samples: timeline.last_engine_block_start_samples,
            last_engine_block_end_samples: timeline.last_engine_block_end_samples,
            loop_wrap_count: timeline.loop_wrap_count,
        }
    }

    pub(crate) fn recording_capture_snapshot(&self) -> RuntimeRecordingCaptureSnapshot {
        self.recording_capture
            .snapshot(self.control.configured, &self.readiness)
    }

    pub(crate) fn media_pipeline_snapshot(&self) -> RuntimeMediaPipelineSnapshot {
        self.media_pipeline.snapshot()
    }

    pub(crate) fn media_service_snapshot(&self) -> RuntimeMediaServiceSnapshot {
        self.media_pipeline.service_snapshot()
    }

    pub(crate) fn media_library_service_snapshot(&self) -> RuntimeMediaLibraryServiceSnapshot {
        self.media_pipeline.library_service_snapshot()
    }

    pub(crate) fn metering_snapshot(&self) -> RuntimeMeteringSnapshot {
        self.metering
            .snapshot()
            .with_execution_topology(&self.execution_topology_summary())
    }
}
