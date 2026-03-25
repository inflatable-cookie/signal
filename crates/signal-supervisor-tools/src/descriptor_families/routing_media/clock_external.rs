use super::*;

#[path = "clock_external_data.rs"]
mod clock_external_data;

use clock_external_data::{
    clock_topology_boundary_surfaces, clock_topology_boundary_validation_steps,
    external_io_boundary_surfaces, external_io_boundary_validation_steps,
};

pub(crate) fn render_clock_topology_boundary_text() -> String {
    let mut rendered = format!(
        "clock_topology_boundary: {CLOCK_TOPOLOGY_BOUNDARY}\ncontract_path: {CLOCK_TOPOLOGY_CONTRACT_PATH}\nacceptance_task: {CLOCK_TOPOLOGY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in clock_topology_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in clock_topology_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned drift, discontinuity, duplex-mismatch, and endpoint-topology meaning, but broader external-I/O, monitoring, and loopback depth still belongs to g06.016",
        "the stable local host edge exposes live host-io receipts directly, while the stable server host edge still omits that live clocking seam and remains outside this focused boundary",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_clock_topology_boundary_json() -> String {
    let surfaces = clock_topology_boundary_surfaces()
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
    let validation_steps = clock_topology_boundary_validation_steps()
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
        "the shared boundary now proves runtime-owned drift, discontinuity, duplex-mismatch, and endpoint-topology meaning, but broader external-I/O, monitoring, and loopback depth still belongs to g06.016",
        "the stable local host edge exposes live host-io receipts directly, while the stable server host edge still omits that live clocking seam and remains outside this focused boundary",
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
        json_string(CLOCK_TOPOLOGY_BOUNDARY),
        json_string(CLOCK_TOPOLOGY_CONTRACT_PATH),
        json_string(CLOCK_TOPOLOGY_ACCEPTANCE_TASK),
        clock_topology_boundary_surfaces().len(),
        surfaces,
        clock_topology_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}

pub(crate) fn render_external_io_boundary_text() -> String {
    let mut rendered = format!(
        "external_io_boundary: {EXTERNAL_IO_BOUNDARY}\ncontract_path: {EXTERNAL_IO_CONTRACT_PATH}\nacceptance_task: {EXTERNAL_IO_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in external_io_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in external_io_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned external-I/O role, monitor state, tap-point, and bounded loopback meaning, but richer measurement-session and calibration workflows still belong to later g06.016 and media-service work",
        "the stable server host edge currently proves explicit unavailable monitoring and loopback state rather than a live server-host hardware seam, so broader live server-side external-I/O depth remains deferred",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_external_io_boundary_json() -> String {
    let surfaces = external_io_boundary_surfaces()
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
    let validation_steps = external_io_boundary_validation_steps()
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
        "the shared boundary now proves runtime-owned external-I/O role, monitor state, tap-point, and bounded loopback meaning, but richer measurement-session and calibration workflows still belong to later g06.016 and media-service work",
        "the stable server host edge currently proves explicit unavailable monitoring and loopback state rather than a live server-host hardware seam, so broader live server-side external-I/O depth remains deferred",
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
        json_string(EXTERNAL_IO_BOUNDARY),
        json_string(EXTERNAL_IO_CONTRACT_PATH),
        json_string(EXTERNAL_IO_ACCEPTANCE_TASK),
        external_io_boundary_surfaces().len(),
        surfaces,
        external_io_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
