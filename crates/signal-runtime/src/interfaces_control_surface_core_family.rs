use crate::{
    RuntimeControllerExpressionMidi2Posture, RuntimeExternalMidiDiscoveryState,
    RuntimeExternalMidiEndpointDirection, RuntimeExternalMidiEndpointGraphSnapshot,
    RuntimeExternalMidiGraphState,
};

/// Overall state of the control surface device graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceGraphState {
    /// Control surface graph backend is unavailable.
    Unavailable,
    /// Graph is available but no control surface devices are connected.
    Empty,
    /// All control surface devices are ready with no guarded conditions.
    Ready,
    /// One or more devices are in a guarded state.
    Guarded,
}

/// Transport control direction capability of a control surface device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceTransportPosture {
    /// Transport control is not available for this device.
    Unavailable,
    /// Device supports input (receiving) transport controls only.
    InputOnly,
    /// Device supports feedback (sending) transport updates only.
    FeedbackOnly,
    /// Device supports both input and feedback transport control.
    Duplex,
    /// Transport control capability is in a guarded state.
    Guarded,
}

/// Mapping input capability of a control surface device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceMappingPosture {
    /// Device does not support mapping input.
    Unsupported,
    /// Device can observe mappings but cannot issue transport or feedback commands.
    ObserveOnly,
    /// Mapping input is available but in a guarded state.
    Guarded,
    /// Mapping input is portable across sessions.
    Portable,
}

/// Feedback output readiness of a control surface device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceFeedbackReadiness {
    /// Feedback output is not available for this device.
    Unavailable,
    /// Feedback output is present but in a guarded state.
    Guarded,
    /// Feedback output is ready and can be written.
    Ready,
}

/// Capability flags for a control surface device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeControlSurfaceCapabilitySummary {
    /// Whether the device supports sending transport control commands.
    pub supports_transport_control: bool,
    /// Whether the device supports receiving mapping input events.
    pub supports_mapping_input: bool,
    /// Whether the device supports writing feedback output.
    pub supports_feedback_output: bool,
    /// Whether the device supports widened expression (MPE, MIDI 2.0, etc.).
    pub supports_widened_expression: bool,
    /// Human-readable summary of the capability flags.
    pub summary: String,
}

/// Descriptor for a single control surface device including posture and capability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeControlSurfaceDeviceDescriptor {
    /// Stable identifier for this control surface device.
    pub device_id: String,
    /// Human-readable name for this control surface device.
    pub device_name: String,
    /// Transport control direction capability of this device.
    pub transport_posture: RuntimeControlSurfaceTransportPosture,
    /// Mapping input capability of this device.
    pub mapping_posture: RuntimeControlSurfaceMappingPosture,
    /// Feedback output readiness of this device.
    pub feedback_readiness: RuntimeControlSurfaceFeedbackReadiness,
    /// Aggregated capability flags for this device.
    pub capability: RuntimeControlSurfaceCapabilitySummary,
    /// Human-readable summary of this device descriptor.
    pub summary: String,
}

/// Aggregate snapshot of all control surface devices and their mapping/feedback counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeControlSurfaceSnapshot {
    /// Current discovery phase of the underlying MIDI graph.
    pub discovery_state: RuntimeExternalMidiDiscoveryState,
    /// Overall readiness of the control surface device graph.
    pub graph_state: RuntimeControlSurfaceGraphState,
    /// Name of the backend provider supplying the device list.
    pub provider_name: String,
    /// Total number of discovered control surface devices.
    pub device_count: usize,
    /// Number of devices with a usable mapping posture.
    pub mapped_device_count: usize,
    /// Number of devices whose feedback output is ready.
    pub feedback_ready_device_count: usize,
    /// Number of devices in a guarded state.
    pub guarded_device_count: usize,
    /// Per-device descriptors for all discovered control surface devices.
    pub devices: Vec<RuntimeControlSurfaceDeviceDescriptor>,
    /// Human-readable summary of the snapshot.
    pub summary: String,
}

impl RuntimeControlSurfaceSnapshot {
    /// Returns an unavailable snapshot with all counts zeroed.
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

    /// Returns an empty snapshot for the given provider with discovery in the `Idle` state.
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

    /// Projects a control surface snapshot from a raw external MIDI endpoint graph snapshot.
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
