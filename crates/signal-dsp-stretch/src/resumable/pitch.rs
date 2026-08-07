use signal_dsp_resample::StreamingResampler;
use signal_primitives::{Sample, SampleRate};

use super::types::ResumableStretchConfig;

/// `2^(semitones/12)`. Resampling divides the frame count by this, so a source
/// position divides by it and a ratio multiplies by it.
pub(crate) fn pitch_shift_factor(semitones: f64) -> f64 {
    if !semitones.is_finite() || semitones.abs() < 1.0e-9 {
        return 1.0;
    }
    let factor = 2.0f64.powf(semitones / 12.0);
    if factor.is_finite() && factor > 0.0 {
        factor
    } else {
        1.0
    }
}

pub(crate) fn build_pitch_stage(
    config: &ResumableStretchConfig,
    factor: f64,
) -> Option<PitchStage> {
    if (factor - 1.0).abs() < 1.0e-12 || config.sample_rate.0 == 0 {
        return None;
    }
    // Same virtual-rate construction the whole-buffer path uses, so the pitched
    // material is identical: resample from `rate * factor` down to `rate`.
    let virtual_rate =
        ((config.sample_rate.0 as f64 * factor).round()).clamp(1.0, u32::MAX as f64) as u32;
    let resample_config = signal_dsp_resample::ResampleConfig::new(
        SampleRate(virtual_rate),
        config.sample_rate,
        signal_dsp_resample::ResampleQuality::BandLimited,
    );
    Some(PitchStage {
        mid: StreamingResampler::new(resample_config),
        side: (config.channels == 2).then(|| StreamingResampler::new(resample_config)),
        mid_scratch: Vec::new(),
        side_scratch: Vec::new(),
        pitched: Vec::new(),
        carry: Vec::new(),
    })
}

/// Resample stage that carries its state across chunk boundaries.
///
/// `signal-dsp-resample` already provides the carry: `StreamingResampler` holds
/// a pending history buffer and a fractional source cursor, which is exactly
/// what a chunk boundary destroys. `resample_mono`, which the whole-buffer pitch
/// path calls, is a thin wrapper over it — so this writes no resampling.
pub(crate) struct PitchStage {
    /// Mid for stereo, or the single channel for mono.
    mid: StreamingResampler,
    /// Side for stereo only.
    side: Option<StreamingResampler>,
    mid_scratch: Vec<Sample>,
    side_scratch: Vec<Sample>,
    pitched: Vec<Sample>,
    /// Pitched frames produced but not yet accepted by the stretch stage.
    ///
    /// The ring-feed loop can exit with frames outstanding when a drain cannot
    /// progress. Without this they would be dropped, because the pitched buffer
    /// is rebuilt from fresh source on the next call — a source drop whose size
    /// depends on the caller's chunking, which is exactly what chunk-count
    /// independence forbids.
    pub(crate) carry: Vec<Sample>,
}

impl PitchStage {
    /// Resample one interleaved chunk into `self.pitched`.
    ///
    /// `finish` drains the resamplers' tails instead of feeding them more.
    pub(crate) fn process(
        &mut self,
        source: &[Sample],
        channels: usize,
        finish: bool,
    ) -> &[Sample] {
        self.pitched.clear();
        self.pitched.extend_from_slice(&self.carry);
        self.carry.clear();
        if channels == 2 {
            let frames = source.len() / 2;
            self.mid_scratch.clear();
            self.side_scratch.clear();
            for frame in source[..frames * 2].chunks_exact(2) {
                self.mid_scratch.push((frame[0] + frame[1]) * 0.5);
                self.side_scratch.push((frame[0] - frame[1]) * 0.5);
            }
            let mid = if finish {
                self.mid.finish()
            } else {
                self.mid.process_chunk(&self.mid_scratch)
            };
            let side = match self.side.as_mut() {
                Some(side) if finish => side.finish(),
                Some(side) => side.process_chunk(&self.side_scratch),
                None => Vec::new(),
            };
            let count = mid.len().min(side.len());
            for index in 0..count {
                self.pitched.push(mid[index] + side[index]);
                self.pitched.push(mid[index] - side[index]);
            }
        } else {
            let produced = if finish {
                self.mid.finish()
            } else {
                self.mid.process_chunk(source)
            };
            self.pitched.extend_from_slice(&produced);
        }
        &self.pitched
    }
}
