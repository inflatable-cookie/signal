use super::*;
use crate::runtime::runtime_plugin_recording::runtime_plugin_parity_coverage;

impl RuntimeObservationApi for SignalRuntime {
    fn subscribe(&mut self, sink: Box<dyn RuntimeEventSink>) -> SubscriptionHandle {
        let handle = SubscriptionHandle(self.next_subscription);
        self.next_subscription = self.next_subscription.saturating_add(1);
        self.sinks.push(sink);
        handle
    }

    fn get_readiness(&self) -> RuntimeReadiness {
        self.readiness.clone()
    }

    fn get_effective_config(&self) -> EffectiveRuntimeConfig {
        EffectiveRuntimeConfig {
            sample_rate: self.config.sample_rate,
            block_size: self.config.graph.block_size,
            anticipative_enabled: self.anticipative_enabled,
            safe_mode_enabled: self.safe_mode_enabled,
            active_output_device: self.active_output_device.clone(),
        }
    }

    fn get_control_snapshot(&self) -> RuntimeControlSnapshot {
        self.control.clone()
    }

    fn get_diagnostics_snapshot(&self) -> RuntimeDiagnosticsSnapshot {
        self.diagnostics_snapshot()
    }

    fn get_supervision_snapshot(&self) -> RuntimeSupervisionSnapshot {
        self.supervision.snapshot(self.safe_mode_enabled)
    }

    fn get_recording_capture_snapshot(&self) -> RuntimeRecordingCaptureSnapshot {
        self.recording_capture_snapshot()
    }

    fn get_media_pipeline_snapshot(&self) -> RuntimeMediaPipelineSnapshot {
        self.media_pipeline_snapshot()
    }

    fn get_media_service_snapshot(&self) -> RuntimeMediaServiceSnapshot {
        self.media_service_snapshot()
    }

    fn get_media_library_service_snapshot(&self) -> RuntimeMediaLibraryServiceSnapshot {
        self.media_library_service_snapshot()
    }

    fn get_offline_stretch_artifact_plan_snapshot(
        &self,
    ) -> RuntimeOfflineStretchArtifactPlanSnapshotSet {
        self.offline_stretch_artifact_plan_snapshot()
    }

    fn get_tempo_map_snapshot(&self) -> RuntimeTempoMapSnapshot {
        self.tempo_map_snapshot()
    }

    fn get_warp_pipeline_snapshot(&self) -> RuntimeWarpPipelineSnapshot {
        self.warp_pipeline_snapshot()
    }

    fn get_clip_processing_pipeline_snapshot(&self) -> RuntimeClipProcessingPipelineSnapshot {
        self.clip_processing_pipeline_snapshot()
    }

    fn get_execution_topology_summary(&self) -> RuntimeExecutionTopologySummary {
        RuntimeExecutionTopologySummary::from_plan(&self.plan.lane_order, &self.plan.planned_nodes)
            .with_plugin_chain_snapshot(&self.plugin_chain_snapshot())
    }

    fn get_graph_latency_samples(&self) -> u32 {
        self.plan.total_latency_samples
    }

    fn get_plugin_discovery_snapshot(&self) -> RuntimePluginDiscoverySnapshot {
        let lifecycle = self.plugin_lifecycle_snapshot();
        let parity_coverage = runtime_plugin_parity_coverage(
            &self.plugin_discovery.discovered_types,
            &lifecycle.sandboxes,
            &self.plugin_placement_policy,
            &self.plugin_discovery.platform_coverage,
        );
        self.plugin_discovery.snapshot(parity_coverage)
    }

    fn get_plugin_lifecycle_snapshot(&self) -> RuntimePluginLifecycleSnapshot {
        self.plugin_lifecycle_snapshot()
    }

    fn get_plugin_chain_snapshot(&self) -> RuntimePluginChainSnapshot {
        self.plugin_chain_snapshot()
    }
}
