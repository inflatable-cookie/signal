use super::*;

#[path = "bus_routing_data.rs"]
mod bus_routing_data;

use bus_routing_data::{
    multi_bus_boundary_surfaces, multi_bus_boundary_validation_steps, sidechain_boundary_surfaces,
    sidechain_boundary_validation_steps,
};

pub(crate) fn render_multi_bus_boundary_text() -> String {
    let mut rendered = format!(
        "multi_bus_boundary: {MULTI_BUS_BOUNDARY}\ncontract_path: {MULTI_BUS_CONTRACT_PATH}\nacceptance_task: {MULTI_BUS_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in multi_bus_boundary_surfaces() {
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
    for step in multi_bus_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned multi-bus connection, auxiliary-path, and fallback receipts through runtime, supervisor, and stable host-edge surfaces, but broader complex plugin-I/O, spatial routing, and immersive bus breadth still belongs to later work",
        "this closes the bounded multi-bus consumer seam, not final plugin-format-specific bus negotiation, product-local routing UX, or spatial renderer policy",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_multi_bus_boundary_json() -> String {
    let surfaces = multi_bus_boundary_surfaces()
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
    let validation_steps = multi_bus_boundary_validation_steps()
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
        "the shared boundary now proves runtime-owned multi-bus connection, auxiliary-path, and fallback receipts through runtime, supervisor, and stable host-edge surfaces, but broader complex plugin-I/O, spatial routing, and immersive bus breadth still belongs to later work",
        "this closes the bounded multi-bus consumer seam, not final plugin-format-specific bus negotiation, product-local routing UX, or spatial renderer policy",
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
        json_string(MULTI_BUS_BOUNDARY),
        json_string(MULTI_BUS_CONTRACT_PATH),
        json_string(MULTI_BUS_ACCEPTANCE_TASK),
        multi_bus_boundary_surfaces().len(),
        surfaces,
        multi_bus_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}

pub(crate) fn render_sidechain_boundary_text() -> String {
    let mut rendered = format!(
        "sidechain_boundary: {SIDECHAIN_BOUNDARY}\ncontract_path: {SIDECHAIN_CONTRACT_PATH}\nacceptance_task: {SIDECHAIN_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in sidechain_boundary_surfaces() {
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
    for step in sidechain_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned sidechain source, target, attachment policy, and fallback receipts through runtime, supervisor, and stable host-edge surfaces, but broader multi-bus, complex plugin-I/O, and spatial routing breadth still belongs to later work",
        "this closes the bounded sidechain consumer seam, not richer auxiliary topology, plugin-format-specific bus attachment breadth, or product-local routing UX",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_sidechain_boundary_json() -> String {
    let surfaces = sidechain_boundary_surfaces()
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
    let validation_steps = sidechain_boundary_validation_steps()
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
        "the shared boundary now proves runtime-owned sidechain source, target, attachment policy, and fallback receipts through runtime, supervisor, and stable host-edge surfaces, but broader multi-bus, complex plugin-I/O, and spatial routing breadth still belongs to later work",
        "this closes the bounded sidechain consumer seam, not richer auxiliary topology, plugin-format-specific bus attachment breadth, or product-local routing UX",
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
        json_string(SIDECHAIN_BOUNDARY),
        json_string(SIDECHAIN_CONTRACT_PATH),
        json_string(SIDECHAIN_ACCEPTANCE_TASK),
        sidechain_boundary_surfaces().len(),
        surfaces,
        sidechain_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
