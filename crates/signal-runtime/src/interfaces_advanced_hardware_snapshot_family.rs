use crate::{
    RuntimeAdvancedHardwareGraphState, RuntimeAdvancedHardwareSnapshot,
    RuntimeControlSurfaceGraphState, RuntimeControlSurfaceSnapshot,
    RuntimeExternalMidiDiscoveryState,
};

#[path = "interfaces_advanced_hardware_snapshot_family/device_projection.rs"]
mod device_projection;

impl RuntimeAdvancedHardwareSnapshot {
    pub fn unavailable() -> Self {
        Self {
            discovery_state: RuntimeExternalMidiDiscoveryState::Unavailable,
            graph_state: RuntimeAdvancedHardwareGraphState::Unavailable,
            provider_name: "runtime-unavailable".into(),
            device_count: 0,
            portable_device_count: 0,
            guarded_device_count: 0,
            context_only_device_count: 0,
            denied_device_count: 0,
            feedback_channel_device_count: 0,
            display_transport_device_count: 0,
            motor_transport_device_count: 0,
            haptic_transport_device_count: 0,
            scene_mapping_device_count: 0,
            feedback_page_device_count: 0,
            safe_action_graph_device_count: 0,
            devices: Vec::new(),
            summary: "discovery=Unavailable graph=Unavailable provider=runtime-unavailable devices=0 portable=0 guarded=0 context-only=0 denied=0 feedback-channels=0 display-transport=0 motor-transport=0 haptic-transport=0 scene-mapping=0 feedback-pages=0 safe-action-graphs=0".into(),
        }
    }

    pub fn empty(provider_name: impl Into<String>) -> Self {
        let provider_name = provider_name.into();
        Self {
            discovery_state: RuntimeExternalMidiDiscoveryState::Idle,
            graph_state: RuntimeAdvancedHardwareGraphState::Empty,
            provider_name: provider_name.clone(),
            device_count: 0,
            portable_device_count: 0,
            guarded_device_count: 0,
            context_only_device_count: 0,
            denied_device_count: 0,
            feedback_channel_device_count: 0,
            display_transport_device_count: 0,
            motor_transport_device_count: 0,
            haptic_transport_device_count: 0,
            scene_mapping_device_count: 0,
            feedback_page_device_count: 0,
            safe_action_graph_device_count: 0,
            devices: Vec::new(),
            summary: format!(
                "discovery=Idle graph=Empty provider={} devices=0 portable=0 guarded=0 context-only=0 denied=0 feedback-channels=0 display-transport=0 motor-transport=0 haptic-transport=0 scene-mapping=0 feedback-pages=0 safe-action-graphs=0",
                provider_name
            ),
        }
    }

    pub fn from_control_surface_snapshot(snapshot: &RuntimeControlSurfaceSnapshot) -> Self {
        if matches!(
            snapshot.discovery_state,
            RuntimeExternalMidiDiscoveryState::Unavailable
        ) || matches!(
            snapshot.graph_state,
            RuntimeControlSurfaceGraphState::Unavailable
        ) {
            return Self::unavailable();
        }
        if snapshot.device_count == 0 {
            return Self::empty(snapshot.provider_name.clone());
        }

        device_projection::project(snapshot)
    }
}
