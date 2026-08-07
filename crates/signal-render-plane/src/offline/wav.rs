//! WAV writing with TPDF dither.

use std::path::Path;

use crate::RenderPlaneError;

// ── WAV writing with TPDF dither ────────────────────────────────────────────

/// Output bit depth for [`write_wav`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WavBitDepth {
    /// 32-bit IEEE float: bit-transparent, no dither needed.
    Float32,
    /// 24-bit integer with TPDF dither.
    Int24,
    /// 16-bit integer with TPDF dither.
    Int16,
}

/// Minimal LCG for dither noise: deterministic, dependency-free, never used
/// for anything security- or statistics-critical beyond decorrelating
/// quantization error.
struct DitherLcg(u64);

impl DitherLcg {
    /// Uniform sample in `[0, 1)`.
    fn next_unit(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 40) as f32) / (1u64 << 24) as f32
    }

    /// TPDF sample in `(-1, 1)` LSB: sum of two independent uniforms,
    /// triangular probability density centered on zero.
    fn next_tpdf(&mut self) -> f32 {
        self.next_unit() + self.next_unit() - 1.0
    }
}

/// Write interleaved f32 PCM to a WAV file at `bit_depth`.
///
/// Integer depths apply TPDF dither (±1 LSB triangular, two independent
/// uniform randoms per sample from a constant-seeded LCG — no `rand`
/// dependency) before quantization, then clamp to the integer range.
/// `Float32` writes samples bit-exactly.
pub fn write_wav(
    path: &Path,
    samples: &[f32],
    channels: u16,
    sample_rate_hz: u32,
    bit_depth: WavBitDepth,
) -> Result<(), RenderPlaneError> {
    let io_error = |error: hound::Error| RenderPlaneError {
        message: format!("wav write failed: {error}"),
    };
    let spec = hound::WavSpec {
        channels: channels.max(1),
        sample_rate: sample_rate_hz.max(1),
        bits_per_sample: match bit_depth {
            WavBitDepth::Float32 => 32,
            WavBitDepth::Int24 => 24,
            WavBitDepth::Int16 => 16,
        },
        sample_format: match bit_depth {
            WavBitDepth::Float32 => hound::SampleFormat::Float,
            WavBitDepth::Int24 | WavBitDepth::Int16 => hound::SampleFormat::Int,
        },
    };
    let mut writer = hound::WavWriter::create(path, spec).map_err(io_error)?;
    match bit_depth {
        WavBitDepth::Float32 => {
            for sample in samples {
                writer.write_sample(*sample).map_err(io_error)?;
            }
        }
        WavBitDepth::Int24 => {
            let mut lcg = DitherLcg(0x0FF1_CED1_74E2_u64);
            const SCALE: f32 = 8_388_608.0; // 2^23
            for sample in samples {
                let dithered = sample * SCALE + lcg.next_tpdf();
                let quantized = dithered.round().clamp(-SCALE, SCALE - 1.0) as i32;
                writer.write_sample(quantized).map_err(io_error)?;
            }
        }
        WavBitDepth::Int16 => {
            let mut lcg = DitherLcg(0x0FF1_CED1_74E2_u64);
            const SCALE: f32 = 32_768.0; // 2^15
            for sample in samples {
                let dithered = sample * SCALE + lcg.next_tpdf();
                let quantized = dithered.round().clamp(-SCALE, SCALE - 1.0) as i16;
                writer.write_sample(quantized).map_err(io_error)?;
            }
        }
    }
    writer.finalize().map_err(io_error)
}
