pub(crate) fn assert_integrated_acceptance_lane_text(rendered: &str) {
    for expected in [
        "integrated_acceptance_lane: signal.runtime.integrated-acceptance-lane",
        "acceptance_task: effigy acceptance:integrated-acceptance-lane",
        "- effigy acceptance:interruption-boundary",
        "- effigy acceptance:analysis-metadata-boundary",
        "- effigy acceptance:recording-continuity",
        "- effigy acceptance:vst3-boundary",
        "title: Adapter And Portability Breadth",
        "id: cross-family-export-proof",
        "cargo run -p signal-supervisor-tools -- --describe-integrated-acceptance-lane --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_integrated_acceptance_lane_json(rendered: &str) {
    for expected in [
        "\"lane\":\"signal.runtime.integrated-acceptance-lane\"",
        "\"contract_path\":\"docs/contracts/030-fault-injection-harness-and-multi-backend-acceptance-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:integrated-acceptance-lane\"",
        "\"required_task_count\":11",
        "\"advisory_task_count\":6",
        "\"id\":\"recovery-and-fault-attribution\"",
        "\"id\":\"adapter-and-portability-breadth\"",
        "\"id\":\"cross-family-export-proof\"",
        "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_integrated_acceptance_evidence\"",
        "\"command\":\"effigy acceptance:integrated-acceptance-lane\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_g07_acceptance_lane_text(rendered: &str) {
    for expected in [
        "g07_acceptance_lane: signal.runtime.g07-integrated-acceptance-lane",
        "acceptance_task: effigy acceptance:g07-integrated-acceptance-lane",
        "- effigy acceptance:multichannel-boundary",
        "- effigy acceptance:preview-transform-boundary",
        "- effigy acceptance:complex-io-boundary",
        "- effigy acceptance:lv2-boundary",
        "title: Linux Plugin And Backend Continuity",
        "id: cross-family-export-proof",
        "id: required-lane-task",
        "cargo run -p signal-supervisor-tools -- --describe-g07-acceptance-lane --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_g07_acceptance_lane_json(rendered: &str) {
    for expected in [
        "\"lane\":\"signal.runtime.g07-integrated-acceptance-lane\"",
        "\"contract_path\":\"docs/contracts/050-multichannel-linux-time-stretch-and-control-surface-acceptance-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:g07-integrated-acceptance-lane\"",
        "\"required_task_count\":15",
        "\"advisory_task_count\":2",
        "\"id\":\"routing-and-multichannel-coherence\"",
        "\"id\":\"linux-plugin-and-backend-continuity\"",
        "\"id\":\"external-control-and-advanced-hardware\"",
        "\"id\":\"stretch-analysis-artifact-and-preview\"",
        "\"id\":\"cross-family-export-proof\"",
        "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_g07_acceptance_evidence\"",
        "\"command\":\"effigy acceptance:g07-integrated-acceptance-lane\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_device_workflow_acceptance_lane_text(rendered: &str) {
    for expected in [
        "device_workflow_acceptance_lane: signal.runtime.device-workflow-acceptance-lane",
        "acceptance_task: effigy acceptance:device-workflow-acceptance-lane",
        "contract_path: docs/contracts/066-cross-backend-device-protocol-and-live-workflow-acceptance-contract.md",
        "- effigy acceptance:external-midi-boundary",
        "- effigy acceptance:advanced-hardware-boundary",
        "title: Live Endpoint Ownership And Protocol Continuity",
        "title: Control-Surface And Advanced Hardware Workflow",
        "title: Cross-Backend Host-Edge Coherence",
        "id: cross-family-export-proof",
        "id: required-lane-task",
        "cargo run -p signal-supervisor-tools -- --describe-device-workflow-acceptance-lane --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_device_workflow_acceptance_lane_json(rendered: &str) {
    for expected in [
        "\"lane\":\"signal.runtime.device-workflow-acceptance-lane\"",
        "\"contract_path\":\"docs/contracts/066-cross-backend-device-protocol-and-live-workflow-acceptance-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:device-workflow-acceptance-lane\"",
        "\"required_task_count\":4",
        "\"advisory_task_count\":0",
        "\"id\":\"live-endpoint-ownership-and-protocol-continuity\"",
        "\"id\":\"control-surface-and-advanced-hardware-workflow\"",
        "\"id\":\"cross-backend-host-edge-coherence\"",
        "\"id\":\"cross-family-export-proof\"",
        "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_device_workflow_acceptance_evidence\"",
        "\"id\":\"required-lane-task\"",
        "\"command\":\"effigy acceptance:device-workflow-acceptance-lane\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_linux_live_acceptance_lane_text(rendered: &str) {
    for expected in [
        "linux_live_acceptance_lane: signal.runtime.linux-live-acceptance-lane",
        "acceptance_task: effigy acceptance:linux-live-acceptance-lane",
        "contract_path: docs/contracts/067-live-linux-backend-acceptance-and-failure-injection-contract.md",
        "- effigy acceptance:linux-live-ownership-boundary",
        "- effigy acceptance:jack-coordination-boundary",
        "- effigy acceptance:pipewire-alsa-parity-boundary",
        "- effigy acceptance:linux-backend-clock-topology-boundary",
        "title: Live Ownership And Guarded Continuity",
        "title: Backend-Native Coordination And Parity",
        "title: Cross-Backend Host-Edge Coherence",
        "id: cross-family-export-proof",
        "cargo test -p signal-supervisor-tools export_json_carries_cross_family_linux_live_acceptance_evidence",
        "id: required-lane-task",
        "cargo run -p signal-supervisor-tools -- --describe-linux-live-acceptance-lane --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_linux_live_acceptance_lane_json(rendered: &str) {
    for expected in [
        "\"lane\":\"signal.runtime.linux-live-acceptance-lane\"",
        "\"contract_path\":\"docs/contracts/067-live-linux-backend-acceptance-and-failure-injection-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:linux-live-acceptance-lane\"",
        "\"required_task_count\":4",
        "\"advisory_task_count\":0",
        "\"id\":\"live-ownership-and-guarded-continuity\"",
        "\"id\":\"backend-native-coordination-and-parity\"",
        "\"id\":\"cross-backend-host-edge-coherence\"",
        "\"id\":\"cross-family-export-proof\"",
        "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_linux_live_acceptance_evidence\"",
        "\"id\":\"required-lane-task\"",
        "\"command\":\"effigy acceptance:linux-live-acceptance-lane\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_immersive_acceptance_lane_text(rendered: &str) {
    for expected in [
        "immersive_acceptance_lane: signal.runtime.immersive-acceptance-lane",
        "acceptance_task: effigy acceptance:immersive-acceptance-lane",
        "contract_path: docs/contracts/068-immersive-render-and-monitoring-acceptance-contract.md",
        "- effigy acceptance:spatial-boundary",
        "title: Room-Policy And Render Continuity",
        "title: Deployment Fold-Down And Monitoring Coherence",
        "title: Cross-Surface Immersive Coherence",
        "id: cross-family-export-proof",
        "cargo test -p signal-supervisor-tools export_json_carries_cross_family_immersive_acceptance_evidence",
        "id: required-lane-task",
        "cargo run -p signal-supervisor-tools -- --describe-immersive-acceptance-lane --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_immersive_acceptance_lane_json(rendered: &str) {
    for expected in [
        "\"lane\":\"signal.runtime.immersive-acceptance-lane\"",
        "\"contract_path\":\"docs/contracts/068-immersive-render-and-monitoring-acceptance-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:immersive-acceptance-lane\"",
        "\"required_task_count\":1",
        "\"advisory_task_count\":0",
        "\"id\":\"room-policy-and-render-continuity\"",
        "\"id\":\"deployment-fold-down-and-monitoring-coherence\"",
        "\"id\":\"cross-surface-immersive-coherence\"",
        "\"id\":\"cross-family-export-proof\"",
        "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_immersive_acceptance_evidence\"",
        "\"id\":\"required-lane-task\"",
        "\"command\":\"effigy acceptance:immersive-acceptance-lane\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}
