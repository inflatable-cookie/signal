//! Lower-latency pitch-preserving preview stretcher.

use signal_primitives::{Sample, SampleRate};

use crate::cache_identity::StretchRatioPoint;
use crate::phase_vocoder::{
    transient_reset_phase_vocoder, transient_reset_phase_vocoder_linked_stereo,
};
use crate::stretch_engine::{
    checked_target_frames, linear_time_scale_interleaved_stereo,
    pitch_shift_interleaved_stereo_to_nominal_rate, pitch_shift_mono_to_nominal_rate,
    sanitize_ratio, stretch_dynamic_ratio_linked_stereo_with_engine,
    stretch_dynamic_ratio_mono_with_engine, stretch_mono_with_engine,
    stretch_to_exact_linked_stereo, stretch_to_exact_mono, StretchRenderError,
};
use crate::{
    plan_realtime_preview_stream, RealtimePreviewPlanError, RealtimePreviewStreamConfig,
    RealtimePreviewStreamingContract,
};

use super::time_stretcher::TimeStretcher;
use super::types::{StretchQuality, REALTIME_PREVIEW_ANALYSIS_HOP, REALTIME_PREVIEW_WINDOW_SIZE};

/// Lower-latency pitch-preserving preview stretcher.
///
/// This is a control-side prototype, not a render-callback object. It uses a
/// shorter STFT window than [`OfflineHighQualityStretcher`] so edits can be
/// previewed with lower algorithmic latency, while keeping the same clean-room
/// transient-reset and linked-stereo foundation.
pub struct RealtimePreviewStretcher {
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
}

impl RealtimePreviewStretcher {
    /// Stretcher with the preview window/hop configuration.
    pub fn new(ratio: f64) -> Self {
        Self::with_window(
            ratio,
            REALTIME_PREVIEW_WINDOW_SIZE,
            REALTIME_PREVIEW_ANALYSIS_HOP,
        )
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
        };
        stretcher.set_ratio(ratio);
        stretcher
    }

    /// Build the stream contract for this preview stretcher.
    pub fn streaming_contract(
        &self,
        sample_rate: SampleRate,
        channel_count: usize,
        max_block_frames: usize,
    ) -> Result<RealtimePreviewStreamingContract, RealtimePreviewPlanError> {
        plan_realtime_preview_stream(RealtimePreviewStreamConfig {
            sample_rate,
            channel_count,
            max_block_frames,
            window_size: self.window_size,
            analysis_hop: self.analysis_hop,
        })
    }

    /// Stretch an interleaved stereo buffer through the linked preview path.
    ///
    /// A trailing odd sample is ignored. This allocates and processes a whole
    /// control-side preview buffer, so callers must not use it on the audio
    /// callback.
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
        Ok(transient_reset_phase_vocoder_linked_stereo(
            even_frames,
            target_frames,
            self.ratio,
            self.window_size,
            self.analysis_hop,
        ))
    }

    /// Apply independent pitch shift and tempo stretch to one mono preview
    /// buffer.
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

    /// Apply independent pitch shift and tempo stretch to interleaved stereo
    /// preview material.
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

    /// Stretch one mono buffer with a stepwise dynamic ratio curve.
    pub fn stretch_dynamic_ratio_mono(
        &mut self,
        input: &[Sample],
        ratio_curve: &[StretchRatioPoint],
    ) -> Result<Vec<Sample>, StretchRenderError> {
        stretch_dynamic_ratio_mono_with_engine(
            input,
            ratio_curve,
            self.ratio,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        )
    }

    /// Stretch an interleaved stereo buffer with a stepwise dynamic ratio
    /// curve through the linked preview path.
    pub fn stretch_dynamic_ratio_interleaved_stereo(
        &mut self,
        frames: &[Sample],
        ratio_curve: &[StretchRatioPoint],
    ) -> Result<Vec<Sample>, StretchRenderError> {
        stretch_dynamic_ratio_linked_stereo_with_engine(
            frames,
            ratio_curve,
            self.ratio,
            self.window_size,
            self.analysis_hop,
        )
    }
}

impl TimeStretcher for RealtimePreviewStretcher {
    fn quality(&self) -> StretchQuality {
        StretchQuality::RealtimePreview
    }

    fn ratio(&self) -> f64 {
        self.ratio
    }

    fn set_ratio(&mut self, ratio: f64) {
        self.ratio = sanitize_ratio(ratio);
    }

    fn stretch_mono(&mut self, input: &[Sample]) -> Result<Vec<Sample>, StretchRenderError> {
        stretch_mono_with_engine(
            input,
            self.ratio,
            self.window_size,
            self.analysis_hop,
            transient_reset_phase_vocoder,
        )
    }
}
