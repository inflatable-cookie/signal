use signal_hardware::{
    AudioSampleFormat, HardwareDiagnosticsSnapshot, HardwareLifecycleContract,
    HardwareLifecycleOwnership, HardwareRestartPolicy,
};
use signal_host_local::{
    host::{
        LocalExecutionSummary, LocalFaultSummary, LocalHardwareSummary, LocalPayloadSummary,
        LocalTransportSummary,
    },
    LocalAudioPumpSummary, LocalAudioStreamState, LocalAudioTransferPolicy,
    LocalRuntimeHostSummary, RecoveryRestartIntent,
};
use signal_plugin::{CompletionState, WatchdogTriggerReason};
use signal_runtime::{RuntimeExecutionTopologySummary, StopReason};

pub(crate) fn sample_local_summary() -> LocalRuntimeHostSummary {
    LocalRuntimeHostSummary {
        backend_name: "coreaudio",
        hardware: LocalHardwareSummary {
            device_id: "coreaudio:default-output".into(),
            device_name: "CoreAudio Default Output".into(),
            sample_rate: 48_000,
            buffer_size: 512,
            input_channels: 0,
            output_channels: 2,
            sample_format: AudioSampleFormat::F32,
            lifecycle: HardwareLifecycleContract {
                ownership: HardwareLifecycleOwnership::HostDrivenCallback,
                restart_policy: HardwareRestartPolicy::HostMustRestart,
            },
            simulated: false,
            backend_diagnostics: HardwareDiagnosticsSnapshot::healthy(),
        },
        audio_pump: LocalAudioPumpSummary {
            stream_state: LocalAudioStreamState::Running,
            transfer_policy: LocalAudioTransferPolicy {
                max_callback_frames: 512,
                max_transfer_channels: 2,
                zero_fill_unwritten_output: true,
            },
            callback_count: 3,
            last_callback_index: Some(2),
            total_callback_frames: 1536,
            total_runtime_output_frames: 1536,
            copied_output_samples: 3072,
            zero_filled_output_samples: 0,
            dropped_output_samples: 0,
            last_callback_output_peak: Some(0.8),
            last_runtime_graph_id: Some("signal.host.local.demo".into()),
        },
        scan_roots: vec!["/plugins".into()],
        execution: LocalExecutionSummary {
            control_requests: 4,
            control_responses: 4,
            heartbeat_responses: 2,
            processed_blocks: 3,
            engine_processed_blocks: 3,
            last_control_message: "activateInstance".into(),
            last_completion_state: CompletionState::Completed,
            last_block_sequence: 7,
            last_engine_graph_id: Some("signal.host.local.demo".into()),
            last_engine_output_peak: Some(0.8),
            last_engine_output_rms: Some(0.42),
            processing_epoch: 2,
            restart_count: 1,
            teardown_count: 1,
            last_recovery_intent: Some(RecoveryRestartIntent::WatchdogRecovery),
            last_stop_reason: Some(StopReason::DegradedModeRecovery),
            last_plugin_state: None,
        },
        transport: LocalTransportSummary {
            sandbox_id: "sandbox-1".into(),
            shared_memory_lease_id: "lease-1".into(),
            shared_memory_region_id: "region-1".into(),
            shared_memory_path: "/tmp/signal-region-1".into(),
            shared_memory_bytes: 4096,
        },
        topology: RuntimeExecutionTopologySummary::default(),
        plugin_dispatch: None,
        last_payload: LocalPayloadSummary {
            event_count: 6,
            parameter_event_count: 2,
            parameter_gesture_event_count: 2,
            parameter_modulation_event_count: 1,
            note_event_count: 1,
            note_expression_event_count: 1,
            midi_event_count: 1,
            generated_event_bytes: 128,
            first_output_sample: Some(0.5),
        },
        faults: LocalFaultSummary {
            deadline_misses: 1,
            heartbeat_misses: 0,
            watchdog_triggered: true,
            watchdog_trigger_reason: Some(WatchdogTriggerReason::DeadlineMisses),
        },
    }
}
