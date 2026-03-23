use super::*;

pub(crate) fn render_block_timing_boundary_text() -> String {
    let mut rendered = format!(
        "block_timing_boundary: {BLOCK_TIMING_BOUNDARY}\ncontract_path: {BLOCK_TIMING_CONTRACT_PATH}\nacceptance_task: {BLOCK_TIMING_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in block_timing_boundary_surfaces() {
        rendered.push_str(&format!("- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n", surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale));
    }
    rendered.push_str("validation_steps:\n");
    for step in block_timing_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "critical-path, hot-node, and worker-lane attribution are still deferred to g06.007 instead of being inferred from block timing alone",
        "host callback cadence remains advisory evidence and does not outrank the runtime-owned per-block timing seam",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_block_timing_boundary_json() -> String {
    let surfaces = block_timing_boundary_surfaces()
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
    let validation_steps = block_timing_boundary_validation_steps()
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
        "critical-path, hot-node, and worker-lane attribution are still deferred to g06.007 instead of being inferred from block timing alone",
        "host callback cadence remains advisory evidence and does not outrank the runtime-owned per-block timing seam",
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
        json_string(BLOCK_TIMING_BOUNDARY),
        json_string(BLOCK_TIMING_CONTRACT_PATH),
        json_string(BLOCK_TIMING_ACCEPTANCE_TASK),
        block_timing_boundary_surfaces().len(),
        surfaces,
        block_timing_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}

pub(crate) fn render_deferred_work_policy_boundary_text() -> String {
    let mut rendered = format!(
        "deferred_work_policy_boundary: {DEFERRED_WORK_POLICY_BOUNDARY}\ncontract_path: {DEFERRED_WORK_POLICY_CONTRACT_PATH}\nacceptance_task: {DEFERRED_WORK_POLICY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in deferred_work_policy_boundary_surfaces() {
        rendered.push_str(&format!("- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n", surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale));
    }
    rendered.push_str("validation_steps:\n");
    for step in deferred_work_policy_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "consumer-facing proof is limited to the current bounded deferred-service family rather than a generic future job scheduler",
        "distributed or remote deferred-work ownership remains deferred beyond this shared local runtime policy boundary",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_deferred_work_policy_boundary_json() -> String {
    let surfaces = deferred_work_policy_boundary_surfaces()
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
    let validation_steps = deferred_work_policy_boundary_validation_steps()
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
        "consumer-facing proof is limited to the current bounded deferred-service family rather than a generic future job scheduler",
        "distributed or remote deferred-work ownership remains deferred beyond this shared local runtime policy boundary",
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
        json_string(DEFERRED_WORK_POLICY_BOUNDARY),
        json_string(DEFERRED_WORK_POLICY_CONTRACT_PATH),
        json_string(DEFERRED_WORK_POLICY_ACCEPTANCE_TASK),
        deferred_work_policy_boundary_surfaces().len(),
        surfaces,
        deferred_work_policy_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
