use super::super::*;
use super::data::{spatial_boundary_surfaces, spatial_boundary_validation_steps};

pub(crate) fn render_spatial_boundary_text() -> String {
    let mut rendered = format!(
        "spatial_boundary: {SPATIAL_BOUNDARY}\ncontract_path: {SPATIAL_CONTRACT_PATH}\nacceptance_task: {SPATIAL_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in spatial_boundary_surfaces() {
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
    for step in spatial_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned richer spatial, deployment-monitoring, and bounded renderer or immersive-export receipts through runtime, supervisor, and stable host-edge surfaces, but true renderer-backed object execution and monitoring-scene breadth still belong to later g08 work",
        "this closes the bounded renderer-capability and immersive-export consumer seam, not renderer-vendor package schemas, publication workflows, or product-local immersive export UX",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_spatial_boundary_json() -> String {
    let surfaces = spatial_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = spatial_boundary_validation_steps()
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
                json_string(step.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let deferred_scope = [
        "the shared boundary now proves runtime-owned richer spatial, deployment-monitoring, and bounded renderer or immersive-export receipts through runtime, supervisor, and stable host-edge surfaces, but true renderer-backed object execution and monitoring-scene breadth still belong to later g08 work",
        "this closes the bounded renderer-capability and immersive-export consumer seam, not renderer-vendor package schemas, publication workflows, or product-local immersive export UX",
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
        json_string(SPATIAL_BOUNDARY),
        json_string(SPATIAL_CONTRACT_PATH),
        json_string(SPATIAL_ACCEPTANCE_TASK),
        spatial_boundary_surfaces().len(),
        surfaces,
        spatial_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
