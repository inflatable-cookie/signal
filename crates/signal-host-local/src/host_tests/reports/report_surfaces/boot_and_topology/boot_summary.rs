use super::super::super::super::*;

#[test]
fn local_host_boot_summary_exposes_negotiated_hardware_contract() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host.boot_default().expect("default local host boot");
    let supervisor = host.supervisor_report();

    assert_eq!(summary.backend_name, "coreaudio");
    assert!(summary.hardware.device_id.starts_with("coreaudio:"));
    assert!(!summary.hardware.device_name.is_empty());
    assert_eq!(summary.hardware.sample_rate, 48_000);
    assert_eq!(summary.hardware.buffer_size, 512);
    assert_eq!(summary.hardware.input_channels, 0);
    assert_eq!(summary.hardware.output_channels, 2);
    assert_eq!(summary.hardware.sample_format, AudioSampleFormat::F32);
    assert_eq!(
        summary.hardware.lifecycle,
        HardwareLifecycleContract {
            ownership: signal_hardware::HardwareLifecycleOwnership::HostDrivenCallback,
            restart_policy: signal_hardware::HardwareRestartPolicy::HostMustRestart,
        }
    );
    assert!(!summary.hardware.simulated);
    assert_eq!(
        supervisor
            .observation
            .effective_config
            .active_output_device
            .as_deref(),
        Some(summary.hardware.device_id.as_str())
    );
    assert_eq!(summary.hardware.backend_diagnostics.xrun_count, 0);
    assert_eq!(summary.hardware.backend_diagnostics.device_loss_count, 0);
    assert_eq!(
        summary.hardware.backend_diagnostics.health,
        signal_hardware::BackendHealth::Healthy
    );
    assert_eq!(summary.audio_pump.stream_state, LocalAudioStreamState::Running);
    assert_eq!(
        summary.audio_pump.transfer_policy,
        LocalAudioTransferPolicy {
            max_callback_frames: 512,
            max_transfer_channels: 2,
            zero_fill_unwritten_output: true,
        }
    );
    assert_eq!(summary.audio_pump.callback_count, 8);
    assert_eq!(summary.audio_pump.total_callback_frames, 8 * 512);
    assert_eq!(summary.audio_pump.total_runtime_output_frames, 8 * 512);
    assert_eq!(summary.audio_pump.copied_output_samples, 8 * 512 * 2);
    assert_eq!(summary.audio_pump.zero_filled_output_samples, 0);
    assert_eq!(summary.audio_pump.dropped_output_samples, 0);
    assert!(summary.audio_pump.last_callback_output_peak.is_some());
    assert_eq!(
        summary.audio_pump.last_runtime_graph_id.as_deref(),
        Some("signal.host.local.demo")
    );
    let plugin_state = summary
        .execution
        .last_plugin_state
        .as_ref()
        .expect("plugin instance state should be projected into local summary");
    assert_eq!(plugin_state.plugin_type_id, "plugin:clap:default");
    assert_eq!(plugin_state.instance_id, "instance:local:default");
    assert_eq!(plugin_state.lifecycle_state, "Active");
    assert_eq!(plugin_state.readiness_state, "Ready");
    assert!(plugin_state.active);
    assert_eq!(plugin_state.processing_sample_rate_hz, Some(48_000));
    assert_eq!(plugin_state.processing_max_block_frames, Some(512));
    assert!(plugin_state.last_fault.is_none());
    let observed_plugin_state = supervisor
        .observation
        .observation
        .last_plugin_instance_state()
        .expect("runtime observation should retain typed plugin state");
    assert_eq!(observed_plugin_state.instance_id, "instance:local:default");
    assert_eq!(observed_plugin_state.lifecycle_state, "Active");
    assert_eq!(observed_plugin_state.readiness_state, "Ready");
    assert!(supervisor
        .render_json()
        .contains("\"plugin_instance_state_events\":"));
}
