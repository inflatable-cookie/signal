use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use rustfft::num_complex::Complex64;

use super::{
    chord_control, hash_samples, render_stage, StageFrameTrace, TraceStage, CHORD_FREQUENCIES,
    SAMPLE_RATE,
};

const PINNED_REVISION: &str = "57b93f4e9206a089a45387eaa39bdc9f310d3308";
const PINNED_LINEAR_REVISION: &str = "5668673560146a9cfe38c25315071e3fd68c8317";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct StageTraceGeometry {
    pub(in crate::frequency_adaptive) block_frames: usize,
    pub(in crate::frequency_adaptive) interval_frames: usize,
    pub(in crate::frequency_adaptive) transform_frames: usize,
    pub(in crate::frequency_adaptive) bands: usize,
    pub(in crate::frequency_adaptive) modified_grid: bool,
    pub(in crate::frequency_adaptive) first_bin_hz: f64,
    pub(in crate::frequency_adaptive) bin_step_hz: f64,
    pub(in crate::frequency_adaptive) source_center: isize,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct StageTraceControlEvidence {
    pub(in crate::frequency_adaptive) name: String,
    pub(in crate::frequency_adaptive) target_count: usize,
    pub(in crate::frequency_adaptive) source_hashes: [u64; 3],
    pub(in crate::frequency_adaptive) signal_hashes: [u64; 3],
    pub(in crate::frequency_adaptive) normalized_magnitude_deltas: [f64; 3],
    pub(in crate::frequency_adaptive) relative_phase_deltas: [f64; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum StageTraceDirection {
    AnalysisTransformGrid,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct StageTraceReview {
    pub(in crate::frequency_adaptive) source_revision: String,
    pub(in crate::frequency_adaptive) linear_revision: String,
    pub(in crate::frequency_adaptive) source_geometry: StageTraceGeometry,
    pub(in crate::frequency_adaptive) signal_geometry: StageTraceGeometry,
    pub(in crate::frequency_adaptive) controls: Vec<StageTraceControlEvidence>,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: StageTraceDirection,
}

#[derive(Clone, Debug, PartialEq)]
struct SourceFrameTrace {
    geometry: StageTraceGeometry,
    current: Vec<Complex64>,
    preliminary: Vec<Complex64>,
    corrected: Vec<Complex64>,
}

pub(in crate::frequency_adaptive) fn review() -> StageTraceReview {
    let source = required_path("SIGNALSMITH_STRETCH_SOURCE");
    let source_revision = command_text("git", &["-C", path_text(&source), "rev-parse", "HEAD"]);
    assert_eq!(source_revision, PINNED_REVISION);
    let linear = source.join("cmd/out/build/_deps/signalsmith-linear-src");
    let linear_revision = command_text("git", &["-C", path_text(&linear), "rev-parse", "HEAD"]);
    assert_eq!(linear_revision, PINNED_LINEAR_REVISION);

    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/stretch-pinned-stage-trace");
    replace_directory(&root);
    let probe = compile_probe(&source, &linear, &root);
    let first = run(&probe, &source_revision, &linear_revision);
    let second = run(&probe, &source_revision, &linear_revision);
    StageTraceReview {
        repeated: first == second,
        ..first
    }
}

fn run(probe: &Path, source_revision: &str, linear_revision: &str) -> StageTraceReview {
    let controls = ["110", "220", "chord"]
        .into_iter()
        .map(|name| compare_control(probe, name))
        .collect::<Vec<_>>();
    let source_geometry = parse_source_trace(probe, "110").geometry;
    StageTraceReview {
        source_revision: source_revision.to_string(),
        linear_revision: linear_revision.to_string(),
        source_geometry,
        signal_geometry: StageTraceGeometry {
            block_frames: 960,
            interval_frames: 240,
            transform_frames: 960,
            bands: 481,
            modified_grid: false,
            first_bin_hz: 0.0,
            bin_step_hz: SAMPLE_RATE as f64 / 960.0,
            source_center: 8_400,
        },
        controls,
        repeated: false,
        direction: StageTraceDirection::AnalysisTransformGrid,
    }
}

fn compare_control(probe: &Path, name: &str) -> StageTraceControlEvidence {
    let source = parse_source_trace(probe, name);
    let signal = signal_trace(name);
    assert_eq!(source.geometry.source_center, signal.source_center);
    let targets = match name {
        "110" => &CHORD_FREQUENCIES[..1],
        "220" => &CHORD_FREQUENCIES[2..3],
        "chord" => &CHORD_FREQUENCIES[..],
        _ => unreachable!("frozen trace control"),
    };
    let source_stages = [&source.current, &source.preliminary, &source.corrected];
    let signal_stages = [&signal.current, &signal.preliminary, &signal.corrected];
    let normalized_magnitude_deltas = std::array::from_fn(|stage| {
        maximum_magnitude_delta(
            source_stages[stage],
            source.geometry,
            signal_stages[stage],
            targets,
        )
    });
    let relative_phase_deltas = std::array::from_fn(|stage| {
        maximum_relative_phase_delta(
            &source.current,
            source_stages[stage + 1],
            source.geometry,
            &signal.current,
            signal_stages[stage + 1],
            targets,
        )
    });
    StageTraceControlEvidence {
        name: name.to_string(),
        target_count: targets.len(),
        source_hashes: source_stages.map(|stage| hash_complex(stage)),
        signal_hashes: signal_stages.map(|stage| hash_complex(stage)),
        normalized_magnitude_deltas,
        relative_phase_deltas,
    }
}

fn signal_trace(name: &str) -> StageFrameTrace {
    let input = match name {
        "110" => tone_control(110.0, 0.16),
        "220" => tone_control(220.0, 0.13),
        "chord" => quantized(chord_control()),
        _ => unreachable!("frozen trace control"),
    };
    render_stage(&input, 2.0, SAMPLE_RATE, TraceStage::Complete)
        .stage_trace
        .expect("frozen interior Signal trace")
}

fn tone_control(frequency: f64, amplitude: f64) -> Vec<f64> {
    quantized(
        (0..SAMPLE_RATE * 2)
            .map(|index| {
                amplitude
                    * (std::f64::consts::TAU * frequency * index as f64 / SAMPLE_RATE as f64).sin()
            })
            .collect(),
    )
}

fn quantized(samples: Vec<f64>) -> Vec<f64> {
    samples
        .into_iter()
        .map(|sample| {
            let value =
                (sample.clamp(-1.0, f64::from(i16::MAX) / 32_768.0) * 32_768.0).round() as i16;
            f64::from(value) / 32_768.0
        })
        .collect()
}

fn maximum_magnitude_delta(
    source: &[Complex64],
    source_geometry: StageTraceGeometry,
    signal: &[Complex64],
    targets: &[f64],
) -> f64 {
    let source_norm = source.iter().map(Complex64::norm_sqr).sum::<f64>().sqrt();
    let signal_norm = signal.iter().map(Complex64::norm_sqr).sum::<f64>().sqrt();
    targets
        .iter()
        .map(|frequency| {
            let source_bin = nearest_bin(*frequency, source_geometry);
            let signal_bin = (*frequency / (SAMPLE_RATE as f64 / 960.0)).round() as usize;
            (source[source_bin].norm() / source_norm - signal[signal_bin].norm() / signal_norm)
                .abs()
        })
        .fold(0.0, f64::max)
}

fn maximum_relative_phase_delta(
    source_current: &[Complex64],
    source_stage: &[Complex64],
    source_geometry: StageTraceGeometry,
    signal_current: &[Complex64],
    signal_stage: &[Complex64],
    targets: &[f64],
) -> f64 {
    targets
        .iter()
        .map(|frequency| {
            let source_bin = nearest_bin(*frequency, source_geometry);
            let signal_bin = (*frequency / (SAMPLE_RATE as f64 / 960.0)).round() as usize;
            let source_phase = (source_stage[source_bin] * source_current[source_bin].conj()).arg();
            let signal_phase = (signal_stage[signal_bin] * signal_current[signal_bin].conj()).arg();
            wrap(source_phase - signal_phase).abs()
        })
        .fold(0.0, f64::max)
}

fn nearest_bin(frequency: f64, geometry: StageTraceGeometry) -> usize {
    ((frequency - geometry.first_bin_hz) / geometry.bin_step_hz)
        .round()
        .clamp(0.0, (geometry.bands - 1) as f64) as usize
}

fn wrap(mut phase: f64) -> f64 {
    while phase > std::f64::consts::PI {
        phase -= std::f64::consts::TAU;
    }
    while phase < -std::f64::consts::PI {
        phase += std::f64::consts::TAU;
    }
    phase
}

fn hash_complex(values: &[Complex64]) -> u64 {
    hash_samples(
        &values
            .iter()
            .flat_map(|value| [value.re, value.im])
            .collect::<Vec<_>>(),
    )
}

fn compile_probe(source: &Path, linear: &Path, root: &Path) -> PathBuf {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/signalsmith_stage_trace.cpp");
    let binary = root.join("signalsmith-stage-trace");
    let compiler = env::var_os("CXX").unwrap_or_else(|| "c++".into());
    let output = Command::new(&compiler)
        .args(["-std=c++17", "-O2"])
        .arg(&fixture)
        .arg(format!("-I{}", source.display()))
        .arg(format!("-I{}/include", linear.display()))
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap_or_else(|error| panic!("run {:?}: {error}", compiler));
    assert!(
        output.status.success(),
        "trace probe compile failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    binary
}

fn parse_source_trace(probe: &Path, name: &str) -> SourceFrameTrace {
    let text = command_text(path_text(probe), &[name]);
    let mut lines = text.lines();
    let meta = lines
        .next()
        .expect("source trace metadata")
        .split('\t')
        .collect::<Vec<_>>();
    assert_eq!(meta.len(), 9);
    assert_eq!(meta[0], "META");
    let geometry = StageTraceGeometry {
        block_frames: parse(meta[1]),
        interval_frames: parse(meta[2]),
        transform_frames: parse(meta[3]),
        bands: parse(meta[4]),
        modified_grid: parse::<usize>(meta[5]) != 0,
        first_bin_hz: parse(meta[6]),
        bin_step_hz: parse(meta[7]),
        source_center: parse(meta[8]),
    };
    let mut current = Vec::with_capacity(geometry.bands);
    let mut preliminary = Vec::with_capacity(geometry.bands);
    let mut corrected = Vec::with_capacity(geometry.bands);
    for (expected_bin, line) in lines.enumerate() {
        let fields = line.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 9);
        assert_eq!(fields[0], "BIN");
        assert_eq!(parse::<usize>(fields[1]), expected_bin);
        current.push(Complex64::new(parse(fields[3]), parse(fields[4])));
        preliminary.push(Complex64::new(parse(fields[5]), parse(fields[6])));
        corrected.push(Complex64::new(parse(fields[7]), parse(fields[8])));
    }
    assert_eq!(current.len(), geometry.bands);
    SourceFrameTrace {
        geometry,
        current,
        preliminary,
        corrected,
    }
}

fn parse<T: std::str::FromStr>(text: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    text.parse().expect("source trace value")
}

fn required_path(variable: &str) -> PathBuf {
    env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("set {variable} for the pinned source trace"))
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
