use super::*;

#[path = "interruption_recording_data.rs"]
mod interruption_recording_data;

use interruption_recording_data::{
    interruption_boundary_surfaces, interruption_boundary_validation_steps,
    recording_continuity_boundary_surfaces, recording_continuity_validation_steps,
};

pub(crate) fn render_interruption_boundary_text() -> String {
    let mut rendered = format!(
        "interruption_boundary: {INTERRUPTION_BOUNDARY}\ncontract_path: {INTERRUPTION_CONTRACT_PATH}\nacceptance_task: {INTERRUPTION_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in interruption_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in interruption_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "device-loss-specific truth is still stronger on broader host I/O surfaces than on a dedicated runtime-owned device-loss receipt",
        "full hardware and recovery milestone breadth remains deferred beyond this interruption taxonomy boundary",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_interruption_boundary_json() -> String {
    let surfaces = interruption_boundary_surfaces()
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
    let validation_steps = interruption_boundary_validation_steps()
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
        "device-loss-specific truth is still stronger on broader host I/O surfaces than on a dedicated runtime-owned device-loss receipt",
        "full hardware and recovery milestone breadth remains deferred beyond this interruption taxonomy boundary",
    ]
    .iter().map(|scope| json_string(scope)).collect::<Vec<_>>().join(",");
    format!(
        concat!(
            "{{",
            "\"boundary\":{},",
            "\"contract_path\":{},",
            "\"acceptance_task\":{},",
            "\"surfaces\":[{}],",
            "\"validation_steps\":[{}],",
            "\"deferred_scope\":[{}]",
            "}}"
        ),
        json_string(INTERRUPTION_BOUNDARY),
        json_string(INTERRUPTION_CONTRACT_PATH),
        json_string(INTERRUPTION_ACCEPTANCE_TASK),
        surfaces,
        validation_steps,
        deferred_scope
    )
}

pub(crate) fn render_recording_continuity_boundary_text() -> String {
    let mut rendered = format!(
        "recording_continuity_boundary: {RECORDING_CONTINUITY_BOUNDARY}\ncontract_path: {RECORDING_CONTINUITY_CONTRACT_PATH}\nacceptance_task: {RECORDING_CONTINUITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in recording_continuity_boundary_surfaces() {
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
    for step in recording_continuity_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "concrete MIDI capture and commit DTOs are still deferred, so the continuity family is typed but not yet format-complete",
        "same-identity resumable capture is currently proven through safe-mode degradation rather than a richer dedicated capture pause or resume API",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_recording_continuity_boundary_json() -> String {
    let surfaces = recording_continuity_boundary_surfaces()
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
    let validation_steps = recording_continuity_validation_steps()
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
        "concrete MIDI capture and commit DTOs are still deferred, so the continuity family is typed but not yet format-complete",
        "same-identity resumable capture is currently proven through safe-mode degradation rather than a richer dedicated capture pause or resume API",
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
        json_string(RECORDING_CONTINUITY_BOUNDARY),
        json_string(RECORDING_CONTINUITY_CONTRACT_PATH),
        json_string(RECORDING_CONTINUITY_ACCEPTANCE_TASK),
        recording_continuity_boundary_surfaces().len(),
        surfaces,
        recording_continuity_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
