use crate::{
    assess_stretch_render_integrity, measure_formant_boundary, measure_stretch_render_integrity,
    measure_tonal_texture, StretchRenderIntegrityLimits,
};

use super::super::HASH_OFFSET;

mod events;
use events::measure_events;

const SAMPLE_RATE: u32 = 44_100;

pub(super) struct Evidence {
    pub row: &'static str,
    pub ratio: f64,
    pub mode: &'static str,
    pub output_frames: usize,
    pub exact_length: bool,
    pub non_finite: usize,
    pub integrity_passed: bool,
    pub endpoint_delta_db: f64,
    pub added_silence: usize,
    pub peak_growth_db: f64,
    pub matched_events: usize,
    pub event_fallback: bool,
    pub mean_event_offset: f64,
    pub max_event_offset: f64,
    pub crest_growth_db: f64,
    pub replica_ratio: f64,
    pub tonal_movement: f64,
    pub static_residual: f64,
    pub unsupported_mass: f64,
    pub texture_delta_db: f64,
    pub formant_residual: f64,
    pub formant_shift_hz: f64,
    pub boundary_growth_db: f64,
    pub boundary_step_dbfs: f64,
    pub render_hash: u64,
    pub measurement_hash: u64,
}

pub(super) fn measure(
    row: &'static str,
    ratio: f64,
    mode: &'static str,
    source: &[f32],
    output: &[f32],
) -> Evidence {
    let expected = (source.len() as f64 * ratio).round() as usize;
    let non_finite = output.iter().filter(|sample| !sample.is_finite()).count();
    let integrity = measure_stretch_render_integrity(source, output, ratio, 2_048, 1.0e-6);
    let integrity_passed = assess_stretch_render_integrity(
        integrity,
        StretchRenderIntegrityLimits::offline_high_quality(),
    )
    .passed;
    let transient = measure_events(source, output, ratio);
    let tonal = measure_tonal_texture(source, output, ratio);
    let formant = measure_formant_boundary(source, output, ratio, SAMPLE_RATE);
    let render_hash = hash_samples(output);
    let fields = [
        output.len() as u64,
        u64::from(output.len() == expected),
        non_finite as u64,
        u64::from(integrity_passed),
        integrity.endpoint_energy_delta_db.to_bits(),
        integrity.added_silence_frames as u64,
        integrity.peak_growth_db.to_bits(),
        transient.matched as u64,
        u64::from(transient.fallback),
        transient.mean_offset.to_bits(),
        transient.max_offset.to_bits(),
        transient.crest_growth.to_bits(),
        transient.replica_ratio.to_bits(),
        tonal.spectral_modulation_delta.to_bits(),
        tonal.mean_spectral_residual_ratio.to_bits(),
        tonal.mean_added_sideband_ratio.to_bits(),
        tonal.envelope_modulation_delta_db.to_bits(),
        formant.mean_envelope_residual_ratio.to_bits(),
        formant.mean_envelope_centroid_shift_hz.to_bits(),
        formant.max_boundary_step_crest_growth_db.to_bits(),
        formant.max_boundary_step_dbfs.to_bits(),
        render_hash,
    ];
    let mut measurement_hash = HASH_OFFSET;
    hash_bytes(&mut measurement_hash, row.as_bytes());
    hash_bytes(&mut measurement_hash, mode.as_bytes());
    hash(&mut measurement_hash, ratio.to_bits());
    for field in fields {
        hash(&mut measurement_hash, field);
    }
    Evidence {
        row,
        ratio,
        mode,
        output_frames: output.len(),
        exact_length: output.len() == expected,
        non_finite,
        integrity_passed,
        endpoint_delta_db: integrity.endpoint_energy_delta_db,
        added_silence: integrity.added_silence_frames,
        peak_growth_db: integrity.peak_growth_db,
        matched_events: transient.matched,
        event_fallback: transient.fallback,
        mean_event_offset: transient.mean_offset,
        max_event_offset: transient.max_offset,
        crest_growth_db: transient.crest_growth,
        replica_ratio: transient.replica_ratio,
        tonal_movement: tonal.spectral_modulation_delta,
        static_residual: tonal.mean_spectral_residual_ratio,
        unsupported_mass: tonal.mean_added_sideband_ratio,
        texture_delta_db: tonal.envelope_modulation_delta_db,
        formant_residual: formant.mean_envelope_residual_ratio,
        formant_shift_hz: formant.mean_envelope_centroid_shift_hz,
        boundary_growth_db: formant.max_boundary_step_crest_growth_db,
        boundary_step_dbfs: formant.max_boundary_step_dbfs,
        render_hash,
        measurement_hash,
    }
}

