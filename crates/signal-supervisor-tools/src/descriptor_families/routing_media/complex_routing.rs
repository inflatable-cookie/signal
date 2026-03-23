use super::*;

pub(crate) fn render_complex_io_boundary_text() -> String {
    let mut rendered = format!(
        "complex_io_boundary: {COMPLEX_IO_BOUNDARY}\ncontract_path: {COMPLEX_IO_CONTRACT_PATH}\nacceptance_task: {COMPLEX_IO_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in complex_io_boundary_surfaces() {
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
    for step in complex_io_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned complex plugin-I/O, pin-matrix, and dynamic bus-negotiation receipts through runtime, supervisor, and stable host-edge surfaces, but broader spatial routing, immersive buses, and product pin-matrix policy still belongs to later work",
        "this closes the bounded pin-matrix and complex plugin-I/O consumer seam, not deeper adapter-private negotiation breadth, full format-specific pin schemas, or richer product-local mixer assignment workflows",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_complex_io_boundary_json() -> String {
    let surfaces = complex_io_boundary_surfaces()
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
    let validation_steps = complex_io_boundary_validation_steps()
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
        "the shared boundary now proves runtime-owned complex plugin-I/O, pin-matrix, and dynamic bus-negotiation receipts through runtime, supervisor, and stable host-edge surfaces, but broader spatial routing, immersive buses, and product pin-matrix policy still belongs to later work",
        "this closes the bounded pin-matrix and complex plugin-I/O consumer seam, not deeper adapter-private negotiation breadth, full format-specific pin schemas, or richer product-local mixer assignment workflows",
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
        json_string(COMPLEX_IO_BOUNDARY),
        json_string(COMPLEX_IO_CONTRACT_PATH),
        json_string(COMPLEX_IO_ACCEPTANCE_TASK),
        complex_io_boundary_surfaces().len(),
        surfaces,
        complex_io_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
