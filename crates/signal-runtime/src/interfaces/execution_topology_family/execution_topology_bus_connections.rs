use super::super::*;

#[derive(Clone)]
struct RuntimePlannedGraphNodeTopologyEndpoint<'a> {
    node_id: &'a str,
    topology_role: GraphNodeTopologyRole,
    input_bus_id: &'a str,
    output_bus_id: &'a str,
    input_bus_intent: RuntimeBusIntent,
    output_bus_intent: RuntimeBusIntent,
    bus_group_id: Option<&'a str>,
    send_return_id: Option<&'a str>,
}

fn runtime_auxiliary_path_for_connection(
    source: &RuntimePlannedGraphNodeTopologyEndpoint<'_>,
    target: &RuntimePlannedGraphNodeTopologyEndpoint<'_>,
) -> Option<(
    RuntimeAuxiliaryPathKind,
    String,
    RuntimeBusRole,
    RuntimeBusIntent,
)> {
    if let Some(send_return_id) = source.send_return_id.or(target.send_return_id) {
        return Some((
            RuntimeAuxiliaryPathKind::SendReturn,
            format!("send_return:{send_return_id}"),
            RuntimeBusRole::AuxSend,
            RuntimeBusIntent::AuxSend,
        ));
    }
    if let Some(bus_group_id) = source.bus_group_id.or(target.bus_group_id) {
        return Some((
            RuntimeAuxiliaryPathKind::Submix,
            format!("bus_group:{bus_group_id}"),
            RuntimeBusRole::Submix,
            RuntimeBusIntent::MainProgram,
        ));
    }
    let source_role = runtime_bus_role_for_endpoint(source.topology_role, source.output_bus_intent);
    let target_role = runtime_bus_role_for_endpoint(target.topology_role, target.input_bus_intent);
    if source_role == RuntimeBusRole::AnalysisTap || target_role == RuntimeBusRole::AnalysisTap {
        return Some((
            RuntimeAuxiliaryPathKind::Analysis,
            format!("analysis:{}", source.output_bus_id),
            RuntimeBusRole::AnalysisTap,
            RuntimeBusIntent::AnalysisTap,
        ));
    }
    None
}

