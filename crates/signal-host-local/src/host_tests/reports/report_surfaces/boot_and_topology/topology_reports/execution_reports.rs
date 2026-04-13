use super::super::super::super::super::*;

#[test]
fn local_host_executes_track_bus_output_topology_through_audio_pump() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host.boot_default().expect("default local host boot");
    let supervisor = host.supervisor_report();
    let topology = &supervisor.observation.execution_topology_summary;

    assert_eq!(summary.audio_pump.stream_state, LocalAudioStreamState::Running);
    assert_eq!(summary.audio_pump.callback_count, 8);
    assert_local_plugin_topology(&summary);
    assert_eq!(summary.topology, *topology);
    assert!(supervisor
        .render_multiline()
        .contains("execution_topology_summary_node_3=output-main"));
}

#[test]
fn local_host_shared_report_surfaces_execution_topology_and_plugin_reports() {
    let _demo_override = crate::host::ensure_default_demo_plugin_override();
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default().expect("default local host boot");
    let report = host.host_supervisor_report();

    assert_eq!(report.observation.observation.execution_topology_summary.node_count, 4);
    assert_eq!(
        report
            .observation
            .observation
            .execution_topology_summary
            .track_lane_node_count,
        2
    );
    assert_eq!(
        report
            .observation
            .observation
            .execution_topology_summary
            .bus_node_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .execution_topology_summary
            .console_node_count,
        1
    );
    assert_eq!(
        report.observation.observation.plugin_discovery_snapshot.scan_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .plugin_discovery_snapshot
            .format_filtered_scan_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .plugin_discovery_snapshot
            .discovered_type_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.discovered_type_count),
        Some(1)
    );
    assert!(report
        .observation
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:vst3:instrument"
            && plugin.format == PluginFormat::Vst3
            && plugin.processing_contract.accepts_note_events));
    assert_eq!(
        report
            .observation
            .observation
            .plugin_lifecycle_snapshot
            .sandboxes
            .first()
            .and_then(|sandbox| sandbox.plugin_format),
        Some(PluginFormat::Vst3)
    );
    assert!(report.render_json().contains("\"node_id\":\"plugin-insert\""));
    assert!(report
        .render_json()
        .contains("\"plugin_sandbox_id\":\"local-default-sandbox\""));
    assert!(report.render_json().contains("\"input_bus_id\":\"bus:track:lead\""));
    assert!(report.render_json().contains("\"output_bus_id\":\"bus:mix:tracks\""));
    assert!(report
        .render_compact()
        .contains("host_audio_graph_matches_runtime=true"));
    assert!(report.render_compact().contains("metering_snapshot_routes=1/2/0/1"));
    assert!(report.render_json().contains("\"device_loss_count\":0"));
    assert!(report
        .render_json()
        .contains("\"metering_snapshot\":{\"meter_count\":"));
}
