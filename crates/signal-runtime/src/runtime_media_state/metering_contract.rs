use super::*;
use crate::runtime::runtime_utils::{runtime_meter_source_role, unique_string};

impl RuntimeMeteringStateModel {
    pub(crate) fn meter_contract_metadata(
        contract: &signal_graph::GraphContractSummary,
        bus_id: &str,
    ) -> (
        RuntimeMeterSourceRole,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Vec<String>,
    ) {
        let matching = contract
            .node_contracts
            .iter()
            .filter(|node| node.output_bus_id == bus_id)
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return (
                RuntimeMeterSourceRole::Utility,
                None,
                None,
                None,
                None,
                Vec::new(),
            );
        }

        let producer_node_ids = matching
            .iter()
            .map(|node| node.node_id.clone())
            .collect::<Vec<_>>();
        let topology_role = {
            let first = matching[0].topology_role;
            if matching.iter().all(|node| node.topology_role == first) {
                runtime_meter_source_role(first)
            } else {
                RuntimeMeterSourceRole::Utility
            }
        };
        let track_lane_id = unique_string(
            matching
                .iter()
                .filter_map(|node| node.track_lane_id.as_ref()),
        );
        let bus_group_id = unique_string(
            matching
                .iter()
                .filter_map(|node| node.bus_group_id.as_ref()),
        );
        let console_group_id = unique_string(
            matching
                .iter()
                .filter_map(|node| node.console_group_id.as_ref()),
        );
        let send_return_id = unique_string(
            matching
                .iter()
                .filter_map(|node| node.send_return_id.as_ref()),
        );
        (
            topology_role,
            track_lane_id,
            bus_group_id,
            console_group_id,
            send_return_id,
            producer_node_ids,
        )
    }
}