pub(super) fn hard_pass(item: &Evidence) -> bool {
    item.exact_length && item.non_finite == 0 && item.integrity_passed
}

pub(super) fn report(evidence: &[Evidence]) -> String {
    let mut output = String::from("row\tratio\tmode\toutput_frames\texact_length\tnon_finite\tintegrity_passed\tendpoint_delta_db\tadded_silence\tpeak_growth_db\tmatched_events\tevent_fallback\tmean_event_offset_frames\tmax_event_offset_frames\tcrest_growth_db\treplica_ratio\ttonal_movement_delta\tstatic_spectral_residual\tunsupported_mass\ttexture_envelope_delta_db\tformant_residual\tformant_shift_hz\tboundary_growth_db\tboundary_step_dbfs\trender_hash\tmeasurement_hash\n");
    for item in evidence {
        output.push_str(&format!(
            "{}\t{:.6}\t{}\t{}\t{}\t{}\t{}\t{:.9}\t{}\t{:.9}\t{}\t{}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:016x}\t{:016x}\n",
            item.row,
            item.ratio,
            item.mode,
            item.output_frames,
            item.exact_length,
            item.non_finite,
            item.integrity_passed,
            item.endpoint_delta_db,
            item.added_silence,
            item.peak_growth_db,
            item.matched_events,
            item.event_fallback,
            item.mean_event_offset,
            item.max_event_offset,
            item.crest_growth_db,
            item.replica_ratio,
            item.tonal_movement,
            item.static_residual,
            item.unsupported_mass,
            item.texture_delta_db,
            item.formant_residual,
            item.formant_shift_hz,
            item.boundary_growth_db,
            item.boundary_step_dbfs,
            item.render_hash,
            item.measurement_hash,
        ));
    }
    output
}

pub(super) fn read_mono(path: &std::path::Path, frame_limit: usize) -> Vec<f32> {
    let mut reader = hound::WavReader::open(path)
        .unwrap_or_else(|error| panic!("open development audio {}: {error}", path.display()));
    let specification = reader.spec();
    assert!(matches!(specification.channels, 1 | 2));
    let interleaved = match specification.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|sample| sample.expect("development float sample"))
            .collect::<Vec<_>>(),
        hound::SampleFormat::Int => {
            let scale = 2.0_f32.powi(i32::from(specification.bits_per_sample) - 1);
            reader
                .samples::<i32>()
                .map(|sample| sample.expect("development integer sample") as f32 / scale)
                .collect::<Vec<_>>()
        }
    };
    let channels = usize::from(specification.channels);
    let frames = (interleaved.len() / channels).min(frame_limit);
    assert_eq!(frames, frame_limit, "short development audio");
    (0..frames)
        .map(|frame| {
            (0..channels)
                .map(|channel| interleaved[frame * channels + channel])
                .sum::<f32>()
                / channels as f32
        })
        .collect()
}

pub(super) fn same_samples(left: &[f32], right: &[f32]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.to_bits() == right.to_bits())
}

pub(super) fn hash_bytes(state: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        hash(state, u64::from(*byte));
    }
}

pub(super) fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}

fn hash_samples(samples: &[f32]) -> u64 {
    samples.iter().fold(HASH_OFFSET, |mut state, sample| {
        hash(&mut state, u64::from(sample.to_bits()));
        state
    })
}
