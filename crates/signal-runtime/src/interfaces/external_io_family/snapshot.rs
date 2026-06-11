use super::*;

impl RuntimeHostIoSummary {
    /// Derives a `RuntimeExternalIoSnapshot` from the current host I/O summary.
    pub fn build_external_io_snapshot(&self) -> RuntimeExternalIoSnapshot {
        let fallback_active = self.clocking.fallback_state != RuntimeHostClockFallbackState::Direct;
        let health_state = if self.audio_pump.stream_state == RuntimeHostAudioStreamState::Faulted {
            RuntimeExternalIoHealthState::Faulted
        } else if fallback_active {
            RuntimeExternalIoHealthState::FallbackActive
        } else if self.hardware.device_loss_count > 0
            || self.hardware.restart_attempt_count > 0
            || self.clocking.transition_state != RuntimeHostClockTransitionState::Stable
            || matches!(
                self.hardware.backend_health,
                BackendHealth::Degraded | BackendHealth::Recovering
            )
        {
            RuntimeExternalIoHealthState::Recovering
        } else {
            RuntimeExternalIoHealthState::Ready
        };
        let device_change_state = if self.audio_pump.stream_state
            == RuntimeHostAudioStreamState::Faulted
            && self.hardware.restart_failure_count > 0
        {
            RuntimeExternalIoDeviceChangeState::Failed
        } else if self.hardware.restart_attempt_count > 0
            || self.clocking.transition_state
                == RuntimeHostClockTransitionState::EnteredRecoveryFallback
        {
            RuntimeExternalIoDeviceChangeState::Recovering
        } else if self.hardware.device_loss_count > 0
            || self.clocking.transition_state != RuntimeHostClockTransitionState::Stable
        {
            RuntimeExternalIoDeviceChangeState::PendingRestart
        } else {
            RuntimeExternalIoDeviceChangeState::Stable
        };
        let primary_role = match self.clocking.endpoint_topology {
            RuntimeHostEndpointTopology::Unconfigured => RuntimeExternalIoPrimaryRole::Unavailable,
            RuntimeHostEndpointTopology::OutputOnly => RuntimeExternalIoPrimaryRole::ProgramOutput,
            RuntimeHostEndpointTopology::InputOnly => RuntimeExternalIoPrimaryRole::ExternalInput,
            RuntimeHostEndpointTopology::Duplex => RuntimeExternalIoPrimaryRole::ProgramDuplex,
            RuntimeHostEndpointTopology::Aggregate => {
                RuntimeExternalIoPrimaryRole::AggregateProgram
            }
        };
        let monitoring_state = if self.audio_pump.stream_state
            == RuntimeHostAudioStreamState::Faulted
        {
            RuntimeExternalIoMonitoringState::Faulted
        } else if matches!(
            self.clocking.endpoint_topology,
            RuntimeHostEndpointTopology::Unconfigured
        ) {
            RuntimeExternalIoMonitoringState::Unavailable
        } else if matches!(
            self.hardware.backend_health,
            BackendHealth::Degraded | BackendHealth::Recovering
        ) {
            RuntimeExternalIoMonitoringState::Degraded
        } else if fallback_active
            || self.clocking.partial_availability
            || self.clocking.duplex_mismatch_state != RuntimeHostDuplexMismatchState::NotApplicable
            || self.clocking.endpoint_topology == RuntimeHostEndpointTopology::Aggregate
        {
            RuntimeExternalIoMonitoringState::Guarded
        } else {
            RuntimeExternalIoMonitoringState::Direct
        };
        let monitoring_tap_point = match primary_role {
            RuntimeExternalIoPrimaryRole::Unavailable => {
                RuntimeExternalIoMonitoringTapPoint::Unavailable
            }
            RuntimeExternalIoPrimaryRole::ExternalInput => {
                RuntimeExternalIoMonitoringTapPoint::PreGraphInput
            }
            RuntimeExternalIoPrimaryRole::ProgramOutput
            | RuntimeExternalIoPrimaryRole::ProgramDuplex
            | RuntimeExternalIoPrimaryRole::AggregateProgram => {
                RuntimeExternalIoMonitoringTapPoint::PostHardwareOutput
            }
        };
        let loopback_state = if self.audio_pump.stream_state == RuntimeHostAudioStreamState::Faulted
        {
            RuntimeExternalIoLoopbackState::Faulted
        } else if matches!(
            self.hardware.backend_health,
            BackendHealth::Degraded | BackendHealth::Recovering
        ) {
            RuntimeExternalIoLoopbackState::Recovering
        } else if self.clocking.endpoint_topology == RuntimeHostEndpointTopology::Duplex
            && !self.clocking.partial_availability
            && self.clocking.clock_domain == RuntimeHostClockDomain::SameClock
        {
            RuntimeExternalIoLoopbackState::Ready
        } else if matches!(
            self.clocking.endpoint_topology,
            RuntimeHostEndpointTopology::Duplex | RuntimeHostEndpointTopology::Aggregate
        ) || self.clocking.partial_availability
            || fallback_active
        {
            RuntimeExternalIoLoopbackState::Guarded
        } else {
            RuntimeExternalIoLoopbackState::Unavailable
        };

        RuntimeExternalIoSnapshot {
            health_state,
            device_change_state,
            primary_role,
            monitoring_state,
            monitoring_tap_point,
            loopback_state,
            io_layout: RuntimeMultichannelIoSummary::for_hardware(
                self.hardware.input_channels,
                self.hardware.output_channels,
            ),
            backend_name: self.hardware.backend_name.clone(),
            active_output_device_id: self.hardware.device_id.clone(),
            active_output_device_name: self.hardware.device_name.clone(),
            stream_state: self.audio_pump.stream_state,
            clock_source: self.clocking.clock_source,
            clock_domain: self.clocking.clock_domain,
            fallback_state: self.clocking.fallback_state,
            transition_state: self.clocking.transition_state,
            drift_state: self.clocking.drift_state,
            discontinuity_state: self.clocking.discontinuity_state,
            duplex_mismatch_state: self.clocking.duplex_mismatch_state,
            endpoint_topology: self.clocking.endpoint_topology,
            partial_availability: self.clocking.partial_availability,
            fallback_active,
            runtime_graph_id_matches_pump: self.runtime_graph_id_matches_pump,
            output_latency_samples: self.latency.output_latency_samples,
            estimated_output_latency_samples: self.latency.estimated_output_latency_samples,
            xrun_count: self.hardware.xrun_count,
            callback_overrun_count: self.hardware.callback_overrun_count,
            device_loss_count: self.hardware.device_loss_count,
            restart_attempt_count: self.hardware.restart_attempt_count,
            restart_failure_count: self.hardware.restart_failure_count,
        }
    }
}
