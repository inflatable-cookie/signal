use std::{fs, path::PathBuf};

use crate::{
    assess_stretch_render_integrity, detect_stretch_transients_with_policy,
    measure_formant_boundary, measure_stretch_render_integrity, measure_tonal_texture,
    measure_transient_detail, OfflineHighQualityPath, OfflineHighQualityStretcher,
    StretchRenderIntegrityLimits, StretchTransientDetectorPolicy, TimeStretcher,
};

use super::super::{
    complete_system_tuning::listening_export::manifest::{render_root, rows, source_root},
    study_local_schedule::{
        schedule::build_schedule,
        study::{analyze, select},
        BASE_HOP, SOURCE_FRAMES,
    },
    HASH_OFFSET,
};
use super::{anchors::detect, render::render_successor_owned};

const SAMPLE_RATE: u32 = 44_100;
const TRANSIENT_WINDOW: usize = 1_024;
const TRANSIENT_HOP: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum DevelopmentDirection {
    ConcealedDevelopmentComparison,
    OwningMechanism,
    SpectralSynthesisAttribution,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct DevelopmentObjectiveReview {
    pub rows: usize,
    pub modes: usize,
    pub renders: usize,
    pub holdout_reads: usize,
    pub hard_failures: usize,
    pub candidate_hard_failures: usize,
    pub candidate_changed_rows: usize,
    pub event_fallback_renders: usize,
    pub candidate_regression_rows: [usize; 4],
    pub hashes: [u64; 4],
    pub direction: DevelopmentDirection,
}

#[derive(Clone, Copy)]
enum Mode {
    Current,
    Successor,
    External,
}

impl Mode {
    fn id(self) -> &'static str {
        match self {
            Self::Current => "current-signal",
            Self::Successor => "event-owned-successor",
            Self::External => "rubber-band-r3",
        }
    }
}

struct Evidence {
    row: &'static str,
    ratio: f64,
    mode: Mode,
    output_frames: usize,
    exact_length: bool,
    non_finite: usize,
    integrity_passed: bool,
    endpoint_delta_db: f64,
    added_silence: usize,
    peak_growth_db: f64,
    matched_events: usize,
    event_fallback: bool,
    mean_event_offset: f64,
    max_event_offset: f64,
    crest_growth_db: f64,
    replica_ratio: f64,
    tonal_movement: f64,
    static_residual: f64,
    unsupported_mass: f64,
    texture_delta_db: f64,
    formant_residual: f64,
    formant_shift_hz: f64,
    boundary_growth_db: f64,
    boundary_step_dbfs: f64,
    render_hash: u64,
    measurement_hash: u64,
}

