use signal_primitives::Sample;

/// Evidence for one additive component in the report-only H/R/P proof.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchHprComponentEvidence {
    /// Sum of squared component samples.
    pub energy: f64,
    /// Component energy divided by total separated-component energy.
    pub energy_share: f64,
    /// Energy margin over the strongest other component, in decibels.
    pub dominance_margin_db: f64,
    /// Stable FNV-1a hash of the component sample bits.
    pub sample_hash: u64,
    /// Whether every component sample is finite.
    pub all_samples_finite: bool,
}

/// Reconstruction and mask evidence from the report-only H/R/P proof.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchHprSeparationEvidence {
    /// Long-stage power-of-two STFT frame size.
    pub long_window_frames: usize,
    /// Short-stage power-of-two STFT frame size.
    pub short_window_frames: usize,
    /// Long-stage quarter-window hop size.
    pub long_hop_frames: usize,
    /// Short-stage quarter-window hop size.
    pub short_hop_frames: usize,
    /// Long-stage odd horizontal median span in frames.
    pub long_horizontal_median_frames: usize,
    /// Long-stage odd vertical median span in bins.
    pub long_vertical_median_bins: usize,
    /// Short-stage odd horizontal median span in frames.
    pub short_horizontal_median_frames: usize,
    /// Short-stage odd vertical median span in bins.
    pub short_vertical_median_bins: usize,
    /// Number of positive-frequency long-stage bins assigned harmonic.
    pub harmonic_mask_bins: usize,
    /// Number of positive-frequency long-stage bins assigned complement.
    pub long_complement_mask_bins: usize,
    /// Number of positive-frequency short-stage bins assigned percussive.
    pub percussive_mask_bins: usize,
    /// Number of positive-frequency short-stage bins assigned residual.
    pub residual_mask_bins: usize,
    /// Whether both binary mask pairs were mutually exclusive and exhaustive.
    pub masks_partition_exactly: bool,
    /// Bins missing or multiply assigned across the two binary mask pairs.
    pub mask_partition_error_bins: usize,
    /// Source samples with no long-stage overlap-add normalization coverage.
    pub long_uncovered_source_samples: usize,
    /// Source samples with no short-stage overlap-add normalization coverage.
    pub short_uncovered_source_samples: usize,
    /// Largest absolute source/recombined-component sample error.
    pub reconstruction_peak_error: f64,
    /// Root-mean-square source/recombined-component sample error.
    pub reconstruction_rms_error: f64,
    /// Absolute reconstruction error at the first source sample.
    pub reconstruction_head_error: f64,
    /// Absolute reconstruction error at the final source sample.
    pub reconstruction_tail_error: f64,
    /// Harmonic-component evidence.
    pub harmonic: StretchHprComponentEvidence,
    /// Residual-component evidence.
    pub residual: StretchHprComponentEvidence,
    /// Percussive-component evidence.
    pub percussive: StretchHprComponentEvidence,
}

/// Report-only two-stage harmonic/residual/percussive source decomposition.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchHprSeparationReview {
    /// Clearly harmonic source component.
    pub harmonic: Vec<Sample>,
    /// Ambiguous source component left after both strict masks.
    pub residual: Vec<Sample>,
    /// Clearly percussive source component.
    pub percussive: Vec<Sample>,
    /// Numerical, mask, and deterministic-component evidence.
    pub evidence: StretchHprSeparationEvidence,
}

/// Report-only additive H/R/P fixed-ratio mono render and evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchHprAdditiveRender {
    /// Sample-aligned sum of the three rendered components.
    pub samples: Vec<Sample>,
    /// Long-window identity-locked harmonic render.
    pub harmonic: Vec<Sample>,
    /// Current-kernel residual render.
    pub residual: Vec<Sample>,
    /// Short-window normalized OLA percussive render.
    pub percussive: Vec<Sample>,
    /// Source-separation evidence used by this render.
    pub separation: StretchHprSeparationEvidence,
    /// Global-map synthesis positions used by the percussive OLA component.
    pub percussive_synthesis_positions: Vec<usize>,
    /// Cropped percussive output frames without OLA normalization coverage.
    pub percussive_uncovered_output_frames: usize,
    /// Whether percussive synthesis positions are monotonic.
    pub percussive_positions_monotonic: bool,
    /// Whether every component exactly matches the final target length.
    pub component_lengths_match: bool,
    /// Harmonic output peak growth relative to its source component.
    pub harmonic_peak_growth_db: f64,
    /// Residual output peak growth relative to its source component.
    pub residual_peak_growth_db: f64,
    /// Percussive output peak growth relative to its source component.
    pub percussive_peak_growth_db: f64,
    /// Final additive output peak growth relative to the full-band source.
    pub recombination_peak_growth_db: f64,
    /// Must remain false: components receive no hidden gain correction.
    pub hidden_component_gain_applied: bool,
}

impl StretchHprAdditiveRender {
    pub(super) fn empty(separation: StretchHprSeparationEvidence) -> Self {
        Self {
            samples: Vec::new(),
            harmonic: Vec::new(),
            residual: Vec::new(),
            percussive: Vec::new(),
            separation,
            percussive_synthesis_positions: Vec::new(),
            percussive_uncovered_output_frames: 0,
            percussive_positions_monotonic: true,
            component_lengths_match: true,
            harmonic_peak_growth_db: 0.0,
            residual_peak_growth_db: 0.0,
            percussive_peak_growth_db: 0.0,
            recombination_peak_growth_db: 0.0,
            hidden_component_gain_applied: false,
        }
    }
}
