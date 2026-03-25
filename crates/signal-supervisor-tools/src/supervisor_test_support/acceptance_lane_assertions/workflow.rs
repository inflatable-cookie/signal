pub(crate) fn assert_control_preview_workflow_acceptance_lane_text(rendered: &str) {
    for expected in [
        "control_preview_workflow_acceptance_lane: signal.runtime.control-preview-workflow-acceptance-lane",
        "acceptance_task: effigy acceptance:control-preview-workflow-acceptance-lane",
        "contract_path: docs/contracts/069-control-surface-and-preview-workflow-acceptance-contract.md",
        "- effigy acceptance:advanced-hardware-boundary",
        "- effigy acceptance:preview-transform-boundary",
        "title: Control-Surface Workflow Coherence",
        "title: Preview Workflow Coherence",
        "title: Cross-Surface Workflow Coherence",
        "id: cross-family-export-proof",
        "cargo test -p signal-supervisor-tools export_json_carries_cross_family_control_preview_workflow_acceptance_evidence",
        "id: lane-descriptor-proof",
        "id: required-lane-task",
        "cargo run -p signal-supervisor-tools -- --describe-control-preview-workflow-acceptance-lane --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_control_preview_workflow_acceptance_lane_json(rendered: &str) {
    for expected in [
        "\"lane\":\"signal.runtime.control-preview-workflow-acceptance-lane\"",
        "\"contract_path\":\"docs/contracts/069-control-surface-and-preview-workflow-acceptance-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:control-preview-workflow-acceptance-lane\"",
        "\"required_task_count\":2",
        "\"advisory_task_count\":0",
        "\"id\":\"control-surface-workflow-coherence\"",
        "\"id\":\"preview-workflow-coherence\"",
        "\"id\":\"cross-surface-workflow-coherence\"",
        "\"id\":\"cross-family-export-proof\"",
        "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_control_preview_workflow_acceptance_evidence\"",
        "\"id\":\"lane-descriptor-proof\"",
        "\"command\":\"cargo test -p signal-supervisor-tools control_preview_workflow_acceptance_lane_json_reports_required_and_deferred_policy\"",
        "\"id\":\"required-lane-task\"",
        "\"command\":\"effigy acceptance:control-preview-workflow-acceptance-lane\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_integrated_live_workflow_acceptance_lane_text(rendered: &str) {
    for expected in [
        "integrated_live_workflow_acceptance_lane: signal.runtime.integrated-live-ownership-and-workflow-acceptance-lane",
        "acceptance_task: effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane",
        "contract_path: docs/contracts/070-integrated-live-ownership-and-workflow-acceptance-contract.md",
        "- effigy acceptance:linux-live-acceptance-lane",
        "- effigy acceptance:device-workflow-acceptance-lane",
        "- effigy acceptance:immersive-acceptance-lane",
        "- effigy acceptance:control-preview-workflow-acceptance-lane",
        "title: Linux Live And Device Workflow Continuity",
        "title: Immersive And Preview Workflow Continuity",
        "title: Cross-Surface Integrated Coherence",
        "title: Shared Grouped Integrated Acceptance Export",
        "id: cross-family-export-proof",
        "cargo test -p signal-supervisor-tools export_json_carries_cross_family_integrated_live_workflow_acceptance_evidence",
        "id: lane-descriptor-proof",
        "id: required-lane-task",
        "cargo run -p signal-supervisor-tools -- --describe-integrated-live-workflow-acceptance-lane --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_integrated_live_workflow_acceptance_lane_json(rendered: &str) {
    for expected in [
        "\"lane\":\"signal.runtime.integrated-live-ownership-and-workflow-acceptance-lane\"",
        "\"contract_path\":\"docs/contracts/070-integrated-live-ownership-and-workflow-acceptance-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane\"",
        "\"required_task_count\":4",
        "\"advisory_task_count\":0",
        "\"id\":\"linux-live-and-device-workflow-continuity\"",
        "\"id\":\"immersive-and-preview-workflow-continuity\"",
        "\"id\":\"cross-surface-integrated-coherence\"",
        "\"id\":\"shared-grouped-integrated-acceptance-export\"",
        "\"id\":\"cross-family-export-proof\"",
        "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_integrated_live_workflow_acceptance_evidence\"",
        "\"id\":\"lane-descriptor-proof\"",
        "\"command\":\"cargo test -p signal-supervisor-tools integrated_live_workflow_acceptance_lane_json_reports_required_and_deferred_policy\"",
        "\"id\":\"required-lane-task\"",
        "\"command\":\"effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_g06_soak_lane_text(rendered: &str) {
    for expected in [
        "g06_soak_lane: signal.g06.long-session-soak-lane",
        "acceptance_task: effigy acceptance:g06-soak-lane",
        "id: required-local-soak-export",
        "status: required",
        "id: deferred-server-soak-export",
        "status: deferred",
        "id: g06-soak-lane-task",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_g06_soak_lane_json(rendered: &str) {
    for expected in [
        "\"lane\":\"signal.g06.long-session-soak-lane\"",
        "\"contract_path\":\"docs/contracts/031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:g06-soak-lane\"",
        "\"id\":\"required-local-soak-export\"",
        "\"status\":\"required\"",
        "\"id\":\"deferred-server-soak-export\"",
        "\"status\":\"deferred\"",
        "\"id\":\"g06-soak-lane-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}
