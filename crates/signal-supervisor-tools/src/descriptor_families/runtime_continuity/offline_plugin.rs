use super::*;

pub(crate) fn render_offline_render_continuity_boundary_text() -> String {
    let mut rendered = format!(
        "offline_render_continuity_boundary: {OFFLINE_RENDER_CONTINUITY_BOUNDARY}\ncontract_path: {OFFLINE_RENDER_CONTINUITY_CONTRACT_PATH}\nacceptance_task: {OFFLINE_RENDER_CONTINUITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in offline_render_continuity_boundary_surfaces() {
        let kind = match surface.kind {
            OfflineRenderContinuityBoundarySurfaceKind::RuntimeSnapshot => "runtime-snapshot",
            OfflineRenderContinuityBoundarySurfaceKind::RuntimeReport => "runtime-report",
            OfflineRenderContinuityBoundarySurfaceKind::HostEdge => "host-edge",
        };
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, kind, surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in offline_render_continuity_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "restart-survival across full process restart still needs later deeper render recovery work beyond the current runtime stop/restart proof",
        "dedicated durable queue ownership and remote render job orchestration remain out of scope for this continuity boundary",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_offline_render_continuity_boundary_json() -> String {
    let surfaces = offline_render_continuity_boundary_surfaces()
        .iter()
        .map(|surface| {
            let kind = match surface.kind {
                OfflineRenderContinuityBoundarySurfaceKind::RuntimeSnapshot => "runtime-snapshot",
                OfflineRenderContinuityBoundarySurfaceKind::RuntimeReport => "runtime-report",
                OfflineRenderContinuityBoundarySurfaceKind::HostEdge => "host-edge",
            };
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
                json_string(kind),
                json_string(surface.crate_name),
                json_string(surface.surface),
                json_string(surface.runtime_anchor),
                json_string(surface.rationale)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = offline_render_continuity_validation_steps()
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
        "restart-survival across full process restart still needs later deeper render recovery work beyond the current runtime stop/restart proof",
        "dedicated durable queue ownership and remote render job orchestration remain out of scope for this continuity boundary",
    ]
    .iter().map(|scope| json_string(scope)).collect::<Vec<_>>().join(",");
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
        json_string(OFFLINE_RENDER_CONTINUITY_BOUNDARY),
        json_string(OFFLINE_RENDER_CONTINUITY_CONTRACT_PATH),
        json_string(OFFLINE_RENDER_CONTINUITY_ACCEPTANCE_TASK),
        offline_render_continuity_boundary_surfaces().len(),
        surfaces,
        offline_render_continuity_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}

pub(crate) fn render_plugin_continuity_boundary_text() -> String {
    let mut rendered = format!(
        "plugin_continuity_boundary: {PLUGIN_CONTINUITY_BOUNDARY}\ncontract_path: {PLUGIN_CONTINUITY_CONTRACT_PATH}\nacceptance_task: {PLUGIN_CONTINUITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in plugin_continuity_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in plugin_continuity_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared-boundary blast radius is now explicit, but dedicated blast-radius DTOs are still deferred beyond the current lifecycle and chain receipts",
        "the exercised proof path is still sandbox-first, so deeper in-process parity and broader adapter transport tuning remain later plugin-format work",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_plugin_continuity_boundary_json() -> String {
    let surfaces = plugin_continuity_boundary_surfaces()
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
    let validation_steps = plugin_continuity_validation_steps()
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
        "shared-boundary blast radius is now explicit, but dedicated blast-radius DTOs are still deferred beyond the current lifecycle and chain receipts",
        "the exercised proof path is still sandbox-first, so deeper in-process parity and broader adapter transport tuning remain later plugin-format work",
    ]
    .iter().map(|scope| json_string(scope)).collect::<Vec<_>>().join(",");
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
        json_string(PLUGIN_CONTINUITY_BOUNDARY),
        json_string(PLUGIN_CONTINUITY_CONTRACT_PATH),
        json_string(PLUGIN_CONTINUITY_ACCEPTANCE_TASK),
        plugin_continuity_boundary_surfaces().len(),
        surfaces,
        plugin_continuity_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
