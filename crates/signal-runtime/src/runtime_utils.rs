use crate::interfaces::*;
use signal_graph::GraphNodeTopologyRole;
use std::collections::BTreeSet;

pub(crate) fn transport_session_provenance(
    intent: TransportAttachIntent,
) -> TransportSessionProvenance {
    match intent {
        TransportAttachIntent::SteadyState => TransportSessionProvenance::SteadyOrigin,
        TransportAttachIntent::RecoveryOverlap => TransportSessionProvenance::RecoveryReplacement,
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
