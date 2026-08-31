//! Offline high-quality time-stretcher with path selection.

use signal_primitives::{Sample, SampleRate};

use crate::cache_identity::StretchRatioPoint;
use crate::phase_vocoder::{
    transient_reset_phase_vocoder, transient_reset_phase_vocoder_linked_stereo,
};
use crate::stretch_engine::{
    checked_output_frames, checked_target_frames, linear_time_scale_interleaved_stereo,
    pitch_shift_interleaved_stereo_to_nominal_rate, pitch_shift_mono_to_nominal_rate,
    sanitize_ratio, short_window_analysis_hop_for_path, short_window_size_for_path,
    should_select_compression_short_window, should_select_compression_short_window_interleaved,
    should_select_expansion_short_window, should_select_expansion_short_window_interleaved,
    stretch_mono_with_engine, stretch_to_exact_linked_stereo, stretch_to_exact_mono,
    StretchRenderError,
};

use super::time_stretcher::TimeStretcher;
use super::types::{
    OfflineHighQualityPath, StretchQuality, DEFAULT_ANALYSIS_HOP, DEFAULT_WINDOW_SIZE,
};

/// Offline high-quality time-stretcher.
///
/// This is the first Signal-owned offline-quality DSP path: a deterministic
/// whole-buffer STFT stretcher with identity phase locking and transient phase
/// resets. It is exposed as [`StretchQuality::OfflineHighQuality`] for
/// export/cache/freeze artifact planning, while product-facing consumption is
/// gated by accepted promotion evidence on each artifact plan.
#[derive(Debug)]
pub struct OfflineHighQualityStretcher {
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    path: OfflineHighQualityPath,
}

impl OfflineHighQualityStretcher {
    /// Stretcher with the default window/hop configuration.
    pub fn new(ratio: f64) -> Self {
        Self::with_window(ratio, DEFAULT_WINDOW_SIZE, DEFAULT_ANALYSIS_HOP)
    }

    /// Stretcher with an explicit window size and analysis hop. The window
    /// is clamped to a power of two ≥ 64; the hop to `1..=window/2`.
    pub fn with_window(ratio: f64, window_size: usize, analysis_hop: usize) -> Self {
        let window_size = window_size.next_power_of_two().max(64);
        let analysis_hop = analysis_hop.clamp(1, window_size / 2);
        let mut stretcher = Self {
            ratio: 1.0,
            window_size,
            analysis_hop,
            path: OfflineHighQualityPath::Default,
        };
        stretcher.set_ratio(ratio);
        stretcher
    }

    /// Stretcher with the default window/hop and an explicit offline path.
    pub fn with_path(ratio: f64, path: OfflineHighQualityPath) -> Self {
        let mut stretcher = Self::new(ratio);
        stretcher.path = path;
        stretcher
    }

    /// Current offline high-quality renderer path.
    pub fn path(&self) -> OfflineHighQualityPath {
        self.path
    }

    /// Set the offline high-quality renderer path.
    pub fn set_path(&mut self, path: OfflineHighQualityPath) {
        self.path = path;
    }

    /// Stretch an interleaved stereo buffer through the linked
    /// OfflineHighQuality prototype path.
    ///
    /// This path uses a mid/side linked analysis surface so stereo image
    /// metrics can be measured against a candidate that preserves channel
    /// relationships directly. A trailing odd sample is ignored.
    pub fn stretch_interleaved_stereo(
        &mut self,
        frames: &[Sample],
    ) -> Result<Vec<Sample>, StretchRenderError> {
        let frame_count = frames.len() / 2;
        let target_frames = checked_target_frames(frame_count, self.ratio, 2)?;
        if frame_count == 0 || target_frames == 0 {
            return Ok(Vec::new());
        }
        let even_frames = &frames[..frame_count * 2];
        if (self.ratio - 1.0).abs() < 1.0e-9 {
            return Ok(even_frames.to_vec());
        }
        if frame_count < self.window_size {
            return Ok(linear_time_scale_interleaved_stereo(
                even_frames,
                target_frames,
            ));
        }

        let default_output = transient_reset_phase_vocoder_linked_stereo(
            even_frames,
            target_frames,
            self.ratio,
            self.window_size,
            self.analysis_hop,
        );
        let selected_short_window = match self.path {
            OfflineHighQualityPath::Default => false,
            OfflineHighQualityPath::CompressionShortWindowSelector => {
                should_select_compression_short_window_interleaved(
                    even_frames,
                    &default_output,
                    self.ratio,
                )
            }
            OfflineHighQualityPath::ExpansionShortWindowSelector => {
                should_select_expansion_short_window_interleaved(
                    even_frames,
                    &default_output,
                    self.ratio,
                )
            }
        };
        if selected_short_window {
            Ok(transient_reset_phase_vocoder_linked_stereo(
                even_frames,
                target_frames,
                self.ratio,
                short_window_size_for_path(self.path),
                short_window_analysis_hop_for_path(self.path),
            ))
        } else {
            Ok(default_output)
        }
    }

