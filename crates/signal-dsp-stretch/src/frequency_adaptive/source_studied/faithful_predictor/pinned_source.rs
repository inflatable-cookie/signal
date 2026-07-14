use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use super::{
    chord_control, chord_spectrum_metrics, hash_samples, render_stage, spectrum_metrics,
    TraceStage, CHORD_FREQUENCIES, SAMPLE_RATE,
};

const PINNED_REVISION: &str = "57b93f4e9206a089a45387eaa39bdc9f310d3308";
const PINNED_VERSION: &str = "1.3.2";
const INPUT_FRAMES: usize = SAMPLE_RATE * 2;
const OUTPUT_FRAMES: usize = INPUT_FRAMES * 2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct PinnedToneEvidence {
    pub(in crate::frequency_adaptive) frequency_hz: f64,
    pub(in crate::frequency_adaptive) input_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) output_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) output_peak_error_hz: f64,
    pub(in crate::frequency_adaptive) strongest_sideband_offset_hz: f64,
    pub(in crate::frequency_adaptive) signal_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) signal_minus_pinned_db: f64,
    pub(in crate::frequency_adaptive) output_hash: u64,
    pub(in crate::frequency_adaptive) signal_hash: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum PinnedSourceDirection {
    SignalTranslationDivergence,
    StudiedTopologyGateMismatch,
    StudiedTopologyGateMismatchAndSignalDivergence,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct PinnedSourceReview {
    pub(in crate::frequency_adaptive) revision: String,
    pub(in crate::frequency_adaptive) version: String,
    pub(in crate::frequency_adaptive) geometry: [usize; 3],
    pub(in crate::frequency_adaptive) tones: [PinnedToneEvidence; 4],
    pub(in crate::frequency_adaptive) chord_input_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) chord_output_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) chord_peak_error_hz: f64,
    pub(in crate::frequency_adaptive) chord_sideband_offset_hz: f64,
    pub(in crate::frequency_adaptive) chord_signal_out_of_band_db: f64,
    pub(in crate::frequency_adaptive) chord_signal_minus_pinned_db: f64,
    pub(in crate::frequency_adaptive) structural_failures: [usize; 4],
    pub(in crate::frequency_adaptive) output_hashes: [u64; 5],
    pub(in crate::frequency_adaptive) signal_hashes: [u64; 5],
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: PinnedSourceDirection,
}

pub(in crate::frequency_adaptive) fn review() -> PinnedSourceReview {
    let binary = required_path("SIGNALSMITH_STRETCH_BIN");
    let source = required_path("SIGNALSMITH_STRETCH_SOURCE");
    let canonical_binary = binary
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve {}: {error}", binary.display()));
    let canonical_source = source
        .canonicalize()
        .unwrap_or_else(|error| panic!("resolve {}: {error}", source.display()));
    assert!(
        canonical_binary.starts_with(&canonical_source),
        "pinned CLI must be built inside the verified source checkout"
    );
    let revision = command_text("git", &["-C", path_text(&source), "rev-parse", "HEAD"]);
    assert_eq!(revision, PINNED_REVISION);
    let version = command_text(path_text(&binary), &["-v"]);
    assert_eq!(version, PINNED_VERSION);

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-pinned-source-synthetic");
    replace_directory(&root);
    let first = run(&binary, &root.join("first"), &revision, &version);
    let second = run(&binary, &root.join("second"), &revision, &version);
    PinnedSourceReview {
        repeated: first == second,
        ..first
    }
}

