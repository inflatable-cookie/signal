pub(crate) const HOST_EDGE_BOUNDARY: &str = "signal.host.edge.boundary";
pub(crate) const HOST_EDGE_CONTRACT_PATH: &str =
    "docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md";
pub(crate) const HOST_EDGE_ACCEPTANCE_TASK: &str = "effigy acceptance:host-edge-consumer";
pub(crate) const RELEASE_BOUNDARY: &str = "signal.release.boundary";
pub(crate) const RELEASE_VERSION_SOURCE: &str = "workspace.package.version";
pub(crate) const RELEASE_CHANGELOG_PATH: &str = "CHANGELOG.md";
pub(crate) const RELEASE_CONFORMANCE_TASK: &str = "effigy acceptance:conformance";
pub(crate) const PACKAGING_MANIFEST: &str = "signal.release.packaging-manifest";
pub(crate) const PACKAGING_MANIFEST_CONTRACT_PATH: &str =
    "docs/contracts/010-publication-grade-packaging-manifest-and-release-receipt-contract.md";
pub(crate) const PACKAGING_MANIFEST_ACCEPTANCE_TASK: &str =
    "effigy acceptance:release-packaging-consumer";
pub(crate) const DOWNSTREAM_AUTOMATION_BOUNDARY: &str = "signal.downstream.automation";
pub(crate) const DOWNSTREAM_AUTOMATION_CONTRACT_PATH: &str =
    "docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md";
pub(crate) const DOWNSTREAM_AUTOMATION_MANDATORY_TASK: &str =
    "effigy acceptance:downstream-release";
pub(crate) const DOWNSTREAM_AUTOMATION_OPTIONAL_TASK: &str = "effigy acceptance:downstream-depth";
pub(crate) const DOWNSTREAM_AUTOMATION_COMBINED_TASK: &str =
    "effigy acceptance:downstream-automation";
pub(crate) const DOWNSTREAM_FAIL_GATES: &str = "signal.downstream.fail-gates";
pub(crate) const DOWNSTREAM_FAIL_GATE_TASK: &str = "effigy acceptance:downstream-gate";
pub(crate) const GENERATION_CLOSEOUT: &str = "signal.generation.closeout";
pub(crate) const GENERATION_CLOSEOUT_GENERATION: &str = "g08";
pub(crate) const GENERATION_CLOSEOUT_TASK: &str = "effigy acceptance:g08-closeout";
pub(crate) const GENERATION_CLOSEOUT_CONTRACT_PATH: &str =
    "docs/contracts/071-generation-closeout-and-downstream-workflow-readiness-gate-contract.md";
pub(crate) const GENERATION_CLOSEOUT_ROADMAP_PATH: &str =
    "docs/roadmaps/g08/020-generation-closeout-and-downstream-workflow-readiness-gate.md";
pub(crate) const GENERATION_CLOSEOUT_BACKLOG_PATH: &str =
    "docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md";
pub(crate) const G08_INTEGRATED_ACCEPTANCE_LANE_COMMAND: &str =
    "cargo run -p signal-supervisor-tools -- --describe-integrated-live-workflow-acceptance-lane --format=json";
pub(crate) const GENERATION_CLOSEOUT_NEXT_QUEUE_PATH: &str = GENERATION_CLOSEOUT_BACKLOG_PATH;
pub(crate) const GENERATION_CLOSEOUT_GATE_STATUS: &str = "complete";
pub(crate) const GENERATION_CLOSEOUT_NEXT_QUEUE_STATUS: &str = "backlog";
pub(crate) const GENERATION_CLOSEOUT_PROMOTION_DECISION: &str =
    "close-g08-and-handoff-to-post-g08-backlog";
