use super::runtime_engine_state::RuntimePluginRenderedNodeState;
use crate::interfaces::*;
use signal_graph::GraphNodeTopologyRole;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn transport_session_provenance(
    intent: TransportAttachIntent,
) -> TransportSessionProvenance {
    match intent {
        TransportAttachIntent::SteadyState => TransportSessionProvenance::SteadyOrigin,
        TransportAttachIntent::RecoveryOverlap => TransportSessionProvenance::RecoveryReplacement,
    }
}

pub(crate) fn offline_render_plugin_override_status<'a>(
    latest: Option<&'a RuntimePluginRenderedNodeState>,
    bound_sandbox_id: Option<&String>,
    sandboxes: &BTreeMap<String, RuntimePluginSandboxSnapshot>,
    last_processing_epoch: Option<u64>,
    last_block_sequence: Option<u64>,
) -> (
    RuntimeOfflinePluginOverrideState,
    Option<&'a RuntimePluginRenderedNodeState>,
) {
    let Some(latest) = latest else {
        return (RuntimeOfflinePluginOverrideState::NotAvailable, None);
    };
    let fresh = Some(latest.processing_epoch) == last_processing_epoch
        && Some(latest.block_sequence) == last_block_sequence
        && bound_sandbox_id.is_none_or(|sandbox_id| sandbox_id == &latest.sandbox_id)
        && bound_sandbox_id
            .and_then(|sandbox_id| sandboxes.get(sandbox_id))
            .is_none_or(|sandbox| sandbox.state == RuntimePluginLifecycleState::Ready);
    if fresh {
        (
            RuntimeOfflinePluginOverrideState::FreshLatestBlock,
            Some(latest),
        )
    } else {
        (RuntimeOfflinePluginOverrideState::StaleLatestBlock, None)
    }
}

pub(crate) fn runtime_meter_source_role(role: GraphNodeTopologyRole) -> RuntimeMeterSourceRole {
    match role {
        GraphNodeTopologyRole::Utility => RuntimeMeterSourceRole::Utility,
        GraphNodeTopologyRole::TrackLane => RuntimeMeterSourceRole::TrackLane,
        GraphNodeTopologyRole::Bus => RuntimeMeterSourceRole::Bus,
        GraphNodeTopologyRole::Send => RuntimeMeterSourceRole::Send,
        GraphNodeTopologyRole::Return => RuntimeMeterSourceRole::Return,
        GraphNodeTopologyRole::ConsoleNode => RuntimeMeterSourceRole::ConsoleNode,
    }
}

pub(crate) fn unique_string<'a>(values: impl Iterator<Item = &'a String>) -> Option<String> {
    let mut values = values.cloned().collect::<BTreeSet<_>>().into_iter();
    let first = values.next()?;
    if values.next().is_none() {
        Some(first)
    } else {
        None
    }
}

pub(crate) fn sanitize_asset_id(asset_id: &str) -> String {
    asset_id
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => character,
            _ => '_',
        })
        .collect()
}
