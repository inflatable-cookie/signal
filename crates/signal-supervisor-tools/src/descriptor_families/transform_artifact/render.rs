use super::super::*;
use super::data::{
    transform_artifact_boundary_surfaces, transform_artifact_boundary_validation_steps,
};

pub(crate) fn render_transform_artifact_boundary_text() -> String {
    let mut rendered = format!(
        "transform_artifact_boundary: {TRANSFORM_ARTIFACT_BOUNDARY}\ncontract_path: {TRANSFORM_ARTIFACT_CONTRACT_PATH}\nacceptance_task: {TRANSFORM_ARTIFACT_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in transform_artifact_boundary_surfaces() {
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
    for step in transform_artifact_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned post-warp render, cache readiness, invalidation, reuse, persistence, retention, and cache-placement receipts through runtime, supervisor, clip-render, offline preview, and stable host-edge surfaces, but fuller session persistence UX, cloud sync, quota, and eviction depth still belongs to later g08 work",
        "this closes the bounded transform-artifact and transform-persistence consumer seam, not a full cache engine, browser storage ledger, or product-local transform management workflow",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_transform_artifact_boundary_json() -> String {
    let surfaces = transform_artifact_boundary_surfaces()
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
    let validation_steps = transform_artifact_boundary_validation_steps()
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
        "the shared boundary now proves runtime-owned post-warp render, cache readiness, invalidation, reuse, persistence, retention, and cache-placement receipts through runtime, supervisor, clip-render, offline preview, and stable host-edge surfaces, but fuller session persistence UX, cloud sync, quota, and eviction depth still belongs to later g08 work",
        "this closes the bounded transform-artifact and transform-persistence consumer seam, not a full cache engine, browser storage ledger, or product-local transform management workflow",
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
        json_string(TRANSFORM_ARTIFACT_BOUNDARY),
        json_string(TRANSFORM_ARTIFACT_CONTRACT_PATH),
        json_string(TRANSFORM_ARTIFACT_ACCEPTANCE_TASK),
        transform_artifact_boundary_surfaces().len(),
        surfaces,
        transform_artifact_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
