use super::types::CliMode;

#[derive(Clone, Copy)]
pub(crate) struct DescribeFlagSpec {
    pub(crate) flag: &'static str,
    pub(crate) mode: CliMode,
}

pub(crate) const DESCRIBE_FLAG_SPECS: &[DescribeFlagSpec] = &[
    DescribeFlagSpec {
        flag: "--describe-export",
        mode: CliMode::DescribeExport,
    },
    DescribeFlagSpec {
        flag: "--describe-conformance-matrix",
        mode: CliMode::DescribeConformanceMatrix,
    },
    DescribeFlagSpec {
        flag: "--describe-interruption-boundary",
        mode: CliMode::DescribeInterruptionBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-fault-diagnostic-boundary",
        mode: CliMode::DescribeFaultDiagnosticBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-critical-path-boundary",
        mode: CliMode::DescribeCriticalPathBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-block-timing-boundary",
        mode: CliMode::DescribeBlockTimingBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-deferred-work-policy-boundary",
        mode: CliMode::DescribeDeferredWorkPolicyBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-recording-continuity-boundary",
        mode: CliMode::DescribeRecordingContinuityBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-offline-render-continuity-boundary",
        mode: CliMode::DescribeOfflineRenderContinuityBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-plugin-continuity-boundary",
        mode: CliMode::DescribePluginContinuityBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-vst3-boundary",
        mode: CliMode::DescribeVst3Boundary,
    },
    DescribeFlagSpec {
        flag: "--describe-au-boundary",
        mode: CliMode::DescribeAuBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-macos-au-coreaudio-boundary",
        mode: CliMode::DescribeMacosAuCoreaudioBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-lv2-boundary",
        mode: CliMode::DescribeLv2Boundary,
    },
    DescribeFlagSpec {
        flag: "--describe-linux-lv2-execution-boundary",
        mode: CliMode::DescribeLinuxLv2ExecutionBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-cross-adapter-parity-boundary",
        mode: CliMode::DescribeCrossAdapterParityBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-linux-plugin-parity-boundary",
        mode: CliMode::DescribeLinuxPluginParityBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-linux-audio-backend-boundary",
        mode: CliMode::DescribeLinuxAudioBackendBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-linux-live-ownership-boundary",
        mode: CliMode::DescribeLinuxLiveOwnershipBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-jack-coordination-boundary",
        mode: CliMode::DescribeJackCoordinationBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-pipewire-alsa-parity-boundary",
        mode: CliMode::DescribePipeWireAlsaParityBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-linux-backend-clock-topology-boundary",
        mode: CliMode::DescribeLinuxBackendClockTopologyBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-external-midi-boundary",
        mode: CliMode::DescribeExternalMidiBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-generic-event-boundary",
        mode: CliMode::DescribeGenericEventBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-controller-expression-boundary",
        mode: CliMode::DescribeControllerExpressionBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-control-surface-boundary",
        mode: CliMode::DescribeControlSurfaceBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-advanced-hardware-boundary",
        mode: CliMode::DescribeAdvancedHardwareBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-recall-portability-boundary",
        mode: CliMode::DescribeRecallPortabilityBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-device-supervision-boundary",
        mode: CliMode::DescribeDeviceSupervisionBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-clock-topology-boundary",
        mode: CliMode::DescribeClockTopologyBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-external-io-boundary",
        mode: CliMode::DescribeExternalIoBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-media-service-boundary",
        mode: CliMode::DescribeMediaServiceBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-analysis-metadata-boundary",
        mode: CliMode::DescribeAnalysisMetadataBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-multichannel-boundary",
        mode: CliMode::DescribeMultichannelBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-multi-bus-boundary",
        mode: CliMode::DescribeMultiBusBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-sidechain-boundary",
        mode: CliMode::DescribeSidechainBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-complex-io-boundary",
        mode: CliMode::DescribeComplexIoBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-spatial-boundary",
        mode: CliMode::DescribeSpatialBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-stretch-boundary",
        mode: CliMode::DescribeStretchBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-marker-analysis-boundary",
        mode: CliMode::DescribeMarkerAnalysisBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-transform-artifact-boundary",
        mode: CliMode::DescribeTransformArtifactBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-preview-transform-boundary",
        mode: CliMode::DescribePreviewTransformBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-integrated-acceptance-lane",
        mode: CliMode::DescribeIntegratedAcceptanceLane,
    },
    DescribeFlagSpec {
        flag: "--describe-g07-acceptance-lane",
        mode: CliMode::DescribeG07AcceptanceLane,
    },
    DescribeFlagSpec {
        flag: "--describe-device-workflow-acceptance-lane",
        mode: CliMode::DescribeDeviceWorkflowAcceptanceLane,
    },
    DescribeFlagSpec {
        flag: "--describe-linux-live-acceptance-lane",
        mode: CliMode::DescribeLinuxLiveAcceptanceLane,
    },
    DescribeFlagSpec {
        flag: "--describe-immersive-acceptance-lane",
        mode: CliMode::DescribeImmersiveAcceptanceLane,
    },
    DescribeFlagSpec {
        flag: "--describe-control-preview-workflow-acceptance-lane",
        mode: CliMode::DescribeControlPreviewWorkflowAcceptanceLane,
    },
    DescribeFlagSpec {
        flag: "--describe-integrated-live-workflow-acceptance-lane",
        mode: CliMode::DescribeIntegratedLiveWorkflowAcceptanceLane,
    },
    DescribeFlagSpec {
        flag: "--describe-g06-soak-lane",
        mode: CliMode::DescribeG06SoakLane,
    },
    DescribeFlagSpec {
        flag: "--describe-host-edge-boundary",
        mode: CliMode::DescribeHostEdgeBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-release-boundary",
        mode: CliMode::DescribeReleaseBoundary,
    },
    DescribeFlagSpec {
        flag: "--describe-packaging-manifest",
        mode: CliMode::DescribePackagingManifest,
    },
    DescribeFlagSpec {
        flag: "--describe-downstream-automation",
        mode: CliMode::DescribeDownstreamAutomation,
    },
    DescribeFlagSpec {
        flag: "--describe-downstream-fail-gates",
        mode: CliMode::DescribeDownstreamFailGates,
    },
    DescribeFlagSpec {
        flag: "--describe-generation-closeout",
        mode: CliMode::DescribeGenerationCloseout,
    },
];
