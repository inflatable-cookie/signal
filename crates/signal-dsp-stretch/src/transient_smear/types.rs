#[cfg(any(test, feature = "evidence"))]
use crate::benchmark::StretchMetricValue;

/// One detected transient candidate in stretch audio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchTransientEvent {
    /// Sample-frame index at the beginning of the detected analysis frame.
    pub frame_index: usize,
    /// Normalized positive frame-energy rise score.
    pub energy_score: f64,
    /// Normalized positive spectral-flux score.
    pub spectral_flux_score: f64,
    /// Combined detector score. Higher means a stronger transient candidate.
    pub combined_score: f64,
}

/// Threshold policy for stretch transient detection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchTransientDetectorPolicy {
    /// Minimum combined normalized energy-rise plus spectral-flux score.
    pub minimum_combined_score: f64,
    /// Minimum normalized spectral-flux score.
    pub minimum_spectral_flux_score: f64,
}

impl StretchTransientDetectorPolicy {
    /// Current production selector and metric policy.
    pub const fn production() -> Self {
        Self {
            minimum_combined_score: 3.0,
            minimum_spectral_flux_score: 2.0,
        }
    }

    /// Recovery policy used when the production output detector misses.
    pub const fn candidate_review() -> Self {
        Self {
            minimum_combined_score: 2.0,
            minimum_spectral_flux_score: 1.5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SelectorTransientSmearMeasurement {
    pub(crate) missed_transients: usize,
    pub(crate) max_smear_frames: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::transient_smear) struct RawTransientSmearMeasurement {
    pub(in crate::transient_smear) ratio: f64,
    pub(in crate::transient_smear) input_transients: usize,
    pub(in crate::transient_smear) output_transients: usize,
    pub(in crate::transient_smear) matched_transients: usize,
    pub(in crate::transient_smear) missed_transients: usize,
    pub(in crate::transient_smear) mean_smear_frames: f64,
    pub(in crate::transient_smear) max_smear_frames: f64,
    pub(in crate::transient_smear) max_matched_smear_frames: f64,
    pub(in crate::transient_smear) max_matched_input_frame: f64,
    pub(in crate::transient_smear) max_matched_output_frame: f64,
    pub(in crate::transient_smear) max_matched_input_width_frames: f64,
    pub(in crate::transient_smear) max_matched_output_width_frames: f64,
}

/// Transient smear measurement for one rendered stretch output.
#[cfg(any(test, feature = "evidence"))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchTransientSmearMeasurement {
    /// Output/input duration ratio measured.
    pub ratio: f64,
    /// Number of detected input transient candidates.
    pub input_transients: usize,
    /// Number of detected output transient candidates.
    pub output_transients: usize,
    /// Number of input transients matched to stretched-output transients.
    pub matched_transients: usize,
    /// Number of input transients with no output transient match.
    pub missed_transients: usize,
    /// Mean positive attack widening in sample frames.
    pub mean_smear_frames: f64,
    /// Worst positive attack widening in sample frames.
    pub max_smear_frames: f64,
    /// Worst positive attack widening among matched transient events only.
    pub max_matched_smear_frames: f64,
    /// Input frame for the worst matched transient smear event.
    pub max_matched_input_frame: f64,
    /// Output frame for the worst matched transient smear event.
    pub max_matched_output_frame: f64,
    /// Input attack width for the worst matched transient smear event.
    pub max_matched_input_width_frames: f64,
    /// Output attack width for the worst matched transient smear event.
    pub max_matched_output_width_frames: f64,
    /// Metric reported to the acceptance harness.
    pub metric: StretchMetricValue,
}

/// Detector policies for one transient-smear measurement.
///
/// One entry point replaces the four that previously wrapped the same private
/// function: default, explicit single policy, separate input and output
/// policies, and separate policies plus an output recovery fallback.
#[cfg(any(test, feature = "evidence"))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchTransientSmearPolicies {
    /// Detector applied to the source.
    pub input: StretchTransientDetectorPolicy,
    /// Detector applied to the rendered output.
    pub output: StretchTransientDetectorPolicy,
    /// Fallback detector for outputs the primary detector misses.
    pub output_recovery: Option<StretchTransientDetectorPolicy>,
}

#[cfg(any(test, feature = "evidence"))]
impl StretchTransientSmearPolicies {
    /// Production detectors with candidate-review recovery, the policy the
    /// corpus and selector gates measure with.
    pub const fn production() -> Self {
        Self {
            input: StretchTransientDetectorPolicy::production(),
            output: StretchTransientDetectorPolicy::production(),
            output_recovery: Some(StretchTransientDetectorPolicy::candidate_review()),
        }
    }

    /// One detector on both sides, with no recovery pass.
    pub const fn symmetric(policy: StretchTransientDetectorPolicy) -> Self {
        Self {
            input: policy,
            output: policy,
            output_recovery: None,
        }
    }
}

#[cfg(any(test, feature = "evidence"))]
impl Default for StretchTransientSmearPolicies {
    fn default() -> Self {
        Self::production()
    }
}