pub(in crate::frequency_adaptive) fn development_objective_review() -> DevelopmentObjectiveReview {
    let mut evidence = Vec::with_capacity(27);
    let mut manifest_hash = HASH_OFFSET;
    let mut render_hash = HASH_OFFSET;
    let mut measurement_hash = HASH_OFFSET;
    let mut candidate_changed_rows = 0;

    for row in rows() {
        let source = read_mono(&source_root().join(&row.source), Some(SOURCE_FRAMES));
        let expected = (SOURCE_FRAMES as f64 * row.ratio).round() as usize;
        let current = render_current(&source, row.ratio);
        let successor = render_successor(&source, row.ratio);
        let external = read_mono(&render_root().join(&row.rubber_band), Some(expected));
        candidate_changed_rows += usize::from(!same_samples(&successor, &current));

        hash_bytes(&mut manifest_hash, row.id.as_bytes());
        hash(&mut manifest_hash, row.ratio.to_bits());
        hash_bytes(&mut manifest_hash, row.source.as_bytes());
        hash_bytes(&mut manifest_hash, row.rubber_band.as_bytes());

        for (mode, output) in [
            (Mode::Current, current),
            (Mode::Successor, successor),
            (Mode::External, external),
        ] {
            let item = measure(row.id, row.ratio, mode, &source, &output);
            hash(&mut render_hash, item.render_hash);
            hash(&mut measurement_hash, item.measurement_hash);
            evidence.push(item);
        }
    }

    let report = report(&evidence);
    let path = report_path();
    fs::create_dir_all(path.parent().expect("development report parent"))
        .expect("create development report directory");
    fs::write(&path, &report).expect("write development objective report");
    let mut aggregate_hash = HASH_OFFSET;
    for value in [manifest_hash, render_hash, measurement_hash] {
        hash(&mut aggregate_hash, value);
    }
    hash_bytes(&mut aggregate_hash, report.as_bytes());

    let hard_failures = evidence.iter().filter(|item| !hard_pass(item)).count();
    let candidate_hard_failures = evidence
        .iter()
        .filter(|item| matches!(item.mode, Mode::Successor) && !hard_pass(item))
        .count();
    let event_fallback_renders = evidence.iter().filter(|item| item.event_fallback).count();
    let mut candidate_regression_rows = [0; 4];
    for modes in evidence.chunks_exact(3) {
        let current = &modes[0];
        let candidate = &modes[1];
        candidate_regression_rows[0] +=
            usize::from(candidate.mean_event_offset > current.mean_event_offset);
        candidate_regression_rows[1] +=
            usize::from(candidate.replica_ratio > current.replica_ratio);
        candidate_regression_rows[2] +=
            usize::from(candidate.static_residual > current.static_residual);
        candidate_regression_rows[3] +=
            usize::from(candidate.formant_residual > current.formant_residual);
    }
    let broad_objective_regression = candidate_regression_rows[0] >= 5
        && candidate_regression_rows[1] >= 5
        && candidate_regression_rows[2] == 9
        && candidate_regression_rows[3] == 9;
    DevelopmentObjectiveReview {
        rows: 9,
        modes: 3,
        renders: evidence.len(),
        holdout_reads: 0,
        hard_failures,
        candidate_hard_failures,
        candidate_changed_rows,
        event_fallback_renders,
        candidate_regression_rows,
        hashes: [manifest_hash, render_hash, measurement_hash, aggregate_hash],
        direction: if candidate_hard_failures != 0 {
            DevelopmentDirection::OwningMechanism
        } else if broad_objective_regression {
            DevelopmentDirection::SpectralSynthesisAttribution
        } else {
            DevelopmentDirection::ConcealedDevelopmentComparison
        },
    }
}

fn render_current(source: &[f32], ratio: f64) -> Vec<f32> {
    OfflineHighQualityStretcher::with_path(ratio, OfflineHighQualityPath::Default)
        .stretch_mono(source)
}

fn render_successor(source: &[f32], ratio: f64) -> Vec<f32> {
    let channels = vec![source
        .iter()
        .map(|sample| f64::from(*sample))
        .collect::<Vec<_>>()];
    let study = analyze(&channels, SOURCE_FRAMES);
    let points = select(&study, 3.0, 2);
    let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &points);
    let anchors = detect(&channels, SOURCE_FRAMES);
    render_successor_owned(&channels, ratio, &points, &anchors.positions, &schedule)
        .samples
        .remove(0)
        .into_iter()
        .map(|sample| sample as f32)
        .collect()
}

