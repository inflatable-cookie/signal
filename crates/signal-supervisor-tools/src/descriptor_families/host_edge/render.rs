use super::data::{
    host_edge_surface_records, host_edge_validation_steps, HostEdgeStabilityTier,
};
use super::super::*;

pub(crate) fn render_host_edge_boundary_text() -> String {
    let mut rendered = format!(
        "host_edge_boundary: {HOST_EDGE_BOUNDARY}\ncontract_path: {HOST_EDGE_CONTRACT_PATH}\nacceptance_task: {HOST_EDGE_ACCEPTANCE_TASK}\nstable_surfaces:\n"
    );
    for surface in host_edge_surface_records()
        .iter()
        .filter(|surface| surface.tier == HostEdgeStabilityTier::Public)
    {
        rendered.push_str(&format!(
            "- id: {}\n  tier: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.tier.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("intentionally_unstable:\n");
    for surface in host_edge_surface_records()
        .iter()
        .filter(|surface| surface.tier != HostEdgeStabilityTier::Public)
    {
        rendered.push_str(&format!(
            "- id: {}\n  tier: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.tier.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in host_edge_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered
}

pub(crate) fn render_host_edge_boundary_json() -> String {
    let stable_surfaces = host_edge_surface_records()
        .iter()
        .filter(|surface| surface.tier == HostEdgeStabilityTier::Public)
        .map(|surface| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"tier\":{},",
                    "\"crate\":{},",
                    "\"surface\":{},",
                    "\"runtime_anchor\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(surface.id),
                json_string(surface.tier.label()),
                json_string(surface.crate_name),
                json_string(surface.surface),
                json_string(surface.runtime_anchor),
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let unstable_surfaces = host_edge_surface_records()
        .iter()
        .filter(|surface| surface.tier != HostEdgeStabilityTier::Public)
        .map(|surface| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"tier\":{},",
                    "\"crate\":{},",
                    "\"surface\":{},",
                    "\"runtime_anchor\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(surface.id),
                json_string(surface.tier.label()),
                json_string(surface.crate_name),
                json_string(surface.surface),
                json_string(surface.runtime_anchor),
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = host_edge_validation_steps()
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
    format!(
        concat!(
            "{{",
            "\"boundary\":{},",
            "\"contract_path\":{},",
            "\"acceptance_task\":{},",
            "\"stable_surfaces\":[{}],",
            "\"intentionally_unstable\":[{}],",
            "\"validation_steps\":[{}]",
            "}}"
        ),
        json_string(HOST_EDGE_BOUNDARY),
        json_string(HOST_EDGE_CONTRACT_PATH),
        json_string(HOST_EDGE_ACCEPTANCE_TASK),
        stable_surfaces,
        unstable_surfaces,
        validation_steps,
    )
}
