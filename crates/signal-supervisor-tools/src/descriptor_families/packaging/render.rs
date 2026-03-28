use super::super::*;
use super::data::{
    packaging_manifest_inputs, packaging_manifest_unsupported_paths,
    packaging_manifest_validation_steps, packaging_receipt_surfaces,
};

pub(crate) fn render_packaging_manifest_text() -> String {
    let mut rendered = format!(
        "packaging_manifest: {PACKAGING_MANIFEST}\nrelease_version: {}\nversion_source: {RELEASE_VERSION_SOURCE}\ncontract_path: {PACKAGING_MANIFEST_CONTRACT_PATH}\nacceptance_task: {PACKAGING_MANIFEST_ACCEPTANCE_TASK}\ninputs:\n",
        env!("CARGO_PKG_VERSION")
    );
    for input in packaging_manifest_inputs() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  path_or_command: {}\n  rationale: {}\n",
            input.id,
            input.kind.label(),
            input.path_or_command,
            input.rationale,
        ));
    }
    rendered.push_str("receipt_surfaces:\n");
    for receipt in packaging_receipt_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  surface: {}\n  rationale: {}\n",
            receipt.id, receipt.surface, receipt.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in packaging_manifest_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("unsupported_publication_paths:\n");
    for scope in packaging_manifest_unsupported_paths() {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_packaging_manifest_json() -> String {
    let inputs = packaging_manifest_inputs()
        .iter()
        .map(|input| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"path_or_command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(input.id),
                json_string(input.kind.label()),
                json_string(input.path_or_command),
                json_string(input.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let receipts = packaging_receipt_surfaces()
        .iter()
        .map(|receipt| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"surface\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(receipt.id),
                json_string(receipt.surface),
                json_string(receipt.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = packaging_manifest_validation_steps()
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
    let unsupported = packaging_manifest_unsupported_paths()
        .iter()
        .map(|scope| json_string(scope))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"manifest\":{},",
            "\"release_version\":{},",
            "\"version_source\":{},",
            "\"contract_path\":{},",
            "\"acceptance_task\":{},",
            "\"inputs\":[{}],",
            "\"receipt_surfaces\":[{}],",
            "\"validation_steps\":[{}],",
            "\"unsupported_publication_paths\":[{}]",
            "}}"
        ),
        json_string(PACKAGING_MANIFEST),
        json_string(env!("CARGO_PKG_VERSION")),
        json_string(RELEASE_VERSION_SOURCE),
        json_string(PACKAGING_MANIFEST_CONTRACT_PATH),
        json_string(PACKAGING_MANIFEST_ACCEPTANCE_TASK),
        inputs,
        receipts,
        validation_steps,
        unsupported,
    )
}
