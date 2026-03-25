use super::{assert_rejects_positionals, assert_supports_describe};
use crate::CliMode;

#[test]
fn parse_args_supports_runtime_boundary_modes() {
    assert_supports_describe(
        "--describe-interruption-boundary",
        CliMode::DescribeInterruptionBoundary,
    );
    assert_supports_describe(
        "--describe-fault-diagnostic-boundary",
        CliMode::DescribeFaultDiagnosticBoundary,
    );
    assert_supports_describe(
        "--describe-critical-path-boundary",
        CliMode::DescribeCriticalPathBoundary,
    );
    assert_supports_describe(
        "--describe-block-timing-boundary",
        CliMode::DescribeBlockTimingBoundary,
    );
    assert_supports_describe(
        "--describe-deferred-work-policy-boundary",
        CliMode::DescribeDeferredWorkPolicyBoundary,
    );
    assert_supports_describe(
        "--describe-recording-continuity-boundary",
        CliMode::DescribeRecordingContinuityBoundary,
    );
    assert_supports_describe(
        "--describe-offline-render-continuity-boundary",
        CliMode::DescribeOfflineRenderContinuityBoundary,
    );
    assert_supports_describe(
        "--describe-plugin-continuity-boundary",
        CliMode::DescribePluginContinuityBoundary,
    );
    assert_supports_describe(
        "--describe-host-edge-boundary",
        CliMode::DescribeHostEdgeBoundary,
    );
}

#[test]
fn parse_args_rejects_positionals_with_runtime_boundary_modes() {
    assert_rejects_positionals("--describe-interruption-boundary");
    assert_rejects_positionals("--describe-fault-diagnostic-boundary");
    assert_rejects_positionals("--describe-critical-path-boundary");
    assert_rejects_positionals("--describe-block-timing-boundary");
    assert_rejects_positionals("--describe-deferred-work-policy-boundary");
    assert_rejects_positionals("--describe-recording-continuity-boundary");
    assert_rejects_positionals("--describe-offline-render-continuity-boundary");
    assert_rejects_positionals("--describe-plugin-continuity-boundary");
    assert_rejects_positionals("--describe-host-edge-boundary");
}
