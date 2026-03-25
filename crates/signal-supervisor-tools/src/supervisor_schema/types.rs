#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConformanceMatrixEntryKind {
    PublicBoundaryTest,
    ExportConsumerTest,
    Example,
    Introspection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ConformanceMatrixEntry {
    pub(crate) id: &'static str,
    pub(crate) kind: ConformanceMatrixEntryKind,
    pub(crate) crate_name: &'static str,
    pub(crate) surface: &'static str,
    pub(crate) command: &'static str,
    pub(crate) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct IntegratedAcceptanceFamily {
    pub(crate) id: &'static str,
    pub(crate) title: &'static str,
    pub(crate) required_tasks: &'static [&'static str],
    pub(crate) advisory_tasks: &'static [&'static str],
    pub(crate) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct IntegratedAcceptanceValidationStep {
    pub(crate) id: &'static str,
    pub(crate) command: &'static str,
    pub(crate) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct G06SoakLaneScenarioRecord {
    pub(crate) id: &'static str,
    pub(crate) status: &'static str,
    pub(crate) command: &'static str,
    pub(crate) typed_output: &'static str,
    pub(crate) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct G06SoakLaneValidationStep {
    pub(crate) id: &'static str,
    pub(crate) command: &'static str,
    pub(crate) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GenerationReadinessArea {
    pub(crate) id: &'static str,
    pub(crate) status: &'static str,
    pub(crate) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GenerationCloseoutValidationStep {
    pub(crate) id: &'static str,
    pub(crate) command: &'static str,
    pub(crate) rationale: &'static str,
}

impl ConformanceMatrixEntryKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::PublicBoundaryTest => "public-boundary-test",
            Self::ExportConsumerTest => "export-consumer-test",
            Self::Example => "example",
            Self::Introspection => "introspection",
        }
    }
}