pub fn derive_runtime_bus_connections(
    planned_nodes: &[RuntimePlannedGraphNode],
) -> (
    Vec<RuntimeBusConnectionSummary>,
    Vec<RuntimeAuxiliaryPathSummary>,
) {
    let mut producers_by_bus =
        std::collections::BTreeMap::<&str, Vec<RuntimePlannedGraphNodeTopologyEndpoint<'_>>>::new();
    for node in planned_nodes {
        producers_by_bus
            .entry(node.output_bus_id.as_str())
            .or_default()
            .push(RuntimePlannedGraphNodeTopologyEndpoint {
                node_id: node.node_id.as_str(),
                topology_role: node.topology_role,
                input_bus_id: node.input_bus_id.as_str(),
                output_bus_id: node.output_bus_id.as_str(),
                input_bus_intent: node.input_bus_intent,
                output_bus_intent: node.output_bus_intent,
                bus_group_id: node.bus_group_id.as_deref(),
                send_return_id: node.send_return_id.as_deref(),
            });
    }

    let mut connections = Vec::new();
    let mut auxiliary_paths =
        std::collections::BTreeMap::<String, RuntimeAuxiliaryPathSummary>::new();

    for node in planned_nodes {
        let Some(producers) = producers_by_bus.get(node.input_bus_id.as_str()) else {
            continue;
        };
        let target = RuntimePlannedGraphNodeTopologyEndpoint {
            node_id: node.node_id.as_str(),
            topology_role: node.topology_role,
            input_bus_id: node.input_bus_id.as_str(),
            output_bus_id: node.output_bus_id.as_str(),
            input_bus_intent: node.input_bus_intent,
            output_bus_intent: node.output_bus_intent,
            bus_group_id: node.bus_group_id.as_deref(),
            send_return_id: node.send_return_id.as_deref(),
        };
        for source in producers {
            let auxiliary_path = runtime_auxiliary_path_for_connection(source, &target);
            let source_bus_role =
                runtime_bus_role_for_endpoint(source.topology_role, source.output_bus_intent);
            let target_bus_role =
                runtime_bus_role_for_endpoint(target.topology_role, target.input_bus_intent);
            let connection_id = format!(
                "{}:{}->{}:{}",
                source.node_id, source.output_bus_id, target.node_id, target.input_bus_id
            );
            let attachment_class = RuntimeBusConnectionAttachmentClass::Required;
            let fallback_outcome = RuntimeBusConnectionFallbackOutcome::NoFallback;
            let summary = format!(
                "connection={} source={}:{}/{:?} target={}:{}/{:?} path={:?} attachment={:?} fallback={:?}",
                connection_id,
                source.node_id,
                source.output_bus_id,
                source_bus_role,
                target.node_id,
                target.input_bus_id,
                target_bus_role,
                auxiliary_path.as_ref().map(|(kind, path_id, _, _)| format!("{kind:?}:{path_id}")),
                attachment_class,
                fallback_outcome,
            );
            connections.push(RuntimeBusConnectionSummary {
                connection_id: connection_id.clone(),
                source_node_id: source.node_id.into(),
                source_bus_id: source.output_bus_id.into(),
                source_bus_role,
                target_node_id: target.node_id.into(),
                target_bus_id: target.input_bus_id.into(),
                target_bus_role,
                auxiliary_path_kind: auxiliary_path.as_ref().map(|(kind, _, _, _)| *kind),
                auxiliary_path_id: auxiliary_path
                    .as_ref()
                    .map(|(_, path_id, _, _)| path_id.clone()),
                attachment_class,
                fallback_outcome,
                summary,
            });

            if let Some((path_kind, auxiliary_path_id, bus_role, material_bus_intent)) =
                auxiliary_path
            {
                let path = auxiliary_paths
                    .entry(auxiliary_path_id.clone())
                    .or_insert_with(|| RuntimeAuxiliaryPathSummary {
                        auxiliary_path_id: auxiliary_path_id.clone(),
                        path_kind,
                        bus_role,
                        material_bus_intent,
                        source_node_ids: Vec::new(),
                        target_node_ids: Vec::new(),
                        bus_ids: Vec::new(),
                        connection_ids: Vec::new(),
                        attachment_class,
                        fallback_outcome,
                        summary: String::new(),
                    });
                if !path.source_node_ids.contains(&source.node_id.to_string()) {
                    path.source_node_ids.push(source.node_id.to_string());
                }
                if !path.target_node_ids.contains(&target.node_id.to_string()) {
                    path.target_node_ids.push(target.node_id.to_string());
                }
                if !path.bus_ids.contains(&source.output_bus_id.to_string()) {
                    path.bus_ids.push(source.output_bus_id.to_string());
                }
                if !path.bus_ids.contains(&target.input_bus_id.to_string()) {
                    path.bus_ids.push(target.input_bus_id.to_string());
                }
                if !path.connection_ids.contains(&connection_id) {
                    path.connection_ids.push(connection_id.clone());
                }
            }
        }
    }

    let mut auxiliary_paths = auxiliary_paths.into_values().collect::<Vec<_>>();
    for path in &mut auxiliary_paths {
        path.summary = format!(
            "path={} kind={:?} role={:?} material={:?} sources={:?} targets={:?} buses={:?} connections={} attachment={:?} fallback={:?}",
            path.auxiliary_path_id,
            path.path_kind,
            path.bus_role,
            path.material_bus_intent,
            path.source_node_ids,
            path.target_node_ids,
            path.bus_ids,
            path.connection_ids.len(),
            path.attachment_class,
            path.fallback_outcome,
        );
    }

    (connections, auxiliary_paths)
}
