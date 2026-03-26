use signal_runtime::{
    RuntimeHostClockDomain, RuntimeHostClockFallbackState, RuntimeHostClockTransitionState,
};

use super::super::{LocalClockTransitionMemory, LocalRuntimeHost};

impl LocalRuntimeHost {
    pub(crate) fn host_clock_transition_state(
        &self,
        configured_stream: bool,
        clock_domain: RuntimeHostClockDomain,
        fallback_state: RuntimeHostClockFallbackState,
    ) -> RuntimeHostClockTransitionState {
        let mut memory = self.clock_transition_memory.borrow_mut();
        let transition = if !memory.initialized {
            RuntimeHostClockTransitionState::InitialObservation
        } else if memory.configured_stream && !configured_stream {
            RuntimeHostClockTransitionState::LostConfiguration
        } else if !memory.configured_stream && configured_stream {
            match clock_domain {
                RuntimeHostClockDomain::Aggregate => {
                    RuntimeHostClockTransitionState::EnteredAggregateClock
                }
                RuntimeHostClockDomain::Degraded => {
                    RuntimeHostClockTransitionState::EnteredRecoveryFallback
                }
                RuntimeHostClockDomain::CrossClock => {
                    RuntimeHostClockTransitionState::EnteredCrossClockFallback
                }
                RuntimeHostClockDomain::SameClock => {
                    RuntimeHostClockTransitionState::ReturnedToDirect
                }
            }
        } else if memory.domain == clock_domain && memory.fallback_state == fallback_state {
            RuntimeHostClockTransitionState::Stable
        } else if clock_domain == RuntimeHostClockDomain::Aggregate
            && memory.domain != RuntimeHostClockDomain::Aggregate
        {
            RuntimeHostClockTransitionState::EnteredAggregateClock
        } else if clock_domain == RuntimeHostClockDomain::Degraded
            && memory.domain != RuntimeHostClockDomain::Degraded
        {
            RuntimeHostClockTransitionState::EnteredRecoveryFallback
        } else if fallback_state == RuntimeHostClockFallbackState::RuntimeResampled
            && memory.fallback_state != RuntimeHostClockFallbackState::RuntimeResampled
        {
            RuntimeHostClockTransitionState::EnteredCrossClockFallback
        } else if fallback_state == RuntimeHostClockFallbackState::Direct
            && memory.fallback_state != RuntimeHostClockFallbackState::Direct
        {
            RuntimeHostClockTransitionState::ReturnedToDirect
        } else {
            RuntimeHostClockTransitionState::Reconfigured
        };

        *memory = LocalClockTransitionMemory {
            configured_stream,
            domain: clock_domain,
            fallback_state,
            initialized: true,
        };
        transition
    }
}
