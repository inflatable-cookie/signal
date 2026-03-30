use crate::{
    RuntimeControllerExpressionMidi2Posture, RuntimeExternalMidiDiscoveryState,
    RuntimeExternalMidiEndpointDirection, RuntimeExternalMidiEndpointGraphSnapshot,
    RuntimeExternalMidiGraphState,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceGraphState {
    Unavailable,
    Empty,
    Ready,
    Guarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceTransportPosture {
    Unavailable,
    InputOnly,
    FeedbackOnly,
    Duplex,
    Guarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceMappingPosture {
    Unsupported,
    ObserveOnly,
    Guarded,
    Portable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceFeedbackReadiness {
    Unavailable,
    Guarded,
    Ready,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeControlSurfaceCapabilitySummary {
    pub supports_transport_control: bool,
    pub supports_mapping_input: bool,
    pub supports_feedback_output: bool,
    pub supports_widened_expression: bool,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeControlSurfaceDeviceDescriptor {
    pub device_id: String,
    pub device_name: String,
    pub transport_posture: RuntimeControlSurfaceTransportPosture,
    pub mapping_posture: RuntimeControlSurfaceMappingPosture,
    pub feedback_readiness: RuntimeControlSurfaceFeedbackReadiness,
    pub capability: RuntimeControlSurfaceCapabilitySummary,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeControlSurfaceSnapshot {
    pub discovery_state: RuntimeExternalMidiDiscoveryState,
    pub graph_state: RuntimeControlSurfaceGraphState,
    pub provider_name: String,
    pub device_count: usize,
    pub mapped_device_count: usize,
    pub feedback_ready_device_count: usize,
    pub guarded_device_count: usize,
    pub devices: Vec<RuntimeControlSurfaceDeviceDescriptor>,
    pub summary: String,
}

impl RuntimeControlSurfaceSnapshot {
    pub fn unavailable() -> Self {
        Self {
            discovery_state: RuntimeExternalMidiDiscoveryState::Unavailable,
            graph_state: RuntimeControlSurfaceGraphState::Unavailable,
            provider_name: "runtime-unavailable".into(),
            device_count: 0,
            mapped_device_count: 0,
            feedback_ready_device_count: 0,
            guarded_device_count: 0,
            devices: Vec::new(),
            summary: "discovery=Unavailable graph=Unavailable provider=runtime-unavailable devices=0 mapped=0 feedback-ready=0 guarded=0".into(),
        }
    }

    pub fn empty(provider_name: impl Into<String>) -> Self {
        let provider_name = provider_name.into();
        Self {
            discovery_state: RuntimeExternalMidiDiscoveryState::Idle,
            graph_state: RuntimeControlSurfaceGraphState::Empty,
            provider_name: provider_name.clone(),
            device_count: 0,
            mapped_device_count: 0,
            feedback_ready_device_count: 0,
            guarded_device_count: 0,
            devices: Vec::new(),
            summary: format!(
                "discovery=Idle graph=Empty provider={} devices=0 mapped=0 feedback-ready=0 guarded=0",
                provider_name
            ),
        }
    }

    pub fn from_external_midi_snapshot(
        snapshot: &RuntimeExternalMidiEndpointGraphSnapshot,
    ) -> Self {
        if matches!(
            snapshot.discovery_state,
            RuntimeExternalMidiDiscoveryState::Unavailable
        ) || matches!(
            snapshot.graph_state,
            RuntimeExternalMidiGraphState::Unavailable
        ) {
            return Self::unavailable();
        }
        if snapshot.device_count == 0 {
            return Self::empty(snapshot.provider_name.clone());
        }

        let mut devices = Vec::with_capacity(snapshot.devices.len());
        let mut mapped_device_count = 0;
        let mut feedback_ready_device_count = 0;
        let mut guarded_device_count = 0;

        for device in &snapshot.devices {
            let endpoints = snapshot
                .endpoints
                .iter()
                .filter(|endpoint| endpoint.device_id == device.device_id)
                .collect::<Vec<_>>();
            let has_input = endpoints.iter().any(|endpoint| {
                matches!(
                    endpoint.direction,
                    RuntimeExternalMidiEndpointDirection::Input
                        | RuntimeExternalMidiEndpointDirection::Duplex
                )
            });
            let has_output = endpoints.iter().any(|endpoint| {
                matches!(
                    endpoint.direction,
                    RuntimeExternalMidiEndpointDirection::Output
                        | RuntimeExternalMidiEndpointDirection::Duplex
                )
            });
            let supports_transport_control = endpoints.iter().any(|endpoint| {
                endpoint.capability.supports_transport_clock
                    || endpoint.capability.supports_controller_events
            });
            let supports_mapping_input = endpoints.iter().any(|endpoint| {
                endpoint.capability.supports_controller_events
                    || endpoint.capability.supports_note_events
            });
            let supports_feedback_output = has_output;
            let supports_widened_expression = endpoints.iter().any(|endpoint| {
                endpoint.capability.supports_note_pressure_expression
                    || endpoint.capability.supports_note_timbre_expression
                    || endpoint.capability.supports_note_tuning_expression
                    || endpoint.capability.supports_mpe
                    || !matches!(
                        endpoint.capability.midi2_posture,
                        RuntimeControllerExpressionMidi2Posture::Unsupported
                    )
            });
            let guarded = endpoints
                .iter()
                .any(|endpoint| endpoint.capability.control_surface_guarded)
                || matches!(snapshot.graph_state, RuntimeExternalMidiGraphState::Guarded)
                || (!has_input && !has_output);

            let transport_posture = if endpoints.is_empty() {
                RuntimeControlSurfaceTransportPosture::Unavailable
            } else if guarded {
                RuntimeControlSurfaceTransportPosture::Guarded
            } else if has_input && has_output {
                RuntimeControlSurfaceTransportPosture::Duplex
            } else if has_input {
                RuntimeControlSurfaceTransportPosture::InputOnly
            } else if has_output {
                RuntimeControlSurfaceTransportPosture::FeedbackOnly
            } else {
                RuntimeControlSurfaceTransportPosture::Unavailable
            };
            let mapping_posture = if !supports_mapping_input {
                RuntimeControlSurfaceMappingPosture::Unsupported
            } else if guarded || supports_widened_expression {
                RuntimeControlSurfaceMappingPosture::Guarded
            } else if !supports_transport_control && !supports_feedback_output {
                RuntimeControlSurfaceMappingPosture::ObserveOnly
            } else {
                RuntimeControlSurfaceMappingPosture::Portable
            };
            let feedback_readiness = if !supports_feedback_output {
                RuntimeControlSurfaceFeedbackReadiness::Unavailable
            } else if guarded {
                RuntimeControlSurfaceFeedbackReadiness::Guarded
            } else {
                RuntimeControlSurfaceFeedbackReadiness::Ready
            };

            if !matches!(
                mapping_posture,
                RuntimeControlSurfaceMappingPosture::Unsupported
            ) {
                mapped_device_count += 1;
            }
            if matches!(
                feedback_readiness,
                RuntimeControlSurfaceFeedbackReadiness::Ready
            ) {
                feedback_ready_device_count += 1;
            }
            if guarded {
                guarded_device_count += 1;
            }

            let capability = RuntimeControlSurfaceCapabilitySummary {
                supports_transport_control,
                supports_mapping_input,
                supports_feedback_output,
                supports_widened_expression,
                summary: format!(
                    "transport-control={} mapping-input={} feedback-output={} widened-expression={}",
                    supports_transport_control,
                    supports_mapping_input,
                    supports_feedback_output,
                    supports_widened_expression
                ),
            };
            devices.push(RuntimeControlSurfaceDeviceDescriptor {
                device_id: device.device_id.clone(),
                device_name: device.device_name.clone(),
                transport_posture,
                mapping_posture,
                feedback_readiness,
                capability: capability.clone(),
                summary: format!(
                    "transport={:?} mapping={:?} feedback={:?} capability={}",
                    transport_posture, mapping_posture, feedback_readiness, capability.summary
                ),
            });
        }

        let graph_state = if guarded_device_count > 0 {
            RuntimeControlSurfaceGraphState::Guarded
        } else {
            RuntimeControlSurfaceGraphState::Ready
        };

        Self {
            discovery_state: snapshot.discovery_state,
            graph_state,
            provider_name: snapshot.provider_name.clone(),
            device_count: devices.len(),
            mapped_device_count,
            feedback_ready_device_count,
            guarded_device_count,
            devices,
            summary: format!(
                "discovery={:?} graph={:?} provider={} devices={} mapped={} feedback-ready={} guarded={}",
                snapshot.discovery_state,
                graph_state,
                snapshot.provider_name,
                snapshot.devices.len(),
                mapped_device_count,
                feedback_ready_device_count,
                guarded_device_count
            ),
        }
    }
}
