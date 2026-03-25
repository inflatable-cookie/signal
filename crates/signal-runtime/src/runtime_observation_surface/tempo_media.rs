use super::*;

impl SignalRuntime {
    pub(crate) fn current_resolved_tempo(&self) -> RuntimeResolvedTempo {
        self.tempo_map.resolve(
            self.applied_transport
                .map(|transport| transport.timeline_position_samples)
                .or(self.timeline.last_transport_timeline_position_samples),
            self.applied_transport,
            self.timeline.last_transport_tempo_bpm,
        )
    }

    pub(crate) fn tempo_map_snapshot(&self) -> RuntimeTempoMapSnapshot {
        let resolved_tempo = self.current_resolved_tempo();
        self.tempo_map.snapshot(&resolved_tempo)
    }

    pub(crate) fn warp_pipeline_snapshot(&self) -> RuntimeWarpPipelineSnapshot {
        let resolved_tempo = self.current_resolved_tempo();
        self.warp_pipeline
            .snapshot(&resolved_tempo, &self.media_pipeline)
    }

    pub(crate) fn clip_processing_pipeline_snapshot(
        &self,
    ) -> RuntimeClipProcessingPipelineSnapshot {
        let resolved_tempo = self.current_resolved_tempo();
        self.clip_processing_pipeline.snapshot(
            &self.media_pipeline,
            &self.warp_pipeline,
            &resolved_tempo,
        )
    }
}
