use super::*;

#[test]
fn runtime_offline_render_contract_preview_reuses_runtime_topology_tempo_clip_and_recall_contracts()
{
    let (preview, selection) = build_runtime_offline_render_contract_preview();
    assert_eq!(preview.request_id, "render:preview");
    assert_eq!(preview.export_sample_rate_hz, 48_000);
    assert_eq!(preview.clip_count, 1);
    assert_eq!(preview.ready_clip_count, 1);
    assert_eq!(preview.stem_count, 1);
    assert_eq!(preview.freeze_artifact_count, 1);
    assert_eq!(preview.resolved_tempo_bpm, 132.0);
    assert_eq!(
        preview.resolved_tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );
    assert_eq!(preview.stem_targets[0].stem_id, "stem:track:lead");
    assert_eq!(
        preview.stem_targets[0].target_kind,
        RuntimeOfflineRenderTargetKind::TrackLane
    );
    assert_eq!(
        preview.stem_targets[0].target_id.as_deref(),
        Some("track:lead")
    );
    assert_eq!(
        preview.stem_targets[0].resolved_node_ids,
        vec!["plugin-a".to_string(), "plugin-b".to_string()]
    );
    assert_eq!(preview.freeze_artifacts[0].artifact_id, "freeze:track:lead");
    assert_eq!(preview.freeze_artifacts[0].recall_stage_count, 2);
    assert_eq!(
        preview.freeze_artifacts[0].recall_stage_ids,
        selection.stage_ids
    );
    assert_eq!(
        preview.freeze_artifacts[0].recall_states,
        vec![
            RuntimePluginRecallState::Warm,
            RuntimePluginRecallState::Recovered
        ]
    );
    assert_eq!(preview.chain_contract.chain_count, 1);
    assert_eq!(preview.chain_contract.stage_count, 2);
    assert_eq!(preview.chain_contract.pending_render_stage_count, 2);
    assert_eq!(preview.chain_contract.settling_stage_count, 0);
    assert_eq!(preview.chain_contract.compensated_stage_count, 0);
    assert_eq!(preview.chain_contract.total_planned_latency_samples, 36);
    assert_eq!(preview.chain_contract.total_realized_latency_samples, 0);
    assert_eq!(preview.chain_contract.total_tail_samples, 0);
    assert_eq!(preview.chain_contract.complex_io_stage_count, 2);
    assert_eq!(
        preview.chain_contract.multi_output_instrument_stage_count,
        1
    );
    assert_eq!(preview.chain_contract.bus_capable_fx_stage_count, 1);
    assert_eq!(preview.chain_contract.sidechain_capable_fx_stage_count, 1);
    assert_eq!(preview.chain_contract.recall_stage_count, 2);
    assert_eq!(preview.chain_contract.warm_recall_stage_count, 1);
    assert_eq!(preview.chain_contract.recovered_recall_stage_count, 1);
    assert_eq!(preview.chain_contract.cold_recall_stage_count, 0);
    assert_eq!(preview.chain_contract.unavailable_recall_stage_count, 0);
    assert_eq!(preview.chain_contract.complex_io_stages.len(), 2);
    assert_eq!(
        preview.chain_contract.complex_io_stages[0].plugin_type_id,
        Some("plugin:vst3:multiout-instrument".to_string())
    );
    assert!(
        preview.chain_contract.complex_io_stages[0]
            .topology
            .multi_output_instrument
    );
    assert_eq!(
        preview.chain_contract.complex_io_stages[0]
            .topology
            .instrument_output_group_count,
        2
    );
    assert_eq!(
        preview.chain_contract.complex_io_stages[1]
            .topology
            .bus_capable_fx_class,
        Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
    );
    assert!(preview.chain_contract.summary.contains("pending=2"));
    assert!(preview
        .chain_contract
        .summary
        .contains("complex_io_stages=2"));
    assert!(preview.chain_contract.summary.contains("recall=2/"));
    assert!(preview.summary.contains("stems=1"));
    assert!(preview.summary.contains("freeze_artifacts=1"));
    assert!(preview.summary.contains("chain_contract=chains=1"));
}
