use super::*;

pub(crate) fn render_controller_expression_boundary_text() -> String {
    let mut rendered = format!(
        "controller_expression_boundary: {CONTROLLER_EXPRESSION_BOUNDARY}\ncontract_path: {CONTROLLER_EXPRESSION_CONTRACT_PATH}\nacceptance_task: {CONTROLLER_EXPRESSION_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in controller_expression_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in controller_expression_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves bounded richer controller-expression posture, not full MIDI 2.0 UMP transport, negotiation, or profile exchange depth",
        "the current seam keeps runtime-owned MPE and MIDI 2.0 posture consumable through runtime and stable host edges, but control-surface mapping and feedback semantics remain later work",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_controller_expression_boundary_json() -> String {
    let surfaces = controller_expression_boundary_surfaces()
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
    let validation_steps = controller_expression_boundary_validation_steps()
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
        "the shared boundary now proves bounded richer controller-expression posture, not full MIDI 2.0 UMP transport, negotiation, or profile exchange depth",
        "the current seam keeps runtime-owned MPE and MIDI 2.0 posture consumable through runtime and stable host edges, but control-surface mapping and feedback semantics remain later work",
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
        json_string(CONTROLLER_EXPRESSION_BOUNDARY),
        json_string(CONTROLLER_EXPRESSION_CONTRACT_PATH),
        json_string(CONTROLLER_EXPRESSION_ACCEPTANCE_TASK),
        controller_expression_boundary_surfaces().len(),
        surfaces,
        controller_expression_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
