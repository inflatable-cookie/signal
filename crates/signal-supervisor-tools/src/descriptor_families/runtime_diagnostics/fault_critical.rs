use super::*;

pub(crate) fn render_fault_diagnostic_boundary_text() -> String {
    let mut rendered = format!(
        "fault_diagnostic_boundary: {FAULT_DIAGNOSTIC_BOUNDARY}\ncontract_path: {FAULT_DIAGNOSTIC_CONTRACT_PATH}\nacceptance_task: {FAULT_DIAGNOSTIC_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in fault_diagnostic_boundary_surfaces() {
        rendered.push_str(&format!("- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n", surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale));
    }
    rendered.push_str("validation_steps:\n");
    for step in fault_diagnostic_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "callback pressure remains advisory host evidence rather than a stronger canonical runtime family",
        "per-event traces, remote diagnostics pipelines, and product-specific diagnostic UX remain out of scope for this shared boundary",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_fault_diagnostic_boundary_json() -> String {
    let surfaces = fault_diagnostic_boundary_surfaces()
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
    let validation_steps = fault_diagnostic_boundary_validation_steps()
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
        "callback pressure remains advisory host evidence rather than a stronger canonical runtime family",
        "per-event traces, remote diagnostics pipelines, and product-specific diagnostic UX remain out of scope for this shared boundary",
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
        json_string(FAULT_DIAGNOSTIC_BOUNDARY),
        json_string(FAULT_DIAGNOSTIC_CONTRACT_PATH),
        json_string(FAULT_DIAGNOSTIC_ACCEPTANCE_TASK),
        fault_diagnostic_boundary_surfaces().len(),
        surfaces,
        fault_diagnostic_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}

pub(crate) fn render_critical_path_boundary_text() -> String {
    let mut rendered = format!(
        "critical_path_boundary: {CRITICAL_PATH_BOUNDARY}\ncontract_path: {CRITICAL_PATH_CONTRACT_PATH}\nacceptance_task: {CRITICAL_PATH_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in critical_path_boundary_surfaces() {
        rendered.push_str(&format!("- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n", surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale));
    }
    rendered.push_str("validation_steps:\n");
    for step in critical_path_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "deeper scheduler attribution beyond the current bounded hot-node, hot-group, and critical-path lane receipts remains deferred to later profiling work",
        "node-by-node elapsed-time traces, flamegraph exports, and host thread telemetry remain outside this bounded consumer surface",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_critical_path_boundary_json() -> String {
    let surfaces = critical_path_boundary_surfaces()
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
    let validation_steps = critical_path_boundary_validation_steps()
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
        "deeper scheduler attribution beyond the current bounded hot-node, hot-group, and critical-path lane receipts remains deferred to later profiling work",
        "node-by-node elapsed-time traces, flamegraph exports, and host thread telemetry remain outside this bounded consumer surface",
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
        json_string(CRITICAL_PATH_BOUNDARY),
        json_string(CRITICAL_PATH_CONTRACT_PATH),
        json_string(CRITICAL_PATH_ACCEPTANCE_TASK),
        critical_path_boundary_surfaces().len(),
        surfaces,
        critical_path_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
