#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct RegionRecord {
    pub peak: usize,
    pub owner: usize,
    pub supported: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct MaterialGuidance {
    pub tonalness: f64,
    pub noisiness: f64,
    pub transientness: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StateTickControl {
    pub transient_center: bool,
    pub ordinary_bypass: bool,
    pub analysis_advance: f64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum TerminalState {
    #[default]
    Reset,
    Attack,
    Ordinary,
    Unlocked,
    Locked,
}

impl TerminalState {
    pub fn index(self) -> usize {
        match self {
            Self::Reset => 0,
            Self::Attack => 1,
            Self::Ordinary => 2,
            Self::Unlocked => 3,
            Self::Locked => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct StateTickReport {
    pub states: [usize; 5],
    pub borrowed_regions: usize,
    pub local_regions: usize,
    pub owner_switches: usize,
    pub non_finite_values: usize,
    pub hash: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StateError {
    CurrentShape,
    GuidanceShape,
    OutputShape,
    StateShape,
    AnalysisAdvance,
}
