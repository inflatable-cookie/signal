use super::*;

impl RuntimeHostIoSummary {
    /// Builds an unavailable/faulted [`RuntimeExternalIoSnapshot`] when no host I/O data is present.
    pub fn unavailable_external_io_snapshot(
        effective_config: &EffectiveRuntimeConfig,
        device_supervision_snapshot: &RuntimeDeviceSupervisionSnapshot,
    ) -> RuntimeExternalIoSnapshot {
        let faulted = matches!(
            device_supervision_snapshot.state,
            RuntimeDeviceSupervisionState::Faulted | RuntimeDeviceSupervisionState::Exhausted
        );
        RuntimeExternalIoSnapshot {
            health_state: if faulted {
                RuntimeExternalIoHealthState::Faulted
            } else {
                RuntimeExternalIoHealthState::Unavailable
            },
            device_change_state: RuntimeExternalIoDeviceChangeState::Unavailable,
            primary_role: RuntimeExternalIoPrimaryRole::Unavailable,
            monitoring_state: if faulted {
                RuntimeExternalIoMonitoringState::Faulted
            } else {
                RuntimeExternalIoMonitoringState::Unavailable
            },
            monitoring_tap_point: RuntimeExternalIoMonitoringTapPoint::Unavailable,
            loopback_state: if faulted {
                RuntimeExternalIoLoopbackState::Faulted
            } else {
                RuntimeExternalIoLoopbackState::Unavailable
            },
            io_layout: RuntimeMultichannelIoSummary::for_hardware(0, 0),
            linux_backend_identity: RuntimeLinuxAudioBackendIdentity::Unavailable,
            linux_backend_portability: RuntimeLinuxAudioBackendPortabilityBand::Unsupported,
            backend_name: "runtime-unavailable".into(),
            active_output_device_id: effective_config
                .active_output_device
                .clone()
                .unwrap_or_else(|| "runtime:unavailable".into()),
            active_output_device_name: effective_config
                .active_output_device
                .clone()
                .unwrap_or_else(|| "Unavailable External I/O".into()),
            stream_state: if faulted {
                RuntimeHostAudioStreamState::Faulted
            } else {
                RuntimeHostAudioStreamState::Stopped
            },
            clock_source: RuntimeHostClockSource::Internal,
            clock_domain: RuntimeHostClockDomain::Degraded,
            fallback_state: RuntimeHostClockFallbackState::Unconfigured,
            transition_state: RuntimeHostClockTransitionState::LostConfiguration,
            drift_state: RuntimeHostClockDriftState::Unconfigured,
            discontinuity_state: RuntimeHostClockDiscontinuityState::LostConfiguration,
            duplex_mismatch_state: RuntimeHostDuplexMismatchState::NotApplicable,
            endpoint_topology: RuntimeHostEndpointTopology::Unconfigured,
            linux_clocking_parity: RuntimeLinuxAudioBackendClockingParityBand::Unsupported,
            linux_duplex_parity: RuntimeLinuxAudioBackendDuplexParityState::Unsupported,
            linux_endpoint_topology_parity:
                RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported,
            partial_availability: false,
            fallback_active: false,
            runtime_graph_id_matches_pump: false,
            output_latency_samples: 0,
            estimated_output_latency_samples: 0,
            xrun_count: 0,
            callback_overrun_count: 0,
            device_loss_count: device_supervision_snapshot.device_loss_count,
            restart_attempt_count: device_supervision_snapshot.restart_attempt_count.unwrap_or(0),
            restart_failure_count: device_supervision_snapshot.restart_failure_count.unwrap_or(0),
            summary: format!(
                "health={:?} device_change={:?} role={:?} monitor={:?}/{:?} loopback={:?} backend=runtime-unavailable device={} stream={:?} endpoint={:?}",
                if faulted {
                    RuntimeExternalIoHealthState::Faulted
                } else {
                    RuntimeExternalIoHealthState::Unavailable
                },
                RuntimeExternalIoDeviceChangeState::Unavailable,
                RuntimeExternalIoPrimaryRole::Unavailable,
                if faulted {
                    RuntimeExternalIoMonitoringState::Faulted
                } else {
                    RuntimeExternalIoMonitoringState::Unavailable
                },
                RuntimeExternalIoMonitoringTapPoint::Unavailable,
                if faulted {
                    RuntimeExternalIoLoopbackState::Faulted
                } else {
                    RuntimeExternalIoLoopbackState::Unavailable
                },
                effective_config
                    .active_output_device
                    .as_deref()
                    .unwrap_or("runtime:unavailable"),
                if faulted {
                    RuntimeHostAudioStreamState::Faulted
                } else {
                    RuntimeHostAudioStreamState::Stopped
                },
                RuntimeHostEndpointTopology::Unconfigured,
            ),
        }
    }
}
