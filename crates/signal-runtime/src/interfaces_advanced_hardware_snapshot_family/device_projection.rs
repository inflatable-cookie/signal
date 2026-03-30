use crate::{
    RuntimeAdvancedHardwareGraphState, RuntimeAdvancedHardwareSnapshot,
    RuntimeControlSurfaceSnapshot,
};

#[path = "device_projection/device_descriptor.rs"]
mod device_descriptor;

use device_descriptor::{project_device, DeviceProjectionCounts};

pub(super) fn project(snapshot: &RuntimeControlSurfaceSnapshot) -> RuntimeAdvancedHardwareSnapshot {
    let mut devices = Vec::with_capacity(snapshot.devices.len());
    let mut counts = DeviceProjectionCounts::default();

    for device in &snapshot.devices {
        let projection = project_device(device);
        counts.accumulate(&projection.counts);
        devices.push(projection.descriptor);
    }

    let graph_state = if counts.guarded_device_count > 0
        || counts.context_only_device_count > 0
        || counts.denied_device_count > 0
    {
        RuntimeAdvancedHardwareGraphState::Guarded
    } else {
        RuntimeAdvancedHardwareGraphState::Ready
    };

    let device_count = devices.len();
    let summary = format!(
        "discovery={:?} graph={:?} provider={} devices={} portable={} guarded={} context-only={} denied={} feedback-channels={} display-transport={} motor-transport={} haptic-transport={} scene-mapping={} feedback-pages={} safe-action-graphs={}",
        snapshot.discovery_state,
        graph_state,
        snapshot.provider_name,
        device_count,
        counts.portable_device_count,
        counts.guarded_device_count,
        counts.context_only_device_count,
        counts.denied_device_count,
        counts.feedback_channel_device_count,
        counts.display_transport_device_count,
        counts.motor_transport_device_count,
        counts.haptic_transport_device_count,
        counts.scene_mapping_device_count,
        counts.feedback_page_device_count,
        counts.safe_action_graph_device_count
    );

    RuntimeAdvancedHardwareSnapshot {
        discovery_state: snapshot.discovery_state,
        graph_state,
        provider_name: snapshot.provider_name.clone(),
        device_count,
        portable_device_count: counts.portable_device_count,
        guarded_device_count: counts.guarded_device_count,
        context_only_device_count: counts.context_only_device_count,
        denied_device_count: counts.denied_device_count,
        feedback_channel_device_count: counts.feedback_channel_device_count,
        display_transport_device_count: counts.display_transport_device_count,
        motor_transport_device_count: counts.motor_transport_device_count,
        haptic_transport_device_count: counts.haptic_transport_device_count,
        scene_mapping_device_count: counts.scene_mapping_device_count,
        feedback_page_device_count: counts.feedback_page_device_count,
        safe_action_graph_device_count: counts.safe_action_graph_device_count,
        devices,
        summary,
    }
}
