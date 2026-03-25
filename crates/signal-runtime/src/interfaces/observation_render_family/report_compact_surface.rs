use super::*;

pub(crate) struct RuntimeObservationCompactSections {
    pub runtime_surface_summaries: String,
    pub execution_topology_summary: String,
    pub metering_summary: String,
    pub recording_capture: String,
    pub marker_analysis: String,
    pub transform_artifact: String,
    pub media_pipeline: String,
    pub media_service: String,
    pub offline_render_session: String,
    pub deferred_service: String,
    pub engine_transport: String,
    pub scheduler_topology: String,
    pub linux_backend_session_summary: String,
    pub pipewire_alsa_parity_summary: String,
    pub jack_coordination_summary: String,
}

pub(crate) fn build_runtime_observation_report_compact_sections(
    report: &RuntimeObservationReport,
) -> RuntimeObservationCompactSections {
    let tempo_map = (report.tempo_map_snapshot.segment_count > 0)
        .then(|| format_runtime_tempo_map_snapshot_compact(&report.tempo_map_snapshot))
        .unwrap_or_default();
    let warp = (report.warp_pipeline_snapshot.clip_count > 0)
        .then(|| format_runtime_warp_pipeline_snapshot_compact(&report.warp_pipeline_snapshot))
        .unwrap_or_default();
    let clip_processing = (report.clip_processing_pipeline_snapshot.clip_count > 0)
        .then(|| {
            format_runtime_clip_processing_pipeline_snapshot_compact(
                &report.clip_processing_pipeline_snapshot,
            )
        })
        .unwrap_or_default();
    let stretch_engine = (report.stretch_engine_snapshot.clip_count > 0)
        .then(|| format_runtime_stretch_engine_snapshot_compact(&report.stretch_engine_snapshot))
        .unwrap_or_default();
    let marker_analysis = (report.marker_analysis_snapshot.clip_count > 0)
        .then(|| format_runtime_marker_analysis_snapshot_compact(&report.marker_analysis_snapshot))
        .unwrap_or_default();
    let transform_artifact = (report.transform_artifact_snapshot.clip_count > 0)
        .then(|| {
            format_runtime_transform_artifact_snapshot_compact(&report.transform_artifact_snapshot)
        })
        .unwrap_or_default();
    let media_pipeline = (report.media_pipeline_snapshot.asset_count > 0)
        .then(|| format_runtime_media_pipeline_snapshot_compact(&report.media_pipeline_snapshot))
        .unwrap_or_default();
    let media_service = (report.media_service_snapshot.indexed_asset_count > 0
        || report.media_service_snapshot.invalidation_active
        || matches!(
            report.media_service_snapshot.preview_state,
            RuntimeMediaPreviewState::Previewing | RuntimeMediaPreviewState::Invalidated
        ))
    .then(|| format_runtime_media_service_snapshot_compact(&report.media_service_snapshot))
    .unwrap_or_default();
    let media_library = (report.media_library_snapshot.indexed_asset_count > 0)
        .then(|| {
            format_runtime_media_library_service_snapshot_compact(&report.media_library_snapshot)
        })
        .unwrap_or_default();
    let plugin_discovery = (report.plugin_discovery_snapshot.scan_count > 0)
        .then(|| {
            format_runtime_plugin_discovery_snapshot_compact(&report.plugin_discovery_snapshot)
        })
        .unwrap_or_default();
    let plugin_lifecycle = (report.plugin_lifecycle_snapshot.sandbox_count > 0)
        .then(|| {
            format_runtime_plugin_lifecycle_snapshot_compact(&report.plugin_lifecycle_snapshot)
        })
        .unwrap_or_default();
    let lv2_extension = (report.lv2_extension_snapshot.plugin_type_count > 0)
        .then(|| format_runtime_lv2_extension_snapshot_compact(&report.lv2_extension_snapshot))
        .unwrap_or_default();
    let plugin_pin_matrix = (report.plugin_pin_matrix_snapshot.plugin_type_count > 0)
        .then(|| {
            format_runtime_plugin_pin_matrix_snapshot_compact(&report.plugin_pin_matrix_snapshot)
        })
        .unwrap_or_default();
    let plugin_chain = (report.plugin_chain_snapshot.chain_count > 0)
        .then(|| format_runtime_plugin_chain_snapshot_compact(&report.plugin_chain_snapshot))
        .unwrap_or_default();
    let automation = (report.automation_snapshot.parameter_id != 0
        || report.automation_snapshot.lane_count > 0
        || report.automation_snapshot.last_batch_epoch.is_some())
    .then(|| format_runtime_automation_snapshot_compact(&report.automation_snapshot))
    .unwrap_or_default();
    let plugin_events = (report.plugin_event_snapshot.total_events > 0
        || report.plugin_event_snapshot.last_processing_epoch.is_some())
    .then(|| format_runtime_plugin_event_snapshot_compact(&report.plugin_event_snapshot))
    .unwrap_or_default();
    let transport_timeline = format_runtime_transport_timeline_compact(&report.timeline_snapshot);
    let scheduler_snapshot = format_runtime_scheduler_snapshot_compact(&report.scheduler_snapshot);
    let scheduler_summary = format_runtime_scheduler_summary_compact(&report.scheduler_summary);
    let block_summary = format_runtime_block_summary_compact(&report.block_summary);
    let degradation_summary =
        format_runtime_degradation_summary_compact(&report.degradation_summary);
    let fault_status = format_runtime_fault_status_compact(&report.fault_status);
    let fault_diagnostic_receipt =
        format_runtime_fault_diagnostic_receipt_compact(&report.fault_diagnostic_receipt);
    let interruption_summary =
        format_runtime_interruption_summary_compact(&report.interruption_summary);
    let device_supervision_summary =
        format_runtime_device_supervision_snapshot_compact(&report.device_supervision_snapshot);
    let external_io_summary =
        format_runtime_external_io_snapshot_compact(&report.external_io_snapshot);
    let linux_backend_session_summary = format_runtime_linux_backend_session_snapshot_compact(
        &report.linux_backend_session_snapshot,
    );
    let pipewire_alsa_parity_summary =
        format_runtime_pipewire_alsa_parity_snapshot_compact(&report.pipewire_alsa_parity_snapshot);
    let jack_coordination_summary =
        format_runtime_jack_coordination_snapshot_compact(&report.jack_coordination_snapshot);
    let external_midi_summary =
        format_runtime_external_midi_snapshot_compact(&report.external_midi_snapshot);

    RuntimeObservationCompactSections {
        runtime_surface_summaries: format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
            tempo_map,
            warp,
            clip_processing,
            stretch_engine,
            media_pipeline,
            media_service,
            media_library,
            plugin_discovery,
            plugin_lifecycle,
            lv2_extension,
            plugin_pin_matrix,
            plugin_chain,
            automation,
            plugin_events,
            transport_timeline,
            scheduler_snapshot,
            scheduler_summary,
            block_summary,
            degradation_summary,
            fault_status,
            fault_diagnostic_receipt,
            interruption_summary,
            device_supervision_summary,
            external_io_summary,
            linux_backend_session_summary,
            pipewire_alsa_parity_summary,
            jack_coordination_summary,
            external_midi_summary,
        ),
        execution_topology_summary: format_runtime_execution_topology_summary_compact(
            &report.execution_topology_summary,
        ),
        metering_summary: format_runtime_metering_snapshot_compact(&report.metering_snapshot),
        recording_capture: format_runtime_recording_capture_snapshot_compact(
            &report.recording_capture_snapshot,
        ),
        marker_analysis,
        transform_artifact,
        media_pipeline,
        media_service,
        offline_render_session: (report.offline_render_session_snapshot.active_session_count > 0
            || report
                .offline_render_session_snapshot
                .last_session
                .is_some()
            || report
                .offline_render_session_snapshot
                .last_cancellation
                .is_some()
            || report.offline_render_session_snapshot.last_purge.is_some())
        .then(|| {
            format_runtime_offline_render_session_snapshot_compact(
                &report.offline_render_session_snapshot,
            )
        })
        .unwrap_or_default(),
        deferred_service: report
            .last_deferred_service_receipt
            .as_ref()
            .map(format_runtime_deferred_service_receipt_compact)
            .unwrap_or_default(),
        engine_transport: format_runtime_engine_transport_compact(&report.engine_block_snapshot),
        scheduler_topology: format_scheduler_topology_compact(
            &report.engine_block_snapshot.scheduler_topology,
        ),
        linux_backend_session_summary,
        pipewire_alsa_parity_summary,
        jack_coordination_summary,
    }
}
