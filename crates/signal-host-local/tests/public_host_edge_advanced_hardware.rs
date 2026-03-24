use signal_host_local::LocalRuntimeHost;
use signal_runtime::{RuntimeAdvancedHardwareGraphState, RuntimeConfig, SignalRuntime};

#[test]
fn local_shared_host_edge_exports_runtime_advanced_hardware_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let host = LocalRuntimeHost::new(runtime);
    let report = host.host_supervisor_report();

    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .graph_state,
        RuntimeAdvancedHardwareGraphState::Empty
    );
    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .provider_name,
        "signal-host-local"
    );
    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .device_count,
        0
    );
    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .display_transport_device_count,
        0
    );
    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .motor_transport_device_count,
        0
    );
    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .haptic_transport_device_count,
        0
    );
    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .scene_mapping_device_count,
        0
    );
    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .feedback_page_device_count,
        0
    );
    assert_eq!(
        report
            .observation
            .observation
            .advanced_hardware_snapshot
            .safe_action_graph_device_count,
        0
    );
    assert!(report
        .observation
        .observation
        .advanced_hardware_snapshot
        .devices
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"advanced_hardware_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-local\""));
    assert!(rendered.contains("\"display_transport_device_count\":0"));
    assert!(rendered.contains("\"motor_transport_device_count\":0"));
    assert!(rendered.contains("\"haptic_transport_device_count\":0"));
    assert!(rendered.contains("\"scene_mapping_device_count\":0"));
    assert!(rendered.contains("\"feedback_page_device_count\":0"));
    assert!(rendered.contains("\"safe_action_graph_device_count\":0"));
}
