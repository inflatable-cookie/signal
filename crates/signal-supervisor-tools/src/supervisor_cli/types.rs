#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostProfile {
    Local,
    Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Scenario {
    Default,
    Timeout,
    Crash,
    Heartbeat,
    Soak,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostSummaryDebugSection {
    Payload,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CliMode {
    Run {
        profile: HostProfile,
        scenario: Scenario,
    },
    DescribeExport,
    DescribeConformanceMatrix,
    DescribeInterruptionBoundary,
    DescribeFaultDiagnosticBoundary,
    DescribeCriticalPathBoundary,
    DescribeBlockTimingBoundary,
    DescribeDeferredWorkPolicyBoundary,
    DescribeRecordingContinuityBoundary,
    DescribeOfflineRenderContinuityBoundary,
    DescribePluginContinuityBoundary,
    DescribeVst3Boundary,
    DescribeAuBoundary,
    DescribeMacosAuCoreaudioBoundary,
    DescribeLv2Boundary,
    DescribeLinuxLv2ExecutionBoundary,
    DescribeCrossAdapterParityBoundary,
    DescribeLinuxPluginParityBoundary,
    DescribeLinuxAudioBackendBoundary,
    DescribeLinuxLiveOwnershipBoundary,
    DescribeJackCoordinationBoundary,
    DescribePipeWireAlsaParityBoundary,
    DescribeLinuxBackendClockTopologyBoundary,
    DescribeExternalMidiBoundary,
    DescribeGenericEventBoundary,
    DescribeControllerExpressionBoundary,
    DescribeControlSurfaceBoundary,
    DescribeAdvancedHardwareBoundary,
    DescribeRecallPortabilityBoundary,
    DescribeDeviceSupervisionBoundary,
    DescribeClockTopologyBoundary,
    DescribeExternalIoBoundary,
    DescribeMediaServiceBoundary,
    DescribeAnalysisMetadataBoundary,
    DescribeMultichannelBoundary,
    DescribeMultiBusBoundary,
    DescribeSidechainBoundary,
    DescribeComplexIoBoundary,
    DescribeSpatialBoundary,
    DescribeStretchBoundary,
    DescribeMarkerAnalysisBoundary,
    DescribeTransformArtifactBoundary,
    DescribePreviewTransformBoundary,
    DescribeIntegratedAcceptanceLane,
    DescribeG07AcceptanceLane,
    DescribeDeviceWorkflowAcceptanceLane,
    DescribeLinuxLiveAcceptanceLane,
    DescribeImmersiveAcceptanceLane,
    DescribeControlPreviewWorkflowAcceptanceLane,
    DescribeIntegratedLiveWorkflowAcceptanceLane,
    DescribeG06SoakLane,
    DescribeHostEdgeBoundary,
    DescribeReleaseBoundary,
    DescribePackagingManifest,
    DescribeDownstreamAutomation,
    DescribeDownstreamFailGates,
    DescribeGenerationCloseout,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ExportDebugOptions {
    pub(crate) payload: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CliArgs {
    pub(crate) format: OutputFormat,
    pub(crate) debug: ExportDebugOptions,
    pub(crate) mode: CliMode,
}

impl HostProfile {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "local" => Ok(Self::Local),
            "server" => Ok(Self::Server),
            _ => Err(format!(
                "unknown profile {value:?}; expected one of: local, server"
            )),
        }
    }
}

impl Scenario {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "default" => Ok(Self::Default),
            "timeout" => Ok(Self::Timeout),
            "crash" => Ok(Self::Crash),
            "heartbeat" => Ok(Self::Heartbeat),
            "soak" => Ok(Self::Soak),
            "mixed" => Ok(Self::Mixed),
            _ => Err(format!(
                "unknown scenario {value:?}; expected one of: default, timeout, crash, heartbeat, soak, mixed"
            )),
        }
    }
}

impl OutputFormat {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(format!(
                "unknown format {value:?}; expected one of: text, json"
            )),
        }
    }
}

impl HostSummaryDebugSection {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Payload => "payload",
        }
    }
}

impl ExportDebugOptions {
    pub(crate) fn supports(self, section: HostSummaryDebugSection) -> bool {
        match section {
            HostSummaryDebugSection::Payload => self.payload,
        }
    }
}