    /// Apply independent pitch shift and tempo stretch to one mono buffer.
    ///
    /// `pitch_shift_semitones` changes pitch without changing the final
    /// duration target. The current [`Self::ratio`] remains the tempo/output
    /// duration contract, so output length is
    /// `round(input.len() as f64 * self.ratio)`.
    pub fn stretch_pitch_mono(
        &mut self,
        input: &[Sample],
        sample_rate: SampleRate,
        pitch_shift_semitones: f64,
    ) -> Result<Vec<Sample>, StretchRenderError> {
        let target_len = checked_target_frames(input.len(), self.ratio, 1)?;
        if input.is_empty() || target_len == 0 {
            return Ok(Vec::new());
        }
        if pitch_shift_semitones.abs() < 1.0e-9 || sample_rate.0 == 0 {
            return self.stretch_mono(input);
        }

        let pitched = pitch_shift_mono_to_nominal_rate(input, sample_rate, pitch_shift_semitones);
        Ok(stretch_to_exact_mono(
            &pitched,
            target_len,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        ))
    }

    /// Apply independent pitch shift and tempo stretch to interleaved stereo.
    ///
    /// Pitch shift is composed through linked mid/side resampling, then the
    /// linked OfflineHighQuality stereo stretcher restores the requested tempo
    /// duration. A trailing odd sample is ignored.
    pub fn stretch_pitch_interleaved_stereo(
        &mut self,
        frames: &[Sample],
        sample_rate: SampleRate,
        pitch_shift_semitones: f64,
    ) -> Result<Vec<Sample>, StretchRenderError> {
        let frame_count = frames.len() / 2;
        let target_frames = checked_target_frames(frame_count, self.ratio, 2)?;
        if frame_count == 0 || target_frames == 0 {
            return Ok(Vec::new());
        }
        let even_frames = &frames[..frame_count * 2];
        if pitch_shift_semitones.abs() < 1.0e-9 || sample_rate.0 == 0 {
            return self.stretch_interleaved_stereo(even_frames);
        }

        let pitched = pitch_shift_interleaved_stereo_to_nominal_rate(
            even_frames,
            sample_rate,
            pitch_shift_semitones,
        );
        Ok(stretch_to_exact_linked_stereo(
            &pitched,
            target_frames,
            self.window_size,
            self.analysis_hop,
        ))
    }

    /// Apply one static pitch shift while following a stepwise dynamic ratio
    /// curve over interleaved stereo.
    ///
    /// Segment boundaries use the same source-frame vocabulary as
    /// [`Self::stretch_dynamic_ratio_interleaved_stereo`]. Resampling runs
    /// ahead of the stretch over the whole stream, so the stretch plan is in
    /// pitched coordinates — the same order the offline artifact renderer
    /// uses, and the reason there is no per-segment resampler restart to
    /// smooth.
    pub fn stretch_dynamic_ratio_pitch_interleaved_stereo(
        &mut self,
        frames: &[Sample],
        ratio_curve: &[StretchRatioPoint],
        sample_rate: SampleRate,
        pitch_shift_semitones: f64,
    ) -> Result<Vec<Sample>, StretchRenderError> {
        if pitch_shift_semitones.abs() < 1.0e-9 || sample_rate.0 == 0 {
            return self.stretch_dynamic_ratio_interleaved_stereo(frames, ratio_curve);
        }
        self.stretch_dynamic_ratio_resumable(
            frames,
            2,
            ratio_curve,
            sample_rate,
            pitch_shift_semitones,
        )
    }

