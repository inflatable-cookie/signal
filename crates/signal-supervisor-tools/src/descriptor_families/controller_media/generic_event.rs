use super::*;

pub(crate) fn render_generic_event_boundary_text() -> String {
    let mut rendered = format!(
        "generic_event_boundary: {GENERIC_EVENT_BOUNDARY}\ncontract_path: {GENERIC_EVENT_CONTRACT_PATH}\nacceptance_task: {GENERIC_EVENT_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in generic_event_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in generic_event_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared generic event truth is now consumable, but richer per-format packet families, SysEx, controller mapping, and editor semantics remain later work",
        "the current boundary proves bounded event and note-expression capability receipts through runtime and stable host edges, not full CLAP, VST3, and AU packet-model parity",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_generic_event_boundary_json() -> String {
    let surfaces = generic_event_boundary_surfaces()
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
    let validation_steps = generic_event_boundary_validation_steps()
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
        "shared generic event truth is now consumable, but richer per-format packet families, SysEx, controller mapping, and editor semantics remain later work",
        "the current boundary proves bounded event and note-expression capability receipts through runtime and stable host edges, not full CLAP, VST3, and AU packet-model parity",
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
        json_string(GENERIC_EVENT_BOUNDARY),
        json_string(GENERIC_EVENT_CONTRACT_PATH),
        json_string(GENERIC_EVENT_ACCEPTANCE_TASK),
        generic_event_boundary_surfaces().len(),
        surfaces,
        generic_event_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
