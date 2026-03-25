use super::{assert_rejects_positionals, assert_supports_describe};
use crate::CliMode;

#[test]
fn parse_args_supports_acceptance_and_release_modes() {
    assert_supports_describe(
        "--describe-integrated-acceptance-lane",
        CliMode::DescribeIntegratedAcceptanceLane,
    );
    assert_supports_describe(
        "--describe-g07-acceptance-lane",
        CliMode::DescribeG07AcceptanceLane,
    );
    assert_supports_describe(
        "--describe-device-workflow-acceptance-lane",
        CliMode::DescribeDeviceWorkflowAcceptanceLane,
    );
    assert_supports_describe(
        "--describe-linux-live-acceptance-lane",
        CliMode::DescribeLinuxLiveAcceptanceLane,
    );
    assert_supports_describe(
        "--describe-immersive-acceptance-lane",
        CliMode::DescribeImmersiveAcceptanceLane,
    );
    assert_supports_describe(
        "--describe-control-preview-workflow-acceptance-lane",
        CliMode::DescribeControlPreviewWorkflowAcceptanceLane,
    );
    assert_supports_describe(
        "--describe-integrated-live-workflow-acceptance-lane",
        CliMode::DescribeIntegratedLiveWorkflowAcceptanceLane,
    );
    assert_supports_describe("--describe-g06-soak-lane", CliMode::DescribeG06SoakLane);
    assert_supports_describe(
        "--describe-release-boundary",
        CliMode::DescribeReleaseBoundary,
    );
    assert_supports_describe(
        "--describe-packaging-manifest",
        CliMode::DescribePackagingManifest,
    );
    assert_supports_describe(
        "--describe-downstream-automation",
        CliMode::DescribeDownstreamAutomation,
    );
    assert_supports_describe(
        "--describe-downstream-fail-gates",
        CliMode::DescribeDownstreamFailGates,
    );
    assert_supports_describe(
        "--describe-generation-closeout",
        CliMode::DescribeGenerationCloseout,
    );
}

#[test]
fn parse_args_rejects_positionals_with_acceptance_and_release_modes() {
    assert_rejects_positionals("--describe-integrated-acceptance-lane");
    assert_rejects_positionals("--describe-g07-acceptance-lane");
    assert_rejects_positionals("--describe-device-workflow-acceptance-lane");
    assert_rejects_positionals("--describe-linux-live-acceptance-lane");
    assert_rejects_positionals("--describe-immersive-acceptance-lane");
    assert_rejects_positionals("--describe-control-preview-workflow-acceptance-lane");
    assert_rejects_positionals("--describe-integrated-live-workflow-acceptance-lane");
    assert_rejects_positionals("--describe-g06-soak-lane");
    assert_rejects_positionals("--describe-release-boundary");
    assert_rejects_positionals("--describe-packaging-manifest");
    assert_rejects_positionals("--describe-downstream-automation");
    assert_rejects_positionals("--describe-downstream-fail-gates");
    assert_rejects_positionals("--describe-generation-closeout");
}