    /// Stretch one mono buffer with a stepwise dynamic ratio curve.
    ///
    /// `ratio_curve` uses the same sample-frame vocabulary as cache identity:
    /// each [`StretchRatioPoint::timeline_frame`] is interpreted as a
    /// source-frame offset in this buffer where the point's ratio becomes
    /// active. Invalid points are ignored. Gaps before the first valid point
    /// use the stretcher's current [`Self::ratio`].
    pub fn stretch_dynamic_ratio_mono(
        &mut self,
        input: &[Sample],
        ratio_curve: &[StretchRatioPoint],
    ) -> Result<Vec<Sample>, StretchRenderError> {
        self.stretch_dynamic_ratio_resumable(input, 1, ratio_curve, SampleRate(48_000), 0.0)
    }

    /// Render a dynamic ratio curve through the resumable renderer in one call.
    ///
    /// The segmented predecessor rendered each ratio segment independently and
    /// concatenated them, which restarts the phase vocoder at every segment join.
    /// Measured on a sustained `110 Hz` tone across a `1.6 -> 0.8` boundary, that
    /// left a first-difference step of `0.204` against a median step of `0.0051`.
    /// `smooth_dynamic_segment_boundaries_interleaved` attenuated it to `0.0174`
    /// but did not remove it — still above the render's own `p99.9` of `0.0138`.
    ///
    /// The resumable renderer carries phase, detector, and overlap-add state across
    /// the boundary, so there is no join to smooth: `0.0068`, below that same
    /// `p99.9`. Its whole-render `p99.9` is also half the segmented path's, because
    /// every segment restart was contributing, not only the ones at a ratio change.
    fn stretch_dynamic_ratio_resumable(
        &self,
        input: &[Sample],
        channels: usize,
        ratio_curve: &[StretchRatioPoint],
        sample_rate: SampleRate,
        pitch_shift_semitones: f64,
    ) -> Result<Vec<Sample>, StretchRenderError> {
        let frame_count = input.len() / channels;
        let even_input = &input[..frame_count * channels];
        let mut renderer = crate::resumable::ResumableOfflineStretch::new(
            crate::resumable::ResumableStretchConfig {
                channels,
                window_size: self.window_size,
                analysis_hop: self.analysis_hop,
                source_frames: frame_count,
                ratio_curve: ratio_curve.to_vec(),
                fallback_ratio: sanitize_ratio(self.ratio),
                sample_rate,
                pitch_shift_semitones,
            },
        )?;
        checked_output_frames(renderer.target_output_frames() as f64, channels)?;
        let mut output = Vec::with_capacity(renderer.target_output_frames() * channels);
        renderer.render(even_input, &mut output)?;
        renderer.flush(&mut output)?;
        Ok(output)
    }

    /// Stretch an interleaved stereo buffer with a stepwise dynamic ratio
    /// curve through the linked OfflineHighQuality prototype path.
    ///
    /// A trailing odd sample is ignored. Segment boundaries are deterministic
    /// and sample-domain; smoothing/crossfade policy remains promotion work.
    pub fn stretch_dynamic_ratio_interleaved_stereo(
        &mut self,
        frames: &[Sample],
        ratio_curve: &[StretchRatioPoint],
    ) -> Result<Vec<Sample>, StretchRenderError> {
        self.stretch_dynamic_ratio_resumable(frames, 2, ratio_curve, SampleRate(48_000), 0.0)
    }
}

impl TimeStretcher for OfflineHighQualityStretcher {
    fn quality(&self) -> StretchQuality {
        StretchQuality::OfflineHighQuality
    }

    fn ratio(&self) -> f64 {
        self.ratio
    }

    fn set_ratio(&mut self, ratio: f64) {
        self.ratio = sanitize_ratio(ratio);
    }

    fn stretch_mono(&mut self, input: &[Sample]) -> Result<Vec<Sample>, StretchRenderError> {
        let default_output = stretch_mono_with_engine(
            input,
            self.ratio,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        )?;
        let selected_short_window = match self.path {
            OfflineHighQualityPath::Default => false,
            OfflineHighQualityPath::CompressionShortWindowSelector => {
                should_select_compression_short_window(input, &default_output, self.ratio)
            }
            OfflineHighQualityPath::ExpansionShortWindowSelector => {
                should_select_expansion_short_window(input, &default_output, self.ratio)
            }
        };
        if selected_short_window {
            stretch_mono_with_engine(
                input,
                self.ratio,
                short_window_size_for_path(self.path),
                short_window_analysis_hop_for_path(self.path),
                transient_reset_phase_vocoder,
            )
        } else {
            Ok(default_output)
        }
    }
}