fn run(binary: &Path, root: &Path, revision: &str, version: &str) -> PinnedSourceReview {
    fs::create_dir_all(root).unwrap_or_else(|error| panic!("create {}: {error}", root.display()));
    let mut structural_failures = [0; 4];
    let mut output_hashes = [0; 5];
    let mut signal_hashes = [0; 5];
    let tones = CHORD_FREQUENCIES.map(|frequency| {
        let tone_index = CHORD_FREQUENCIES
            .iter()
            .position(|candidate| *candidate == frequency)
            .expect("frozen chord frequency");
        let amplitude = 0.16 - tone_index as f64 * 0.015;
        let input = (0..INPUT_FRAMES)
            .map(|index| {
                amplitude
                    * (std::f64::consts::TAU * frequency * index as f64 / SAMPLE_RATE as f64).sin()
            })
            .collect::<Vec<_>>();
        let stem = format!("tone-{tone_index}");
        let (quantized_input, output) = render(binary, root, &stem, &input);
        structural_failures[0] += usize::from(output.len() != OUTPUT_FRAMES);
        structural_failures[1] += output.iter().filter(|sample| !sample.is_finite()).count();
        let signal = render_stage(&quantized_input, 2.0, SAMPLE_RATE, TraceStage::Complete);
        structural_failures[2] += usize::from(signal.samples.len() != OUTPUT_FRAMES);
        structural_failures[3] += signal
            .samples
            .iter()
            .filter(|sample| !sample.is_finite())
            .count();
        let input_metrics = spectrum_metrics(&quantized_input, std::slice::from_ref(&frequency));
        let output_metrics = spectrum_metrics(
            &output[SAMPLE_RATE..SAMPLE_RATE * 3],
            std::slice::from_ref(&frequency),
        );
        let signal_metrics = spectrum_metrics(
            &signal.samples[SAMPLE_RATE..SAMPLE_RATE * 3],
            std::slice::from_ref(&frequency),
        );
        let output_hash = hash_samples(&output);
        let signal_hash = hash_samples(&signal.samples);
        output_hashes[tone_index] = output_hash;
        signal_hashes[tone_index] = signal_hash;
        PinnedToneEvidence {
            frequency_hz: frequency,
            input_out_of_band_db: input_metrics.out_of_band_db,
            output_out_of_band_db: output_metrics.out_of_band_db,
            output_peak_error_hz: output_metrics.maximum_peak_error_hz,
            strongest_sideband_offset_hz: output_metrics.strongest_sideband_offset_hz,
            signal_out_of_band_db: signal_metrics.out_of_band_db,
            signal_minus_pinned_db: signal_metrics.out_of_band_db - output_metrics.out_of_band_db,
            output_hash,
            signal_hash,
        }
    });
    let (quantized_chord, chord_output) = render(binary, root, "chord", &chord_control());
    structural_failures[0] += usize::from(chord_output.len() != OUTPUT_FRAMES);
    structural_failures[1] += chord_output
        .iter()
        .filter(|sample| !sample.is_finite())
        .count();
    let signal_chord = render_stage(&quantized_chord, 2.0, SAMPLE_RATE, TraceStage::Complete);
    structural_failures[2] += usize::from(signal_chord.samples.len() != OUTPUT_FRAMES);
    structural_failures[3] += signal_chord
        .samples
        .iter()
        .filter(|sample| !sample.is_finite())
        .count();
    let chord_input_metrics = chord_spectrum_metrics(&quantized_chord);
    let chord_output_metrics = chord_spectrum_metrics(&chord_output[SAMPLE_RATE..SAMPLE_RATE * 3]);
    let chord_signal_metrics =
        chord_spectrum_metrics(&signal_chord.samples[SAMPLE_RATE..SAMPLE_RATE * 3]);
    output_hashes[4] = hash_samples(&chord_output);
    signal_hashes[4] = hash_samples(&signal_chord.samples);
    let gate_passed = tones.iter().all(|tone| tone.output_out_of_band_db <= -60.0)
        && chord_output_metrics.out_of_band_db <= -60.0;
    let signal_diverged = tones.iter().any(|tone| tone.signal_minus_pinned_db > 1.0)
        || chord_signal_metrics.out_of_band_db - chord_output_metrics.out_of_band_db > 1.0;
    PinnedSourceReview {
        revision: revision.to_string(),
        version: version.to_string(),
        geometry: [240, 960, 4],
        tones,
        chord_input_out_of_band_db: chord_input_metrics.out_of_band_db,
        chord_output_out_of_band_db: chord_output_metrics.out_of_band_db,
        chord_peak_error_hz: chord_output_metrics.maximum_peak_error_hz,
        chord_sideband_offset_hz: chord_output_metrics.strongest_sideband_offset_hz,
        chord_signal_out_of_band_db: chord_signal_metrics.out_of_band_db,
        chord_signal_minus_pinned_db: chord_signal_metrics.out_of_band_db
            - chord_output_metrics.out_of_band_db,
        structural_failures,
        output_hashes,
        signal_hashes,
        repeated: false,
        direction: if gate_passed {
            PinnedSourceDirection::SignalTranslationDivergence
        } else if signal_diverged {
            PinnedSourceDirection::StudiedTopologyGateMismatchAndSignalDivergence
        } else {
            PinnedSourceDirection::StudiedTopologyGateMismatch
        },
    }
}

fn render(binary: &Path, root: &Path, stem: &str, input: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let input_path = root.join(format!("{stem}-input.wav"));
    let output_path = root.join(format!("{stem}-output.wav"));
    write_wav(&input_path, input);
    let status = Command::new(binary)
        .args([
            input_path.as_os_str(),
            output_path.as_os_str(),
            "--time=2".as_ref(),
        ])
        .status()
        .unwrap_or_else(|error| panic!("run {}: {error}", binary.display()));
    assert!(status.success(), "{} failed", binary.display());
    (
        read_wav(&input_path, INPUT_FRAMES),
        read_wav(&output_path, OUTPUT_FRAMES),
    )
}

fn write_wav(path: &Path, samples: &[f64]) {
    let specification = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, specification)
        .unwrap_or_else(|error| panic!("create {}: {error}", path.display()));
    for sample in samples {
        let quantized =
            (sample.clamp(-1.0, f64::from(i16::MAX) / 32_768.0) * 32_768.0).round() as i16;
        writer.write_sample(quantized).expect("write pinned input");
    }
    writer.finalize().expect("finalize pinned input");
}

fn read_wav(path: &Path, expected_frames: usize) -> Vec<f64> {
    let mut reader = hound::WavReader::open(path)
        .unwrap_or_else(|error| panic!("open {}: {error}", path.display()));
    let specification = reader.spec();
    assert_eq!(specification.channels, 1, "{}", path.display());
    assert_eq!(
        specification.sample_rate,
        SAMPLE_RATE as u32,
        "{}",
        path.display()
    );
    assert_eq!(
        reader.duration() as usize,
        expected_frames,
        "{}",
        path.display()
    );
    reader
        .samples::<i16>()
        .map(|sample| f64::from(sample.expect("read pinned sample")) / 32_768.0)
        .collect()
}

fn required_path(variable: &str) -> PathBuf {
    env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("set {variable} for the pinned source comparator"))
}

fn command_text(program: &str, arguments: &[&str]) -> String {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("run {program}: {error}"));
    assert!(
        output.status.success(),
        "{program} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("command output is UTF-8")
        .trim()
        .to_string()
}

fn path_text(path: &Path) -> &str {
    path.to_str().expect("pinned source path is UTF-8")
}

fn replace_directory(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path)
            .unwrap_or_else(|error| panic!("replace {}: {error}", path.display()));
    }
    fs::create_dir_all(path).unwrap_or_else(|error| panic!("create {}: {error}", path.display()));
}
