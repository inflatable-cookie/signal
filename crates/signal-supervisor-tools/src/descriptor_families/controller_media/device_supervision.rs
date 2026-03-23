use super::*;

pub(crate) fn render_device_supervision_boundary_text() -> String {
    let mut rendered = format!(
        "device_supervision_boundary: {DEVICE_SUPERVISION_BOUNDARY}\ncontract_path: {DEVICE_SUPERVISION_CONTRACT_PATH}\nacceptance_task: {DEVICE_SUPERVISION_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in device_supervision_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in device_supervision_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared device supervision truth is now public, but broader backend-matrix breadth and endpoint-topology depth still remain later hardware work",
        "the current boundary proves recovering, exhausted, and faulted device outcomes on shared runtime and host edges, not product-local recovery UX or remote hardware orchestration",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_device_supervision_boundary_json() -> String {
    let surfaces = device_supervision_boundary_surfaces()
        .iter()
        .map(|surface| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"crate\":{},",
                    "\"surface\":{},",
                    "\"runtime_anchor\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(surface.id),
                json_string(surface.kind.label()),
                json_string(surface.crate_name),
                json_string(surface.surface),
                json_string(surface.runtime_anchor),
                json_string(surface.rationale)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = device_supervision_boundary_validation_steps()
        .iter()
        .map(|step| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(step.id),
                json_string(step.command),
                json_string(step.rationale)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let deferred_scope = [
        "shared device supervision truth is now public, but broader backend-matrix breadth and endpoint-topology depth still remain later hardware work",
        "the current boundary proves recovering, exhausted, and faulted device outcomes on shared runtime and host edges, not product-local recovery UX or remote hardware orchestration",
    ].iter().map(|scope| json_string(scope)).collect::<Vec<_>>().join(",");
    format!(
        concat!(
            "{{",
            "\"boundary\":{},",
            "\"contract_path\":{},",
            "\"acceptance_task\":{},",
            "\"surface_count\":{},",
            "\"surfaces\":[{}],",
            "\"validation_step_count\":{},",
            "\"validation_steps\":[{}],",
            "\"deferred_scope\":[{}]",
            "}}"
        ),
        json_string(DEVICE_SUPERVISION_BOUNDARY),
        json_string(DEVICE_SUPERVISION_CONTRACT_PATH),
        json_string(DEVICE_SUPERVISION_ACCEPTANCE_TASK),
        device_supervision_boundary_surfaces().len(),
        surfaces,
        device_supervision_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
