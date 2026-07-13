use rustfft::{num_complex::Complex64, FftPlanner};

use super::{
    complete_system_tuning::{
        listening_export::{
            audio::read_mono,
            manifest::{render_root, rows},
        },
        objective_grid::{
            audio::{development_cases, synthetic_control, DevelopmentCase},
            metrics::{event_error, identity_error, quality, tone_error},
        },
    },
    study_local_schedule::{
        schedule::{build_schedule, Schedule},
        study::{analyze_with_geometry, select},
    },
    HASH_OFFSET,
};
use crate::{OfflineHighQualityPath, OfflineHighQualityStretcher, TimeStretcher};
use std::fs;

const SAMPLE_RATE: f64 = 48_000.0;
const BASE_HOP: usize = 128;
const GEOMETRY: [usize; 3] = [1_024, 2_048, 4_096];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Architecture {
    FrequencyPartitioned,
    WeightedPredictor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Direction {
    MonoDecisionCheckpoint,
    ArchitectureResearch,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ArchitectureEvidence {
    pub architecture: Architecture,
    pub synthetic_failures: [usize; 8],
    pub development_failures: [usize; 4],
    pub state_counts: [usize; 6],
    pub frequency_owner_counts: [usize; 3],
    pub crossover_ranges_hz: [f64; 4],
    pub synthetic_quality: [f64; 2],
    pub mean_quality: [f64; 5],
    pub output_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ComparatorEvidence {
    pub name: &'static str,
    pub available_rows: usize,
    pub structural_failures: usize,
    pub mean_quality: [f64; 5],
    pub output_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Review {
    pub geometry: [usize; 3],
    pub architecture: [ArchitectureEvidence; 2],
    pub comparators: [ComparatorEvidence; 3],
    pub development_rows: usize,
    pub holdout_reads: usize,
    pub repeated: bool,
    pub direction: Direction,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ExportReview {
    pub rows: usize,
    pub candidates_per_row: usize,
    pub audio_files: usize,
    pub holdout_reads: usize,
    pub structural_failures: [usize; 4],
    pub hashes: [u64; 3],
}

#[derive(Clone, Debug)]
struct Render {
    samples: Vec<Vec<f64>>,
    target_len: usize,
    uncovered: usize,
    non_finite: usize,
    boundary_failures: usize,
    owner_failures: usize,
    state_counts: [usize; 6],
    owner_counts: [usize; 3],
    crossover_ranges_hz: [f64; 4],
    hash: u64,
}

#[derive(Clone)]
struct PhaseState {
    analysis: Vec<f64>,
    synthesis: Vec<f64>,
    source: Option<isize>,
    output: Option<isize>,
}

impl PhaseState {
    fn new(length: usize) -> Self {
        Self {
            analysis: vec![0.0; length / 2 + 1],
            synthesis: vec![0.0; length / 2 + 1],
            source: None,
            output: None,
        }
    }
}

#[derive(Clone)]
struct Guidance {
    low_hz: f64,
    high_hz: f64,
    attack: bool,
}

pub(super) fn review() -> Review {
    let first = run();
    let repeated_review = run();
    let repeated = first.architecture == repeated_review.architecture
        && first.comparators == repeated_review.comparators;
    let pass = repeated
        && first
            .architecture
            .iter()
            .all(|item| item.synthetic_failures == [0; 8])
        && first
            .architecture
            .iter()
            .all(|item| item.development_failures == [0; 4])
        && first
            .comparators
            .iter()
            .all(|item| item.available_rows == 9)
        && first
            .comparators
            .iter()
            .all(|item| item.structural_failures == 0);
    Review {
        repeated,
        direction: if pass {
            Direction::MonoDecisionCheckpoint
        } else {
            Direction::ArchitectureResearch
        },
        ..first
    }
}

pub(super) fn export_development_pack() -> ExportReview {
    use super::complete_system_tuning::listening_export::{
        audio::{level_match, write_mono},
        manifest::assignment,
    };

    let evidence = review();
    let integrity_passed = evidence.architecture.iter().all(|item| {
        item.synthetic_failures[..7]
            .iter()
            .all(|failure| *failure == 0)
            && item.development_failures == [0; 4]
    });
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-source-studied-ch-development-pack");
    if root.exists() {
        fs::remove_dir_all(&root).expect("replace source-studied development pack");
    }
    fs::create_dir_all(root.join("references")).expect("create source-studied references");
    fs::create_dir_all(root.join("trials")).expect("create source-studied trials");
    let mut notes = String::from(
        "row\tratio\tsource\tA\tB\tC\tD\tE\ttransient\ttonal\tgrain_ringing\tboundary\tpreference\tbroad_defect\tnotes\tcompleted\n",
    );
    let mut key = String::from("row\tratio\tletter\tidentity\tgain\n");
    let mut failures = [0; 4];
    let mut hashes = [HASH_OFFSET; 3];
    let mut audio_files = 0;
    if integrity_passed {
        for (row, case) in rows().into_iter().zip(development_cases()) {
            let source = &case.channels[0];
            let target = (source.len() as f64 * row.ratio).round() as usize;
            let mut candidates = vec![
                (
                    "signal-frequency-partitioned".to_string(),
                    execute(
                        std::slice::from_ref(source),
                        row.ratio,
                        Architecture::FrequencyPartitioned,
                    )
                    .samples
                    .remove(0),
                ),
                (
                    "signal-weighted-predictor".to_string(),
                    execute(
                        std::slice::from_ref(source),
                        row.ratio,
                        Architecture::WeightedPredictor,
                    )
                    .samples
                    .remove(0),
                ),
                (
                    "current-signal".to_string(),
                    render_current(source, row.ratio),
                ),
            ];
            let mut rubber = read_mono(&render_root().join(&row.rubber_band));
            rubber.truncate(target);
            candidates.push(("rubber-band-r3".to_string(), rubber));
            let signalsmith_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/stretch-source-studied-signalsmith");
            let mut signalsmith = read_mono(&signalsmith_root.join(&row.rubber_band));
            signalsmith.truncate(target);
            candidates.push(("signalsmith-stretch-1.3.2".to_string(), signalsmith));
            failures[0] += candidates
                .iter()
                .filter(|(_, samples)| samples.len() != target)
                .count();
            failures[1] += candidates
                .iter()
                .flat_map(|(_, samples)| samples)
                .filter(|sample| !sample.is_finite())
                .count();
            let matched = level_match(source, candidates);
            let source_name = format!("{}-source.wav", row.id);
            write_mono(
                &root.join("references").join(&source_name),
                44_100,
                &matched.source,
            );
            audio_files += 1;
            let assignment = assignment(row.id, matched.candidates.len());
            let mut trial_names = Vec::with_capacity(assignment.len());
            for (letter_index, candidate_index) in assignment.into_iter().enumerate() {
                let letter = char::from(b'A' + letter_index as u8);
                let candidate = &matched.candidates[candidate_index];
                let name = format!("{}-{letter}.wav", row.id);
                write_mono(&root.join("trials").join(&name), 44_100, &candidate.samples);
                audio_files += 1;
                trial_names.push(format!("trials/{name}"));
                key.push_str(&format!(
                    "{}\t{:.6}\t{letter}\t{}\t{:.9}\n",
                    row.id, row.ratio, candidate.identity, candidate.gain
                ));
                hash_bytes(&mut hashes[0], candidate.identity.as_bytes());
                mix(&mut hashes[1], candidate.gain.to_bits());
            }
            notes.push_str(&format!(
                "{}\t{:.6}\treferences/{source_name}\t{}\t{}\t{}\t{}\t{}\t\t\t\t\t\t\t\tfalse\n",
                row.id,
                row.ratio,
                trial_names[0],
                trial_names[1],
                trial_names[2],
                trial_names[3],
                trial_names[4],
            ));
        }
    }
    failures[2] = usize::from(!integrity_passed);
    failures[3] = usize::from(audio_files != 54);
    fs::write(root.join("development-listening-notes.tsv"), &notes)
        .expect("write source-studied notes");
    fs::write(root.join("development-listening-key.tsv"), &key).expect("write source-studied key");
    fs::write(
        root.join("README.md"),
        "# Source-Studied Stretch Development Pack\n\nStatus: ready for concealed operator listening\n\nNine mono rows. Each row has source plus candidates A-E. Compare transient integrity, tonal stability, grain/ringing, and boundaries. Record a preference and any repeatable broad defect. Keep `development-listening-key.tsv` closed until every row is complete. Candidates are Signal frequency-partitioned, Signal fixed-grid weighted predictor, current Signal, Rubber Band R3, and Signalsmith Stretch 1.3.2. All trials are RMS matched with a shared peak ceiling. Holdout audio is absent.\n",
    )
    .expect("write source-studied readme");
    hash_bytes(&mut hashes[2], notes.as_bytes());
    ExportReview {
        rows: 9,
        candidates_per_row: 5,
        audio_files,
        holdout_reads: 0,
        structural_failures: failures,
        hashes,
    }
}

fn run() -> Review {
    let synthetic = synthetic_control();
    let development = development_cases();
    let architecture = [
        evaluate_architecture(Architecture::FrequencyPartitioned, &synthetic, &development),
        evaluate_architecture(Architecture::WeightedPredictor, &synthetic, &development),
    ];
    let comparators = [
        evaluate_current(&development),
        evaluate_external("rubber-band-r3", render_root(), &development),
        evaluate_external(
            "signalsmith-stretch-1.3.2",
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/stretch-source-studied-signalsmith"),
            &development,
        ),
    ];
    Review {
        geometry: GEOMETRY,
        architecture,
        comparators,
        development_rows: development.len(),
        holdout_reads: 0,
        repeated: false,
        direction: Direction::ArchitectureResearch,
    }
}

fn evaluate_architecture(
    architecture: Architecture,
    synthetic: &[Vec<f64>],
    development: &[DevelopmentCase],
) -> ArchitectureEvidence {
    let stretched = execute(synthetic, 1.5, architecture);
    let repeated = execute(synthetic, 1.5, architecture);
    let identity = execute(synthetic, 1.0, architecture);
    let synthetic_failures = [
        usize::from(identity_error(&synthetic[0], &identity.samples[0]) > 5.0e-12),
        usize::from(stretched.samples[0].len() != stretched.target_len),
        stretched.uncovered,
        stretched.non_finite,
        stretched.boundary_failures,
        stretched.owner_failures,
        usize::from(stretched.hash != repeated.hash),
        usize::from(
            tone_error(&stretched.samples[0]) > 2.0
                || event_error(&stretched.samples[0], 1.5) > 256,
        ),
    ];
    let synthetic_quality = [
        tone_error(&stretched.samples[0]),
        event_error(&stretched.samples[0], 1.5) as f64,
    ];
    let mut development_failures = [0; 4];
    let mut qualities = Vec::with_capacity(development.len());
    let mut output_hash = stretched.hash;
    let mut state_counts = stretched.state_counts;
    let mut owner_counts = stretched.owner_counts;
    let mut crossover_ranges = stretched.crossover_ranges_hz;
    for case in development {
        let render = execute(&case.channels, case.ratio, architecture);
        development_failures[0] += usize::from(render.samples[0].len() != render.target_len);
        development_failures[1] += render.uncovered;
        development_failures[2] += render.non_finite;
        development_failures[3] += render.boundary_failures + render.owner_failures;
        qualities.push(quality(&case.channels[0], &render.samples[0], case.ratio));
        mix(&mut output_hash, render.hash);
        for (total, count) in state_counts.iter_mut().zip(render.state_counts) {
            *total += count;
        }
        for (total, count) in owner_counts.iter_mut().zip(render.owner_counts) {
            *total += count;
        }
        crossover_ranges[0] = crossover_ranges[0].min(render.crossover_ranges_hz[0]);
        crossover_ranges[1] = crossover_ranges[1].max(render.crossover_ranges_hz[1]);
        crossover_ranges[2] = crossover_ranges[2].min(render.crossover_ranges_hz[2]);
        crossover_ranges[3] = crossover_ranges[3].max(render.crossover_ranges_hz[3]);
    }
    ArchitectureEvidence {
        architecture,
        synthetic_failures,
        development_failures,
        state_counts,
        frequency_owner_counts: owner_counts,
        crossover_ranges_hz: crossover_ranges,
        synthetic_quality,
        mean_quality: mean_quality(&qualities),
        output_hash,
    }
}

fn evaluate_current(development: &[DevelopmentCase]) -> ComparatorEvidence {
    let mut qualities = Vec::with_capacity(development.len());
    let mut output_hash = HASH_OFFSET;
    let mut failures = 0;
    for case in development {
        let output = render_current(&case.channels[0], case.ratio);
        failures += usize::from(
            output.len() != (case.channels[0].len() as f64 * case.ratio).round() as usize,
        );
        failures += output.iter().filter(|sample| !sample.is_finite()).count();
        qualities.push(quality(&case.channels[0], &output, case.ratio));
        hash_samples(&mut output_hash, &output);
    }
    ComparatorEvidence {
        name: "current-signal",
        available_rows: development.len(),
        structural_failures: failures,
        mean_quality: mean_quality(&qualities),
        output_hash,
    }
}

fn render_current(source: &[f64], ratio: f64) -> Vec<f64> {
    let input = source
        .iter()
        .map(|sample| *sample as f32)
        .collect::<Vec<_>>();
    let mut stretcher =
        OfflineHighQualityStretcher::with_path(ratio, OfflineHighQualityPath::Default);
    stretcher
        .stretch_mono(&input)
        .into_iter()
        .map(f64::from)
        .collect()
}

fn evaluate_external(
    name: &'static str,
    root: std::path::PathBuf,
    development: &[DevelopmentCase],
) -> ComparatorEvidence {
    let manifest_rows = rows();
    let mut qualities = Vec::new();
    let mut output_hash = HASH_OFFSET;
    let mut failures = 0;
    for (case, row) in development.iter().zip(manifest_rows) {
        let path = if name == "rubber-band-r3" {
            root.join(row.rubber_band)
        } else {
            root.join(row.rubber_band)
        };
        if !path.exists() {
            continue;
        }
        let mut output = read_mono(&path);
        let target = (case.channels[0].len() as f64 * case.ratio).round() as usize;
        output.truncate(target);
        failures += usize::from(output.len() != target);
        failures += output.iter().filter(|sample| !sample.is_finite()).count();
        qualities.push(quality(&case.channels[0], &output, case.ratio));
        hash_samples(&mut output_hash, &output);
    }
    ComparatorEvidence {
        name,
        available_rows: qualities.len(),
        structural_failures: failures,
        mean_quality: mean_quality(&qualities),
        output_hash,
    }
}

fn execute(channels: &[Vec<f64>], ratio: f64, architecture: Architecture) -> Render {
    if ratio == 1.0 {
        let samples = channels.to_vec();
        let mut hash = HASH_OFFSET;
        for channel in &samples {
            hash_samples(&mut hash, channel);
        }
        return Render {
            target_len: samples[0].len(),
            samples,
            uncovered: 0,
            non_finite: 0,
            boundary_failures: 0,
            owner_failures: 0,
            state_counts: [0; 6],
            owner_counts: [0; 3],
            crossover_ranges_hz: [0.0; 4],
            hash,
        };
    }
    let study = analyze_with_geometry(channels, channels[0].len(), GEOMETRY);
    let points = select(&study, 3.0, 2);
    let schedule = build_schedule(channels[0].len(), BASE_HOP, ratio, &points);
    match architecture {
        Architecture::FrequencyPartitioned => render_partitioned(channels, ratio, &schedule),
        Architecture::WeightedPredictor => render_predictor(channels, ratio, &schedule),
    }
}

fn render_partitioned(channels: &[Vec<f64>], ratio: f64, schedule: &Schedule) -> Render {
    let target_len = (ratio * channels[0].len() as f64).round() as usize;
    let longest = GEOMETRY[2];
    let centers = centers(channels[0].len(), longest);
    let output_centers = centers
        .iter()
        .map(|center| project(*center, channels[0].len(), ratio, schedule))
        .collect::<Vec<_>>();
    let output_start = output_centers[0] - longest as isize / 2;
    let output_end = output_centers[output_centers.len() - 1] + longest as isize / 2;
    let domain_len = (output_end - output_start) as usize;
    let mut outputs = std::array::from_fn::<_, 3, _>(|_| {
        vec![vec![Complex64::new(0.0, 0.0); domain_len]; channels.len()]
    });
    let mut operators = std::array::from_fn::<_, 3, _>(|_| vec![0.0; domain_len]);
    let mut states = (0..channels.len())
        .map(|_| std::array::from_fn(|layer| PhaseState::new(GEOMETRY[layer])))
        .collect::<Vec<_>>();
    let mut previous_classifier = vec![0.0; GEOMETRY[1] / 2 + 1];
    let mut planner = FftPlanner::<f64>::new();
    let mut state_counts = [0; 6];
    let mut owner_counts = [0; 3];
    let mut owner_failures = 0;
    let mut crossover_ranges = [f64::INFINITY, 0.0, f64::INFINITY, 0.0];
    for (frame_index, (&source, &output)) in centers.iter().zip(&output_centers).enumerate() {
        let mut spectra: [Vec<Vec<Complex64>>; 3] = std::array::from_fn(|layer| {
            analyze_frame(channels, source, GEOMETRY[layer], &mut planner)
        });
        let classifier = linked_magnitudes(&spectra[1]);
        let guidance = guidance(&classifier, &previous_classifier, frame_index > 0);
        previous_classifier = classifier;
        crossover_ranges[0] = crossover_ranges[0].min(guidance.low_hz);
        crossover_ranges[1] = crossover_ranges[1].max(guidance.low_hz);
        crossover_ranges[2] = crossover_ranges[2].min(guidance.high_hz);
        crossover_ranges[3] = crossover_ranges[3].max(guidance.high_hz);
        for classifier_bin in 0..=GEOMETRY[1] / 2 {
            let frequency = classifier_bin as f64 * SAMPLE_RATE / GEOMETRY[1] as f64;
            let owners = usize::from(frequency < guidance.low_hz)
                + usize::from(frequency >= guidance.low_hz && frequency < guidance.high_hz)
                + usize::from(frequency >= guidance.high_hz);
            owner_failures += usize::from(owners != 1);
        }
        for layer in 0..3 {
            transport_partitioned(
                &mut spectra[layer],
                &mut states,
                layer,
                source,
                output,
                ratio,
                &guidance,
                &mut state_counts,
            );
            let length = GEOMETRY[layer];
            let window = window(length);
            let inverse = planner.plan_fft_inverse(length);
            for spectrum in &mut spectra[layer] {
                apply_frequency_owner(spectrum, layer, guidance.low_hz, guidance.high_hz);
                owner_counts[layer] += spectrum
                    .iter()
                    .take(length / 2 + 1)
                    .filter(|value| value.norm_sqr() > 0.0)
                    .count();
                inverse.process(spectrum);
            }
            for (offset, weight) in window.iter().copied().enumerate() {
                let logical = output - length as isize / 2 + offset as isize;
                let domain = (logical - output_start) as usize;
                operators[layer][domain] += weight * weight;
                for (channel, spectrum) in spectra[layer].iter().enumerate() {
                    outputs[layer][channel][domain] += spectrum[offset] * (weight / length as f64);
                }
            }
        }
    }
    finish(
        outputs,
        operators,
        output_start,
        target_len,
        owner_failures,
        state_counts,
        owner_counts,
        crossover_ranges,
    )
}

fn render_predictor(channels: &[Vec<f64>], ratio: f64, schedule: &Schedule) -> Render {
    let length = GEOMETRY[1];
    let target_len = (ratio * channels[0].len() as f64).round() as usize;
    let centers = centers(channels[0].len(), length);
    let output_centers = centers
        .iter()
        .map(|center| project(*center, channels[0].len(), ratio, schedule))
        .collect::<Vec<_>>();
    let output_start = output_centers[0] - length as isize / 2;
    let output_end = output_centers[output_centers.len() - 1] + length as isize / 2;
    let domain_len = (output_end - output_start) as usize;
    let mut output = vec![vec![Complex64::new(0.0, 0.0); domain_len]; channels.len()];
    let mut operator = vec![0.0; domain_len];
    let mut states = (0..channels.len())
        .map(|_| PhaseState::new(length))
        .collect::<Vec<_>>();
    let mut planner = FftPlanner::<f64>::new();
    let window = window(length);
    let mut state_counts = [0; 6];
    for (&source, &output_center) in centers.iter().zip(&output_centers) {
        let mut spectra = analyze_frame(channels, source, length, &mut planner);
        transport_predictor(
            &mut spectra,
            &mut states,
            source,
            output_center,
            &mut state_counts,
        );
        let inverse = planner.plan_fft_inverse(length);
        for spectrum in &mut spectra {
            mirror(spectrum);
            inverse.process(spectrum);
        }
        for (offset, weight) in window.iter().copied().enumerate() {
            let logical = output_center - length as isize / 2 + offset as isize;
            let domain = (logical - output_start) as usize;
            operator[domain] += weight * weight;
            for (channel, spectrum) in spectra.iter().enumerate() {
                output[channel][domain] += spectrum[offset] * (weight / length as f64);
            }
        }
    }
    let outputs = [
        output,
        vec![vec![Complex64::new(0.0, 0.0); domain_len]; channels.len()],
        vec![vec![Complex64::new(0.0, 0.0); domain_len]; channels.len()],
    ];
    let operators = [operator, vec![0.0; domain_len], vec![0.0; domain_len]];
    finish(
        outputs,
        operators,
        output_start,
        target_len,
        0,
        state_counts,
        [length / 2 + 1, 0, 0],
        [0.0; 4],
    )
}

fn transport_partitioned(
    spectra: &mut [Vec<Complex64>],
    states: &mut [[PhaseState; 3]],
    layer: usize,
    source: isize,
    output: isize,
    ratio: f64,
    guidance: &Guidance,
    counts: &mut [usize; 6],
) {
    let length = GEOMETRY[layer];
    for (channel, spectrum) in spectra.iter_mut().enumerate() {
        let state = &mut states[channel][layer];
        let first = state.source.is_none();
        let source_hop = state.source.map(|value| source - value).unwrap_or(0) as f64;
        let output_hop = state.output.map(|value| output - value).unwrap_or(0) as f64;
        let peaks = spectral_peaks(spectrum);
        let analysis = spectrum
            .iter()
            .take(length / 2 + 1)
            .map(|value| value.arg())
            .collect::<Vec<_>>();
        let mut ordinary = vec![0.0; length / 2 + 1];
        for bin in 0..=length / 2 {
            ordinary[bin] = if first {
                analysis[bin]
            } else {
                let expected = std::f64::consts::TAU * bin as f64 * source_hop / length as f64;
                let residual = wrap(analysis[bin] - state.analysis[bin] - expected);
                state.synthesis[bin] + (expected + residual) / source_hop * output_hop
            };
            counts[0] += 1;
        }
        for bin in 0..=length / 2 {
            let frequency = bin as f64 * SAMPLE_RATE / length as f64;
            let unlocked = ratio > 2.0 && frequency >= guidance.high_hz;
            let reset = guidance.attack && frequency < guidance.high_hz;
            state.synthesis[bin] = if reset {
                counts[2] += 1;
                counts[4] += 1;
                analysis[bin]
            } else if unlocked || first {
                counts[3] += usize::from(unlocked);
                ordinary[bin]
            } else {
                let peak = nearest_peak(bin, &peaks);
                counts[1] += 1;
                ordinary[peak] + wrap(analysis[bin] - analysis[peak])
            };
            state.analysis[bin] = analysis[bin];
            spectrum[bin] = Complex64::from_polar(spectrum[bin].norm(), state.synthesis[bin]);
        }
        spectrum[0].im = 0.0;
        spectrum[length / 2].im = 0.0;
        state.source = Some(source);
        state.output = Some(output);
    }
    if spectra.len() > 1 {
        for bin in 0..=length / 2 {
            let owner = (0..spectra.len())
                .max_by(|left, right| {
                    spectra[*left][bin]
                        .norm_sqr()
                        .total_cmp(&spectra[*right][bin].norm_sqr())
                })
                .unwrap_or(0);
            let owner_phase = spectra[owner][bin].arg();
            let owner_analysis = states[owner][layer].analysis[bin];
            for channel in 0..spectra.len() {
                if channel != owner {
                    let phase =
                        owner_phase + wrap(states[channel][layer].analysis[bin] - owner_analysis);
                    spectra[channel][bin] =
                        Complex64::from_polar(spectra[channel][bin].norm(), phase);
                    states[channel][layer].synthesis[bin] = phase;
                    counts[5] += 1;
                }
            }
        }
    }
    for spectrum in spectra {
        mirror(spectrum);
    }
}

fn transport_predictor(
    spectra: &mut [Vec<Complex64>],
    states: &mut [PhaseState],
    source: isize,
    output: isize,
    counts: &mut [usize; 6],
) {
    let length = GEOMETRY[1];
    for (channel, spectrum) in spectra.iter_mut().enumerate() {
        let state = &mut states[channel];
        let first = state.source.is_none();
        let source_hop = state.source.map(|value| source - value).unwrap_or(0) as f64;
        let output_hop = state.output.map(|value| output - value).unwrap_or(0) as f64;
        let analysis = spectrum
            .iter()
            .take(length / 2 + 1)
            .map(|value| value.arg())
            .collect::<Vec<_>>();
        let magnitudes = spectrum
            .iter()
            .take(length / 2 + 1)
            .map(|value| value.norm())
            .collect::<Vec<_>>();
        let mut ordinary = vec![0.0; length / 2 + 1];
        for bin in 0..=length / 2 {
            ordinary[bin] = if first {
                analysis[bin]
            } else {
                let expected = std::f64::consts::TAU * bin as f64 * source_hop / length as f64;
                let residual = wrap(analysis[bin] - state.analysis[bin] - expected);
                state.synthesis[bin] + (expected + residual) / source_hop * output_hop
            };
            counts[0] += 1;
        }
        for bin in 0..=length / 2 {
            let mut prediction = Complex64::from_polar(magnitudes[bin].max(1.0e-12), ordinary[bin]);
            for distance in [1_usize, 4] {
                for neighbour in [bin.checked_sub(distance), bin.checked_add(distance)] {
                    let Some(neighbour) = neighbour.filter(|value| *value <= length / 2) else {
                        continue;
                    };
                    let weight = (magnitudes[bin] * magnitudes[neighbour]).sqrt();
                    let phase = ordinary[neighbour] + wrap(analysis[bin] - analysis[neighbour]);
                    prediction += Complex64::from_polar(weight, phase);
                }
            }
            let phase = if prediction.norm_sqr() > 0.0 {
                prediction.arg()
            } else {
                ordinary[bin]
            };
            state.analysis[bin] = analysis[bin];
            state.synthesis[bin] = phase;
            spectrum[bin] = Complex64::from_polar(magnitudes[bin], phase);
            counts[1] += usize::from(!first);
        }
        spectrum[0].im = 0.0;
        spectrum[length / 2].im = 0.0;
        state.source = Some(source);
        state.output = Some(output);
    }
}

fn finish(
    outputs: [Vec<Vec<Complex64>>; 3],
    operators: [Vec<f64>; 3],
    output_start: isize,
    target_len: usize,
    owner_failures: usize,
    state_counts: [usize; 6],
    owner_counts: [usize; 3],
    crossover_ranges_hz: [f64; 4],
) -> Render {
    let crop = (-output_start) as usize;
    let channel_count = outputs[0].len();
    let mut samples = vec![vec![0.0; target_len]; channel_count];
    let mut uncovered = 0;
    for index in 0..target_len {
        let domain = crop + index;
        let mut covered = false;
        for layer in 0..3 {
            let denominator = operators[layer][domain];
            if denominator > 0.0 {
                covered = true;
                for channel in 0..channel_count {
                    samples[channel][index] += outputs[layer][channel][domain].re / denominator;
                }
            }
        }
        uncovered += usize::from(!covered);
    }
    let non_finite = samples
        .iter()
        .flatten()
        .filter(|sample| !sample.is_finite())
        .count();
    let boundary_failures = usize::from(samples.iter().any(|channel| {
        channel.first().is_none_or(|sample| !sample.is_finite())
            || channel.last().is_none_or(|sample| !sample.is_finite())
    }));
    let mut hash = HASH_OFFSET;
    for channel in &samples {
        hash_samples(&mut hash, channel);
    }
    Render {
        samples,
        target_len,
        uncovered,
        non_finite,
        boundary_failures,
        owner_failures,
        state_counts,
        owner_counts,
        crossover_ranges_hz,
        hash,
    }
}

fn analyze_frame(
    channels: &[Vec<f64>],
    center: isize,
    length: usize,
    planner: &mut FftPlanner<f64>,
) -> Vec<Vec<Complex64>> {
    let window = window(length);
    let forward = planner.plan_fft_forward(length);
    channels
        .iter()
        .map(|channel| {
            let mut spectrum = (0..length)
                .map(|offset| {
                    let source = center - length as isize / 2 + offset as isize;
                    Complex64::new(reflected(channel, source) * window[offset], 0.0)
                })
                .collect::<Vec<_>>();
            forward.process(&mut spectrum);
            spectrum
        })
        .collect()
}

fn linked_magnitudes(spectra: &[Vec<Complex64>]) -> Vec<f64> {
    (0..=spectra[0].len() / 2)
        .map(|bin| {
            spectra
                .iter()
                .map(|spectrum| spectrum[bin].norm_sqr())
                .sum::<f64>()
                .sqrt()
        })
        .collect()
}

fn guidance(current: &[f64], previous: &[f64], has_previous: bool) -> Guidance {
    let bins = current.len() - 1;
    let low_nominal = bins / 32;
    let high_nominal = bins / 4;
    let low_bin = valley(current, low_nominal, (bins / 64).max(1));
    let high_bin = valley(current, high_nominal, (bins / 32).max(1));
    let positive_flux = current
        .iter()
        .zip(previous)
        .map(|(now, before)| (now - before).max(0.0))
        .sum::<f64>();
    let energy = current.iter().sum::<f64>().max(1.0e-12);
    let attack = has_previous && positive_flux * 8.0 > energy;
    Guidance {
        low_hz: low_bin as f64 * SAMPLE_RATE / GEOMETRY[1] as f64,
        high_hz: high_bin as f64 * SAMPLE_RATE / GEOMETRY[1] as f64,
        attack,
    }
}

fn valley(magnitudes: &[f64], nominal: usize, radius: usize) -> usize {
    let start = nominal.saturating_sub(radius).max(1);
    let end = (nominal + radius).min(magnitudes.len() - 2);
    (start..=end)
        .min_by(|left, right| magnitudes[*left].total_cmp(&magnitudes[*right]))
        .unwrap_or(nominal)
}

fn apply_frequency_owner(spectrum: &mut [Complex64], layer: usize, low: f64, high: f64) {
    let length = spectrum.len();
    for bin in 0..=length / 2 {
        let frequency = bin as f64 * SAMPLE_RATE / length as f64;
        let owned = match layer {
            0 => frequency >= high,
            1 => frequency >= low && frequency < high,
            2 => frequency < low,
            _ => false,
        };
        if !owned {
            spectrum[bin] = Complex64::new(0.0, 0.0);
        }
    }
    mirror(spectrum);
}

fn spectral_peaks(spectrum: &[Complex64]) -> Vec<usize> {
    let end = spectrum.len() / 2;
    let mut peaks = (1..end)
        .filter(|bin| {
            spectrum[*bin].norm_sqr() >= spectrum[*bin - 1].norm_sqr()
                && spectrum[*bin].norm_sqr() > spectrum[*bin + 1].norm_sqr()
        })
        .collect::<Vec<_>>();
    if peaks.is_empty() {
        peaks.push(
            (0..=end)
                .max_by(|left, right| {
                    spectrum[*left]
                        .norm_sqr()
                        .total_cmp(&spectrum[*right].norm_sqr())
                })
                .unwrap_or(0),
        );
    }
    peaks
}

fn nearest_peak(bin: usize, peaks: &[usize]) -> usize {
    *peaks
        .iter()
        .min_by_key(|peak| peak.abs_diff(bin))
        .expect("at least one spectral peak")
}

fn centers(source_len: usize, longest: usize) -> Vec<isize> {
    let mut result = Vec::new();
    let mut center = -(longest as isize / 2);
    while center < source_len as isize + longest as isize / 2 {
        result.push(center);
        center += BASE_HOP as isize;
    }
    result
}

fn project(source: isize, source_len: usize, ratio: f64, schedule: &Schedule) -> isize {
    if source < 0 || source > source_len as isize {
        (source as f64 * ratio).round() as isize
    } else {
        schedule.positions[source as usize / BASE_HOP] as isize
    }
}

fn window(length: usize) -> Vec<f64> {
    (0..length)
        .map(|index| {
            (0.5 - 0.5 * (std::f64::consts::TAU * index as f64 / length as f64).cos()).sqrt()
        })
        .collect()
}

fn reflected(input: &[f64], mut index: isize) -> f64 {
    let end = input.len() as isize - 1;
    while index < 0 || index > end {
        index = if index < 0 {
            -index - 1
        } else {
            2 * end - index + 1
        };
    }
    input[index as usize]
}

fn mirror(spectrum: &mut [Complex64]) {
    let length = spectrum.len();
    for bin in 1..length / 2 {
        spectrum[length - bin] = spectrum[bin].conj();
    }
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

fn mean_quality(values: &[[f64; 5]]) -> [f64; 5] {
    if values.is_empty() {
        return [f64::NAN; 5];
    }
    std::array::from_fn(|index| {
        values.iter().map(|value| value[index]).sum::<f64>() / values.len() as f64
    })
}

fn hash_samples(state: &mut u64, samples: &[f64]) {
    for sample in samples {
        mix(state, sample.to_bits());
    }
}

fn hash_bytes(state: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        mix(state, u64::from(*byte));
    }
}

fn mix(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
