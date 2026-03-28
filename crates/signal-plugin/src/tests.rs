// Tests for signal-plugin
#[allow(clippy::module_inception)]
mod tests {
    use crate::{
        AudioBlock, AutomationContinuityReport, BlockDispatch, BlockPayload, BlockProcessResult,
        BlockSequenceContinuityReport, CompletionSlot, CompletionState, EventPacket, LoopRange,
        MidiEvent, NoteEvent, NoteEventKind, NoteExpressionEvent, NoteExpressionKind,
        ParameterAutomationSummary, ParameterGestureEvent, ParameterGesturePhase,
        ParameterModulationEvent, ParameterValueEvent, PluginDescriptor, PluginEvent,
        PluginFaultKind, PluginFaultSeverity, PluginFormat, PluginInstanceId, PluginIoLayout,
        PluginLifecycleState, PluginParameterDomain, PluginParameterFlags, PluginReadiness,
        PluginRenderContext, PluginSandboxCapabilities, PluginSandboxError, PluginSandboxErrorKind,
        RestartEscalationPolicy, RestartEscalationState, SandboxControlRequest,
        SandboxControlResponse, SandboxStateMachine, SandboxTransport, SandboxWatchdogPolicy,
        SandboxWatchdogState, SharedMemoryLayout, SharedMemoryLease, WatchdogOutcome,
        WatchdogTriggerReason,
    };
    use signal_ipc::{SharedMemoryTransportKind, SharedMemoryTransportPayload};

    fn test_render_context() -> PluginRenderContext {
        PluginRenderContext {
            sample_rate_hz: 48_000,
            tempo_bpm: 120.0,
            timeline_position_samples: 0,
            playing: true,
            bypassed: false,
            loop_range: Some(LoopRange {
                start_samples: 0,
                end_samples: 96_000,
            }),
            deadline_frames: 512,
        }
    }

