use super::{assert_rejects_positionals, assert_supports_describe};
use crate::CliMode;

#[test]
fn parse_args_supports_plugin_and_linux_boundary_modes() {
    assert_supports_describe("--describe-vst3-boundary", CliMode::DescribeVst3Boundary);
    assert_supports_describe("--describe-au-boundary", CliMode::DescribeAuBoundary);
    assert_supports_describe("--describe-lv2-boundary", CliMode::DescribeLv2Boundary);
    assert_supports_describe(
        "--describe-linux-lv2-execution-boundary",
        CliMode::DescribeLinuxLv2ExecutionBoundary,
    );
    assert_supports_describe(
        "--describe-cross-adapter-parity-boundary",
        CliMode::DescribeCrossAdapterParityBoundary,
    );
    assert_supports_describe(
        "--describe-linux-plugin-parity-boundary",
        CliMode::DescribeLinuxPluginParityBoundary,
    );
    assert_supports_describe(
        "--describe-linux-audio-backend-boundary",
        CliMode::DescribeLinuxAudioBackendBoundary,
    );
    assert_supports_describe(
        "--describe-linux-live-ownership-boundary",
        CliMode::DescribeLinuxLiveOwnershipBoundary,
    );
    assert_supports_describe(
        "--describe-jack-coordination-boundary",
        CliMode::DescribeJackCoordinationBoundary,
    );
    assert_supports_describe(
        "--describe-pipewire-alsa-parity-boundary",
        CliMode::DescribePipeWireAlsaParityBoundary,
    );
    assert_supports_describe(
        "--describe-linux-backend-clock-topology-boundary",
        CliMode::DescribeLinuxBackendClockTopologyBoundary,
    );
}

#[test]
fn parse_args_rejects_positionals_with_plugin_and_linux_boundary_modes() {
    assert_rejects_positionals("--describe-vst3-boundary");
    assert_rejects_positionals("--describe-au-boundary");
    assert_rejects_positionals("--describe-lv2-boundary");
    assert_rejects_positionals("--describe-linux-lv2-execution-boundary");
    assert_rejects_positionals("--describe-cross-adapter-parity-boundary");
    assert_rejects_positionals("--describe-linux-plugin-parity-boundary");
    assert_rejects_positionals("--describe-linux-audio-backend-boundary");
    assert_rejects_positionals("--describe-linux-live-ownership-boundary");
    assert_rejects_positionals("--describe-jack-coordination-boundary");
    assert_rejects_positionals("--describe-pipewire-alsa-parity-boundary");
    assert_rejects_positionals("--describe-linux-backend-clock-topology-boundary");
}
