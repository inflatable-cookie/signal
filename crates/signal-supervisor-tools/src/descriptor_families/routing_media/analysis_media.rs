use super::*;

pub(crate) fn render_media_service_boundary_text() -> String {
    let mut rendered = format!(
        "media_service_boundary: {MEDIA_SERVICE_BOUNDARY}\ncontract_path: {MEDIA_SERVICE_CONTRACT_PATH}\nacceptance_task: {MEDIA_SERVICE_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in media_service_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in media_service_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned media indexing, waveform readiness, preview state, and invalidation receipts, but richer metadata extraction and broader library-service depth still belong to later g06.018 work",
        "this closes the bounded consumer seam for shared media-service state, not product-local browser, collection, or editorial media-management workflows",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_media_service_boundary_json() -> String {
    let surfaces = media_service_boundary_surfaces()
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
    let validation_steps = media_service_boundary_validation_steps()
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
        "the shared boundary now proves runtime-owned media indexing, waveform readiness, preview state, and invalidation receipts, but richer metadata extraction and broader library-service depth still belong to later g06.018 work",
        "this closes the bounded consumer seam for shared media-service state, not product-local browser, collection, or editorial media-management workflows",
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
        json_string(MEDIA_SERVICE_BOUNDARY),
        json_string(MEDIA_SERVICE_CONTRACT_PATH),
        json_string(MEDIA_SERVICE_ACCEPTANCE_TASK),
        media_service_boundary_surfaces().len(),
        surfaces,
        media_service_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}

pub(crate) fn render_analysis_metadata_boundary_text() -> String {
    let mut rendered = format!(
        "analysis_metadata_boundary: {ANALYSIS_METADATA_BOUNDARY}\ncontract_path: {ANALYSIS_METADATA_CONTRACT_PATH}\nacceptance_task: {ANALYSIS_METADATA_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in analysis_metadata_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in analysis_metadata_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned reusable loudness and character descriptors plus explicit deferred-family coverage, but broader rhythm, tonal, and embedding payload depth still belongs to later work",
        "this closes the bounded consumer seam for analysis-metadata and library-service truth, not product-local browser, collection, tagging, or recommendation workflows",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_analysis_metadata_boundary_json() -> String {
    let surfaces = analysis_metadata_boundary_surfaces()
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
    let validation_steps = analysis_metadata_boundary_validation_steps()
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
        "the shared boundary now proves runtime-owned reusable loudness and character descriptors plus explicit deferred-family coverage, but broader rhythm, tonal, and embedding payload depth still belongs to later work",
        "this closes the bounded consumer seam for analysis-metadata and library-service truth, not product-local browser, collection, tagging, or recommendation workflows",
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
        json_string(ANALYSIS_METADATA_BOUNDARY),
        json_string(ANALYSIS_METADATA_CONTRACT_PATH),
        json_string(ANALYSIS_METADATA_ACCEPTANCE_TASK),
        analysis_metadata_boundary_surfaces().len(),
        surfaces,
        analysis_metadata_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
