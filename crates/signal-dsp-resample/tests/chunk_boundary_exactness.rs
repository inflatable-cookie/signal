//! `A24`: `StreamingResampler` is not bit-exact across chunk boundaries.
//!
//! Found by `g10.042` Batch 42.3, where pitched resumable renders failed
//! chunk-count independence. The renderer was correct; this was the source.

use signal_dsp_resample::{ResampleConfig, ResampleQuality, StreamingResampler};
use signal_primitives::SampleRate;

fn tone(frames: usize) -> Vec<f32> {
    (0..frames)
        .map(|index| 0.4 * (std::f32::consts::TAU * 220.0 * index as f32 / 48_000.0).sin())
        .collect()
}

fn resample_in_chunks(config: ResampleConfig, input: &[f32], chunks: usize) -> Vec<f32> {
    let mut resampler = StreamingResampler::new(config);
    let mut output = Vec::new();
    let step = (input.len() / chunks).max(1);
    let mut start = 0;
    while start < input.len() {
        let end = (start + step).min(input.len());
        output.extend(resampler.process_chunk(&input[start..end]));
        start = end;
    }
    output.extend(resampler.finish());
    output
}

/// Output length does not depend on chunking. This holds today.
#[test]
fn chunked_and_whole_resampling_agree_on_length() {
    let config = ResampleConfig::new(
        SampleRate(53_996),
        SampleRate(48_000),
        ResampleQuality::BandLimited,
    );
    let input = tone(96_000);
    let whole = resample_in_chunks(config, &input, 1);
    for chunks in [2usize, 3, 7, 16] {
        assert_eq!(
            whole.len(),
            resample_in_chunks(config, &input, chunks).len(),
            "{chunks} chunks changed the output length"
        );
    }
}

/// `A24`, fixed 2026-08-05.
///
/// The read position is derived from the absolute output index rather than
/// accumulated and rebased, so it cannot depend on chunking. Previously the
/// difference was one ULP, `2.98e-8`, arriving exactly at the first seam —
/// unremarkable alone, but a phase vocoder downstream amplifies it by roughly
/// `190000x`, which is what blocked bit-exact chunk independence for pitched
/// renders.
#[test]
fn chunked_and_whole_resampling_are_bit_exact() {
    let config = ResampleConfig::new(
        SampleRate(53_996),
        SampleRate(48_000),
        ResampleQuality::BandLimited,
    );
    let input = tone(96_000);
    let whole = resample_in_chunks(config, &input, 1);
    for chunks in [2usize, 3, 7] {
        let chunked = resample_in_chunks(config, &input, chunks);
        let first = (0..whole.len().min(chunked.len())).position(|i| whole[i] != chunked[i]);
        assert!(
            first.is_none(),
            "{chunks} chunks first differ at sample {first:?}"
        );
    }
}
