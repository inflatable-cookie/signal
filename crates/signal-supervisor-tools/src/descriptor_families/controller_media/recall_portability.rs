use super::*;

pub(crate) fn render_recall_portability_boundary_text() -> String {
    let mut rendered = format!(
        "recall_portability_boundary: {RECALL_PORTABILITY_BOUNDARY}\ncontract_path: {RECALL_PORTABILITY_CONTRACT_PATH}\nacceptance_task: {RECALL_PORTABILITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in recall_portability_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in recall_portability_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared preset-state portability truth is now consumable, but lossless cross-adapter preset interchange, richer preset families, and adapter-native document models remain later work",
        "the current boundary proves bounded ARA document, source, and region context transfer through runtime and stable host edges, not fuller ARA editor workflow or persistent product document semantics",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_recall_portability_boundary_json() -> String {
    let surfaces = recall_portability_boundary_surfaces()
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
    let validation_steps = recall_portability_boundary_validation_steps()
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
        "shared preset-state portability truth is now consumable, but lossless cross-adapter preset interchange, richer preset families, and adapter-native document models remain later work",
        "the current boundary proves bounded ARA document, source, and region context transfer through runtime and stable host edges, not fuller ARA editor workflow or persistent product document semantics",
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
        json_string(RECALL_PORTABILITY_BOUNDARY),
        json_string(RECALL_PORTABILITY_CONTRACT_PATH),
        json_string(RECALL_PORTABILITY_ACCEPTANCE_TASK),
        recall_portability_boundary_surfaces().len(),
        surfaces,
        recall_portability_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
