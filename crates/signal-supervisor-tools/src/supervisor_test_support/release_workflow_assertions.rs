pub(crate) fn assert_host_edge_boundary_text(rendered: &str) {
    for expected in [
        "host_edge_boundary: signal.host.edge.boundary",
        "acceptance_task: effigy acceptance:host-edge-consumer",
        "surface: RuntimeSupervisorApi implemented by both hosts",
        "surface: supervisor_report() -> RuntimeSupervisorReport",
        "tier: consumer-facing-but-unstable",
        "surface: boot_* fault, recovery, watchdog, and soak helpers",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_host_edge_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.host.edge.boundary\"",
        "\"contract_path\":\"docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:host-edge-consumer\"",
        "\"id\":\"shared-runtime-supervisor-api\"",
        "\"id\":\"shared-supervisor-report\"",
        "\"id\":\"host-summary-dtos\"",
        "\"tier\":\"scenario-only\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_release_boundary_text(rendered: &str) {
    for expected in [
        "release_boundary: signal.release.boundary",
        "release_version: 0.1.0",
        "version_source: workspace.package.version",
        "changelog_path: CHANGELOG.md",
        "conformance_task: effigy acceptance:conformance",
        "cargo run -p signal-supervisor-tools -- --describe-export --format=json",
        "cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json",
        "publication packaging beyond the repo-owned manifest descriptor and receipt inventory",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_release_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.release.boundary\"",
        "\"release_version\":\"0.1.0\"",
        "\"version_source\":\"workspace.package.version\"",
        "\"changelog_path\":\"CHANGELOG.md\"",
        "\"conformance_task\":\"effigy acceptance:conformance\"",
        "\"id\":\"workspace-changelog\"",
        "\"id\":\"consumer-conformance\"",
        "\"id\":\"supervisor-export-description\"",
        "\"id\":\"publication-packaging-manifest\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_packaging_manifest_text(rendered: &str) {
    for expected in [
        "packaging_manifest: signal.release.packaging-manifest",
        "release_version: 0.1.0",
        "contract_path: docs/contracts/010-publication-grade-packaging-manifest-and-release-receipt-contract.md",
        "acceptance_task: effigy acceptance:release-packaging-consumer",
        "cargo run -p signal-supervisor-tools -- --describe-host-edge-boundary --format=json",
        "id: manifest-generation-receipt",
        "id: validation-receipt",
        "crates.io publication and registry upload automation",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_packaging_manifest_json(rendered: &str) {
    for expected in [
        "\"manifest\":\"signal.release.packaging-manifest\"",
        "\"release_version\":\"0.1.0\"",
        "\"contract_path\":\"docs/contracts/010-publication-grade-packaging-manifest-and-release-receipt-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:release-packaging-consumer\"",
        "\"id\":\"release-boundary-descriptor\"",
        "\"id\":\"manifest-generation-receipt\"",
        "\"id\":\"validation-receipt\"",
        "\"id\":\"release-boundary-baseline\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_downstream_automation_text(rendered: &str) {
    for expected in [
        "downstream_automation_boundary: signal.downstream.automation",
        "mandatory_release_task: effigy acceptance:downstream-release",
        "optional_depth_task: effigy acceptance:downstream-depth",
        "id: release-packaging-consumer",
        "id: local-mixed-watchdog-export",
        "signal.supervisor.export JSON with profiling_receipt, soak_receipt, and supervisor_report",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_downstream_automation_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.downstream.automation\"",
        "\"mandatory_release_task\":\"effigy acceptance:downstream-release\"",
        "\"optional_depth_task\":\"effigy acceptance:downstream-depth\"",
        "\"combined_task\":\"effigy acceptance:downstream-automation\"",
        "\"id\":\"downstream-automation-descriptor\"",
        "\"id\":\"local-soak-export\"",
        "\"id\":\"analysis-acceptance\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_downstream_fail_gates_text(rendered: &str) {
    for expected in [
        "downstream_fail_gates: signal.downstream.fail-gates",
        "fail_gate_task: effigy acceptance:downstream-gate",
        "id: mandatory-release-gate",
        "blocks_release: true",
        "id: optional-depth-lane",
        "blocks_release: false",
        "id: server-soak-export",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_downstream_fail_gates_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.downstream.fail-gates\"",
        "\"fail_gate_task\":\"effigy acceptance:downstream-gate\"",
        "\"id\":\"mandatory-release-gate\"",
        "\"blocks_release\":true",
        "\"id\":\"optional-depth-lane\"",
        "\"blocks_release\":false",
        "\"id\":\"server-soak-export\"",
        "\"status\":\"deferred\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_generation_closeout_text(rendered: &str) {
    for expected in [
        "generation_closeout: signal.generation.closeout",
        "generation: g08",
        "contract_path: docs/contracts/071-generation-closeout-and-downstream-workflow-readiness-gate-contract.md",
        "roadmap_path: docs/roadmaps/g08/020-generation-closeout-and-downstream-workflow-readiness-gate.md",
        "closeout_task: effigy acceptance:g08-closeout",
        "promotion_decision: close-g08-and-handoff-to-post-g08-backlog",
        "closeout_gate_status: complete",
        "cargo run -p signal-supervisor-tools -- --describe-integrated-live-workflow-acceptance-lane --format=json",
        "next_queue_path: docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md",
        "next_queue_status: backlog",
        "id: linux-live-and-guarded-ownership-surface",
        "status: sufficient-for-closeout",
        "g08 is closed.",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_generation_closeout_json(rendered: &str) {
    for expected in [
        "\"closeout\":\"signal.generation.closeout\"",
        "\"generation\":\"g08\"",
        "\"contract_path\":\"docs/contracts/071-generation-closeout-and-downstream-workflow-readiness-gate-contract.md\"",
        "\"roadmap_path\":\"docs/roadmaps/g08/020-generation-closeout-and-downstream-workflow-readiness-gate.md\"",
        "\"closeout_task\":\"effigy acceptance:g08-closeout\"",
        "\"promotion_decision\":\"close-g08-and-handoff-to-post-g08-backlog\"",
        "\"closeout_gate_status\":\"complete\"",
        "\"g08_integrated_acceptance_lane_command\":\"cargo run -p signal-supervisor-tools -- --describe-integrated-live-workflow-acceptance-lane --format=json\"",
        "\"next_queue_path\":\"docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md\"",
        "\"next_queue_status\":\"backlog\"",
        "\"id\":\"integrated-acceptance-base\"",
        "\"id\":\"closeout-descriptor-proof\"",
        "\"id\":\"generation-closeout-description\"",
        "\"id\":\"linux-live-and-guarded-ownership-surface\"",
        "\"status\":\"sufficient-for-closeout\"",
        "\"broader repeated-run and environment-specific acceptance depth remain outside the bounded g08 closeout fast path and are now explicit post-g08 backlog work instead of implied follow-up\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}