fn measure(row: &'static str, ratio: f64, mode: Mode, source: &[f32], output: &[f32]) -> Evidence {
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
    hash_bytes(&mut measurement_hash, mode.id().as_bytes());
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

fn hard_pass(item: &Evidence) -> bool {
    item.exact_length && item.non_finite == 0 && item.integrity_passed
}

struct EventEvidence {
    matched: usize,
    fallback: bool,
    mean_offset: f64,
    max_offset: f64,
    crest_growth: f64,
    replica_ratio: f64,
}

fn measure_events(source: &[f32], output: &[f32], ratio: f64) -> EventEvidence {
    let detail = measure_transient_detail(source, output, ratio, TRANSIENT_WINDOW, TRANSIENT_HOP);
    if detail.matched_transients > 0 {
        return EventEvidence {
            matched: detail.matched_transients,
            fallback: false,
            mean_offset: detail.mean_absolute_timing_offset_frames,
            max_offset: detail.max_absolute_timing_offset_frames,
            crest_growth: detail.max_transient_crest_growth_db,
            replica_ratio: replica_at(output, detail.max_crest_output_frame),
        };
    }
    let mut events = detect_stretch_transients_with_policy(
        source,
        TRANSIENT_WINDOW,
        TRANSIENT_HOP,
        StretchTransientDetectorPolicy::production(),
    )
    .into_iter()
    .map(|event| event.frame_index)
    .collect::<Vec<_>>();
    if events.is_empty() {
        events.push(strongest_onset(source));
    }
    let mut offset_sum = 0.0;
    let mut max_offset = 0.0_f64;
    let mut crest_growth = f64::NEG_INFINITY;
    let mut replica_ratio = 0.0_f64;
    for source_event in &events {
        let expected = (*source_event as f64 * ratio).round() as usize;
        let output_event = peak_index(output, expected, 512);
        let offset = output_event.abs_diff(expected) as f64;
        offset_sum += offset;
        max_offset = max_offset.max(offset);
        crest_growth = crest_growth.max(
            20.0 * (local_crest(output, output_event) / local_crest(source, *source_event)).log10(),
        );
        let secondary_start = (output_event + 65).min(output.len());
        let secondary_end = (output_event + 513).min(output.len());
        replica_ratio = replica_ratio.max(
            peak(&output[secondary_start..secondary_end])
                / f64::from(output[output_event].abs()).max(1.0e-12),
        );
    }
    EventEvidence {
        matched: events.len(),
        fallback: true,
        mean_offset: offset_sum / events.len() as f64,
        max_offset,
        crest_growth,
        replica_ratio,
    }
}

fn replica_at(output: &[f32], event: usize) -> f64 {
    let secondary_start = (event + 65).min(output.len());
    let secondary_end = (event + 513).min(output.len());
    peak(&output[secondary_start..secondary_end]) / f64::from(output[event].abs()).max(1.0e-12)
}

fn strongest_onset(samples: &[f32]) -> usize {
    (1..samples.len())
        .max_by(|left, right| {
            let left_rise = (samples[*left] - samples[*left - 1]).abs();
            let right_rise = (samples[*right] - samples[*right - 1]).abs();
            left_rise.total_cmp(&right_rise)
        })
        .unwrap_or(0)
}

fn peak_index(samples: &[f32], center: usize, radius: usize) -> usize {
    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(samples.len());
    (start..end)
        .max_by(|left, right| samples[*left].abs().total_cmp(&samples[*right].abs()))
        .unwrap_or(start)
}

fn local_crest(samples: &[f32], center: usize) -> f64 {
    let start = center.saturating_sub(128);
    let end = (center + 129).min(samples.len());
    let span = &samples[start..end];
    let rms = (span
        .iter()
        .map(|sample| f64::from(*sample) * f64::from(*sample))
        .sum::<f64>()
        / span.len().max(1) as f64)
        .sqrt();
    peak(span) / rms.max(1.0e-12)
}

fn peak(samples: &[f32]) -> f64 {
    samples
        .iter()
        .map(|sample| sample.abs() as f64)
        .fold(0.0, f64::max)
}

fn report(evidence: &[Evidence]) -> String {
    let mut output = String::from("row\tratio\tmode\toutput_frames\texact_length\tnon_finite\tintegrity_passed\tendpoint_delta_db\tadded_silence\tpeak_growth_db\tmatched_events\tevent_fallback\tmean_event_offset_frames\tmax_event_offset_frames\tcrest_growth_db\treplica_ratio\ttonal_movement_delta\tstatic_spectral_residual\tunsupported_mass\ttexture_envelope_delta_db\tformant_residual\tformant_shift_hz\tboundary_growth_db\tboundary_step_dbfs\trender_hash\tmeasurement_hash\n");
    for item in evidence {
        output.push_str(&format!(
            "{}\t{:.6}\t{}\t{}\t{}\t{}\t{}\t{:.9}\t{}\t{:.9}\t{}\t{}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:016x}\t{:016x}\n",
            item.row,
            item.ratio,
            item.mode.id(),
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

fn read_mono(path: &std::path::Path, frame_limit: Option<usize>) -> Vec<f32> {
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
    let frames = (interleaved.len() / channels).min(frame_limit.unwrap_or(usize::MAX));
    assert_eq!(
        frames,
        frame_limit.unwrap_or(frames),
        "short development audio"
    );
    (0..frames)
        .map(|frame| {
            (0..channels)
                .map(|channel| interleaved[frame * channels + channel])
                .sum::<f32>()
                / channels as f32
        })
        .collect()
}

fn same_samples(left: &[f32], right: &[f32]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.to_bits() == right.to_bits())
}

fn hash_samples(samples: &[f32]) -> u64 {
    samples.iter().fold(HASH_OFFSET, |mut state, sample| {
        hash(&mut state, u64::from(sample.to_bits()));
        state
    })
}

fn hash_bytes(state: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        hash(state, u64::from(*byte));
    }
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}

fn report_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-successor-by-development-objective.tsv")
}
