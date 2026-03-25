use super::*;

#[path = "channel_layout_data.rs"]
mod channel_layout_data;

use channel_layout_data::{multichannel_boundary_surfaces, multichannel_boundary_validation_steps};

pub(crate) fn render_multichannel_boundary_text() -> String {
    let mut rendered = format!(
        "multichannel_boundary: {MULTICHANNEL_BOUNDARY}\ncontract_path: {MULTICHANNEL_CONTRACT_PATH}\nacceptance_task: {MULTICHANNEL_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in multichannel_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in multichannel_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves canonical layout, channel-role, and bus-intent receipts through runtime, supervisor, and stable host-edge surfaces, but richer sidechain, spatial, and custom-layout execution depth still belongs to later work",
        "this closes the bounded multichannel consumer seam, not broader Linux device-matrix, time-stretch, or surround render-engine parity work",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_multichannel_boundary_json() -> String {
    let surfaces = multichannel_boundary_surfaces()
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
    let validation_steps = multichannel_boundary_validation_steps()
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
        "the shared boundary now proves canonical layout, channel-role, and bus-intent receipts through runtime, supervisor, and stable host-edge surfaces, but richer sidechain, spatial, and custom-layout execution depth still belongs to later work",
        "this closes the bounded multichannel consumer seam, not broader Linux device-matrix, time-stretch, or surround render-engine parity work",
    ]
    .iter()
    .map(|scope| json_string(scope))
    .collect::<Vec<_>>()
    .join(",");
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
        json_string(MULTICHANNEL_BOUNDARY),
        json_string(MULTICHANNEL_CONTRACT_PATH),
        json_string(MULTICHANNEL_ACCEPTANCE_TASK),
        multichannel_boundary_surfaces().len(),
        surfaces,
        multichannel_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