    fn test_payload(dispatch: &BlockDispatch) -> BlockPayload {
        let sample_count =
            dispatch.header.channel_count as usize * dispatch.header.frame_count as usize;
        let audio = AudioBlock::new(
            dispatch.header.channel_count,
            dispatch.header.frame_count,
            (0..sample_count).map(|index| index as f32).collect(),
        )
        .expect("audio block");
        let events = EventPacket::new(vec![
            PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: 32,
                parameter_id: 7,
                normalized_value: 0.5,
            }),
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: 40,
                parameter_id: 7,
                phase: ParameterGesturePhase::Begin,
            }),
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: 48,
                parameter_id: 7,
                phase: ParameterGesturePhase::End,
            }),
            PluginEvent::NoteExpression(NoteExpressionEvent {
                offset_frames: 56,
                note_id: 7,
                port_index: 0,
                channel: 0,
                key: 60,
                expression: NoteExpressionKind::Pressure,
                value: 0.6,
            }),
            PluginEvent::Midi(MidiEvent {
                offset_frames: 64,
                status: 0x90,
                data1: 60,
                data2: 96,
            }),
        ]);
        BlockPayload::new(audio, events)
    }

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
    fn handshake_request_and_response_capture_protocol_defaults() {
        let request = SandboxControlRequest::handshake("sandbox-a", PluginFormat::Clap);
        let response = SandboxControlResponse::HandshakeAccepted {
            protocol_version: 1,
            capabilities: PluginSandboxCapabilities {
                transport: SandboxTransport::SharedMemory,
                supports_state: true,
                supports_midi: true,
                max_block_frames: 2048,
            },
        };

        assert_eq!(request.sandbox_id, "sandbox-a");
        assert_eq!(request.format, PluginFormat::Clap);
        assert!(matches!(
            response,
            SandboxControlResponse::HandshakeAccepted { .. }
        ));
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

    #[test]
    fn block_dispatch_round_trips_through_shared_memory_regions() {
        let dispatch = BlockDispatch::new(
            PluginInstanceId("instance-dispatch".into()),
            5,
            9,
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
        let mut bytes = vec![0; dispatch.layout.total_bytes() as usize];

        dispatch
            .write_to_shared_memory(&mut bytes)
            .expect("write dispatch");
        let decoded = BlockDispatch::read_from_shared_memory(
            PluginInstanceId("instance-dispatch".into()),
            dispatch.io_layout,
            dispatch.layout,
            &bytes,
        )
        .expect("decode dispatch");

        assert_eq!(decoded.header, dispatch.header);
        assert_eq!(decoded.render_context, dispatch.render_context);
    }

    #[test]
    fn block_process_result_round_trips_through_completion_region() {
        let dispatch = BlockDispatch::new(
            PluginInstanceId("instance-result".into()),
            3,
            4,
            128,
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 0,
                midi_outputs: 0,
            },
            test_render_context(),
            256,
        );
        let mut bytes = vec![0; dispatch.layout.total_bytes() as usize];
        let result = BlockProcessResult {
            slot: CompletionSlot {
                state: CompletionState::Completed,
                processing_epoch: 3,
                block_sequence: 4,
            },
            generated_event_bytes: 64,
            fallback_applied: false,
        };

        result
            .write_to_shared_memory(dispatch.layout, &mut bytes)
            .expect("write result");
        let decoded = BlockProcessResult::read_from_shared_memory(dispatch.layout, &bytes)
            .expect("decode result");

        assert_eq!(decoded, result);
    }

    #[test]
    fn block_payload_round_trips_through_audio_and_event_regions() {
        let dispatch = BlockDispatch::new(
            PluginInstanceId("instance-payload".into()),
            11,
            6,
            128,
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            test_render_context(),
            256,
        );
        let payload = test_payload(&dispatch);
        let mut bytes = vec![0; dispatch.layout.total_bytes() as usize];

        dispatch
            .write_input_payload(&mut bytes, &payload)
            .expect("write input payload");
        let decoded_input = dispatch
            .read_input_payload(&bytes)
            .expect("decode input payload");
        assert_eq!(decoded_input, payload);

        dispatch
            .write_output_payload(&mut bytes, &payload)
            .expect("write output payload");
        let decoded_output = dispatch
            .read_output_payload(&bytes)
            .expect("decode output payload");
        assert_eq!(decoded_output, payload);
    }

    #[test]
    fn event_packet_summary_counts_richer_event_types() {
        let packet = EventPacket::new(vec![
            PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: 0,
                parameter_id: 3,
                normalized_value: 0.1,
            }),
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: 4,
                parameter_id: 3,
                phase: ParameterGesturePhase::Begin,
            }),
            PluginEvent::ParameterModulation(ParameterModulationEvent {
                offset_frames: 8,
                parameter_id: 9,
                amount: -0.2,
            }),
            PluginEvent::Note(NoteEvent {
                offset_frames: 16,
                note_id: 7,
                port_index: 0,
                channel: 0,
                key: 60,
                velocity: 0.8,
                kind: NoteEventKind::NoteOn,
            }),
            PluginEvent::NoteExpression(NoteExpressionEvent {
                offset_frames: 24,
                note_id: 7,
                port_index: 0,
                channel: 0,
                key: 60,
                expression: NoteExpressionKind::Pressure,
                value: 0.7,
            }),
            PluginEvent::NoteExpression(NoteExpressionEvent {
                offset_frames: 28,
                note_id: 7,
                port_index: 0,
                channel: 1,
                key: 61,
                expression: NoteExpressionKind::Timbre,
                value: 0.5,
            }),
            PluginEvent::NoteExpression(NoteExpressionEvent {
                offset_frames: 30,
                note_id: 7,
                port_index: 0,
                channel: 2,
                key: 62,
                expression: NoteExpressionKind::Tuning,
                value: 0.2,
            }),
            PluginEvent::Midi(MidiEvent {
                offset_frames: 32,
                status: 0xB0,
                data1: 1,
                data2: 100,
            }),
        ]);

        let summary = packet.summary();
        assert_eq!(summary.total_events, 8);
        assert_eq!(summary.parameter_value_events, 1);
        assert_eq!(summary.parameter_gesture_events, 1);
        assert_eq!(summary.parameter_modulation_events, 1);
        assert_eq!(summary.note_events, 1);
        assert_eq!(summary.note_expression_events, 3);
        assert_eq!(summary.note_expression_pressure_events, 1);
        assert_eq!(summary.note_expression_timbre_events, 1);
        assert_eq!(summary.note_expression_tuning_events, 1);
        assert_eq!(summary.midi_events, 1);
    }

    #[test]
    fn parameter_automation_summary_tracks_values_modulation_and_gestures() {
        let packet = EventPacket::new(vec![
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: 0,
                parameter_id: 77,
                phase: ParameterGesturePhase::Begin,
            }),
            PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: 4,
                parameter_id: 77,
                normalized_value: 0.2,
            }),
            PluginEvent::ParameterModulation(ParameterModulationEvent {
                offset_frames: 8,
                parameter_id: 77,
                amount: -0.1,
            }),
            PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: 16,
                parameter_id: 77,
                normalized_value: 0.6,
            }),
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: 20,
                parameter_id: 77,
                phase: ParameterGesturePhase::End,
            }),
            PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: 24,
                parameter_id: 9,
                normalized_value: 0.9,
            }),
        ]);

        let summary = packet.parameter_automation_summary(77);
        assert_eq!(
            summary,
            ParameterAutomationSummary {
                parameter_id: 77,
                value_events: 2,
                modulation_events: 1,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(0.2),
                last_value: Some(0.6),
                last_modulation: Some(-0.1),
            }
        );
    }

    #[test]
    fn automation_continuity_report_tracks_segments_and_lease_rollovers() {
        let mut report = AutomationContinuityReport::default();
        report.record(
            2,
            "lease-a",
            ParameterAutomationSummary {
                parameter_id: 77,
                value_events: 1,
                modulation_events: 1,
                gesture_begin_events: 1,
                gesture_end_events: 0,
                first_value: Some(0.1),
                last_value: Some(0.1),
                last_modulation: Some(0.02),
            },
        );
        report.record(
            2,
            "lease-a",
            ParameterAutomationSummary {
                parameter_id: 77,
                value_events: 1,
                modulation_events: 1,
                gesture_begin_events: 0,
                gesture_end_events: 1,
                first_value: Some(0.15),
                last_value: Some(0.15),
                last_modulation: Some(0.04),
            },
        );
        report.record(
            3,
            "lease-b",
            ParameterAutomationSummary {
                parameter_id: 77,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(0.2),
                last_value: Some(0.25),
                last_modulation: Some(0.06),
            },
        );

        assert_eq!(report.parameter_id, 77);
        assert_eq!(report.segment_count(), 2);
        assert_eq!(report.lease_rollovers, 1);
        assert_eq!(report.first_epoch(), Some(2));
        assert_eq!(report.last_epoch(), Some(3));
        assert_eq!(report.segment_epochs(), vec![2, 3]);

        let aggregate = report.aggregate();
        assert_eq!(aggregate.value_events, 4);
        assert_eq!(aggregate.modulation_events, 4);
        assert_eq!(aggregate.gesture_begin_events, 2);
        assert_eq!(aggregate.gesture_end_events, 2);
        assert_eq!(aggregate.first_value, Some(0.1));
        assert_eq!(aggregate.last_value, Some(0.25));
        assert_eq!(aggregate.last_modulation, Some(0.06));
    }

    #[test]
    fn block_sequence_continuity_report_tracks_rollovers_and_gaps() {
        let mut report = BlockSequenceContinuityReport::default();
        report.record(2, "lease-a", 0);
        report.record(2, "lease-a", 1);
        report.record(2, "lease-a", 3);
        report.record(3, "lease-b", 4);
        report.record(3, "lease-b", 5);

        assert_eq!(report.segment_count(), 3);
        assert_eq!(report.segment_epochs(), vec![2, 2, 3]);
        assert_eq!(report.first_block_sequence(), Some(0));
        assert_eq!(report.last_block_sequence(), Some(5));
        assert_eq!(report.sequence_gaps, 1);
        assert_eq!(report.lease_rollovers, 1);
    }

    #[test]
    fn sandbox_watchdog_requires_restart_after_consecutive_timeouts() {
        let mut watchdog = SandboxWatchdogState::new(SandboxWatchdogPolicy {
            max_consecutive_deadline_misses: 2,
            max_consecutive_heartbeat_misses: 3,
        });

        assert_eq!(
            watchdog.record_block_completion(CompletionState::TimedOut),
            WatchdogOutcome::Healthy
        );
        assert_eq!(
            watchdog.record_block_completion(CompletionState::TimedOut),
            WatchdogOutcome::RestartRequired {
                reason: WatchdogTriggerReason::DeadlineMisses,
                consecutive_misses: 2,
            }
        );
    }

    #[test]
    fn sandbox_watchdog_resets_heartbeat_misses_after_response() {
        let mut watchdog = SandboxWatchdogState::new(SandboxWatchdogPolicy {
            max_consecutive_deadline_misses: 2,
            max_consecutive_heartbeat_misses: 2,
        });

        assert_eq!(watchdog.record_heartbeat_miss(), WatchdogOutcome::Healthy);
        watchdog.record_heartbeat_response();
        assert_eq!(watchdog.consecutive_heartbeat_misses(), 0);
    }

    #[test]
    fn restart_escalation_requests_safe_mode_after_threshold() {
        let mut escalation = RestartEscalationState::new(RestartEscalationPolicy {
            safe_mode_restart_threshold: 2,
        });

        assert!(!escalation.record_watchdog_restart());
        assert_eq!(escalation.watchdog_restart_count(), 1);
        assert!(escalation.record_watchdog_restart());
        assert!(escalation.safe_mode_requested());
    }
}
