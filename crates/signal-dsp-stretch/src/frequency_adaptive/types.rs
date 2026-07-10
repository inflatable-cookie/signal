use signal_primitives::Sample;

/// Geometry evidence for one band in the frequency-adaptive reconstruction proof.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchFrequencyAdaptiveBandEvidence {
    /// Centre bin in the full transform's FFT ordering.
    pub center_bin: usize,
    /// Absolute centre frequency.
    pub center_frequency_hz: f64,
    /// Number of nonzero frequency samples in the compact analysis filter.
    pub support_bins: usize,
    /// Source-frame spacing between adjacent coefficients in this band.
    pub decimation_frames: usize,
    /// Number of complex coefficients produced by this band.
    pub coefficient_count: usize,
    /// Peak position of the zero-phase band impulse response.
    pub impulse_peak_frame: usize,
}

/// Transform and reconstruction evidence from the frequency-adaptive proof.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchFrequencyAdaptiveEvidence {
    /// Full transform length.
    pub fft_frames: usize,
    /// Number of analysis bands, including mirrored and completion bands.
    pub band_count: usize,
    /// Total complex coefficient count.
    pub coefficient_count: usize,
    /// Smallest diagonal frame-operator value.
    pub frame_operator_min: f64,
    /// Largest diagonal frame-operator value.
    pub frame_operator_max: f64,
    /// Ratio of maximum to minimum frame-operator values.
    pub frame_condition_ratio: f64,
    /// Frequency samples with no nonzero analysis filter.
    pub uncovered_frequency_bins: usize,
    /// Frequency samples owned by more than one nonzero analysis filter.
    pub multiply_covered_frequency_bins: usize,
    /// Bands whose compact support exceeds their decimation.
    pub painless_support_violations: usize,
    /// Input sample count.
    pub source_frames: usize,
    /// Reconstructed sample count.
    pub output_frames: usize,
    /// Largest absolute reconstruction error.
    pub reconstruction_peak_error: f64,
    /// Root-mean-square reconstruction error.
    pub reconstruction_rms_error: f64,
    /// Absolute reconstruction error at the first sample.
    pub reconstruction_head_error: f64,
    /// Absolute reconstruction error at the final sample.
    pub reconstruction_tail_error: f64,
    /// Non-finite analysis coefficients.
    pub non_finite_coefficients: usize,
    /// Non-finite reconstructed samples.
    pub non_finite_output_samples: usize,
    /// Largest band impulse offset from the declared zero-phase origin.
    pub max_band_impulse_delay_frames: usize,
    /// Stable FNV-1a hash of analysis-filter geometry and values.
    pub filter_hash: u64,
    /// Stable FNV-1a hash of complex coefficient bits.
    pub coefficient_hash: u64,
    /// Stable FNV-1a hash of reconstructed sample bits.
    pub reconstruction_hash: u64,
    /// Per-band geometry and delay evidence.
    pub bands: Vec<StretchFrequencyAdaptiveBandEvidence>,
}

/// Report-only identity reconstruction through a frequency-adaptive frame.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchFrequencyAdaptiveReview {
    /// Exact-length reconstructed mono samples.
    pub samples: Vec<Sample>,
    /// Transform geometry and reconstruction evidence.
    pub evidence: StretchFrequencyAdaptiveEvidence,
}
