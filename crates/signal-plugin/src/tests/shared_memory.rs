use super::*;

#[test]
fn shared_memory_layout_regions_do_not_overlap() {
    let layout = SharedMemoryLayout::single_block(2048, 512);
    assert!(layout.audio_output.offset_bytes >= layout.audio_input.size_bytes);
    assert!(layout.completion.offset_bytes > layout.render_context.offset_bytes);
    assert_eq!(layout.total_bytes(), layout.completion.offset_bytes + 64);
}

#[test]
fn sandbox_state_machine_advances_through_processing_states() {
    let mut machine = SandboxStateMachine::new();
    let dispatch = BlockDispatch::new(
        PluginInstanceId("instance-1".into()),
        7,
        3,
        512,
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        },
        test_render_context(),
        1024,
    );

    machine.begin_block(&dispatch);
    assert_eq!(machine.slot().state, CompletionState::ReadyForProcessing);

    assert!(machine.mark_processing());
    assert_eq!(machine.slot().state, CompletionState::Processing);

    assert!(machine.mark_completed(7, 3));
    assert_eq!(machine.slot().state, CompletionState::Completed);
}

#[test]
fn completion_rejects_mismatched_epoch_or_sequence() {
    let mut machine = SandboxStateMachine::new();
    let dispatch = BlockDispatch::new(
        PluginInstanceId("instance-2".into()),
        5,
        11,
        256,
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        test_render_context(),
        512,
    );

    machine.begin_block(&dispatch);
    machine.mark_processing();

    assert!(!machine.mark_completed(4, 11));
    assert_eq!(machine.slot().state, CompletionState::Processing);
}

#[test]
fn plugin_descriptor_carries_neutral_contract_metadata() {
    let descriptor =
        PluginDescriptor::new("plugin:test", "Signal", "Test Plugin", PluginFormat::Clap)
            .with_version("1.2.3")
            .with_feature(crate::PluginFeature::AudioEffect)
            .with_audio_buses(
                PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                }
                .main_audio_buses(),
            )
            .with_parameters(vec![crate::PluginParameterDescriptor {
                parameter_id: 9,
                name: "Cutoff".into(),
                unit: Some("Hz".into()),
                domain: PluginParameterDomain::Hertz,
                default_normalized: 0.5,
                min_plain: 20.0,
                max_plain: 20_000.0,
                step_count: None,
                flags: PluginParameterFlags::automatable(),
            }])
            .with_state_contract(crate::PluginStateContract {
                supports_snapshot: true,
                supports_reset: true,
                supports_bypass: true,
                exposes_latency: false,
                exposes_tail: false,
            })
            .with_processing_contract(crate::PluginProcessingContract {
                max_block_frames: 2048,
                sample_accurate_automation: true,
                accepts_midi: true,
                accepts_note_events: true,
                supports_note_expression: true,
                produces_midi: false,
                silence_aware: true,
            })
            .with_lifecycle_contract(crate::PluginLifecycleContract {
                requires_main_thread_for_state: true,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: false,
            });

    assert_eq!(descriptor.version.as_deref(), Some("1.2.3"));
    assert_eq!(descriptor.audio_buses.len(), 2);
    assert_eq!(descriptor.parameters.len(), 1);
    assert!(descriptor.state_contract.supports_snapshot);
    assert!(descriptor.processing_contract.sample_accurate_automation);
    assert!(descriptor.lifecycle_contract.requires_main_thread_for_state);
}

#[test]
fn plugin_sandbox_errors_map_into_plugin_fault_readiness_taxonomy() {
    let protocol_error = PluginSandboxError::new(
        PluginSandboxErrorKind::ProtocolViolation,
        "sandbox protocol mismatch",
    )
    .as_fault();
    let crash_error =
        PluginSandboxError::new(PluginSandboxErrorKind::Crashed, "sandbox process exited")
            .as_fault();

    assert_eq!(protocol_error.kind, PluginFaultKind::ProtocolViolation);
    assert_eq!(protocol_error.severity, PluginFaultSeverity::Critical);
    assert!(matches!(
        PluginReadiness::from_fault(protocol_error),
        PluginReadiness::Failed { .. }
    ));

    assert_eq!(crash_error.kind, PluginFaultKind::Crash);
    assert_eq!(crash_error.severity, PluginFaultSeverity::Fatal);

    let snapshot = crate::PluginInstanceSnapshot {
        plugin_type_id: crate::PluginTypeId("plugin:test".into()),
        instance_id: PluginInstanceId("instance:test".into()),
        lifecycle_state: PluginLifecycleState::Prepared,
        readiness: PluginReadiness::Starting,
        processing: Some(crate::PluginProcessConfiguration {
            sample_rate_hz: 48_000,
            max_block_frames: 512,
            io_layout: PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        }),
    };
    assert_eq!(snapshot.lifecycle_state, PluginLifecycleState::Prepared);
}

#[test]
fn shared_memory_lease_tracks_epoch_invalidations() {
    let layout = SharedMemoryLayout::single_block(2048, 512);
    let mut lease = SharedMemoryLease::new("lease-a", 3, layout);

    assert_eq!(lease.total_bytes(), layout.total_bytes());
    assert!(lease.is_epoch_valid(3));
    assert!(lease.invalidate_epoch(3));
    assert!(!lease.is_epoch_valid(3));
    assert_eq!(lease.invalidated_epochs(), &[3]);
}

#[test]
fn shared_memory_lease_binds_transport_metadata() {
    let lease = SharedMemoryLease::new("lease-a", 4, SharedMemoryLayout::single_block(256, 64))
        .with_transport(SharedMemoryTransportPayload {
            region_id: "region-a".into(),
            transport_kind: SharedMemoryTransportKind::MappedFile,
            backing_path: "/tmp/region-a.signal-shm".into(),
            total_bytes: 320,
        });

    let transport = lease.transport().expect("transport binding");
    assert_eq!(transport.region_id, "region-a");
    assert_eq!(transport.total_bytes, 320);
}
