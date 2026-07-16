use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const PINNED_REVISION: &str = "57b93f4e9206a089a45387eaa39bdc9f310d3308";

pub(super) struct ExternalEngines {
    signalsmith: PathBuf,
    rubber_band: PathBuf,
    pub(in super::super) signalsmith_revision: String,
    pub(in super::super) signalsmith_version: String,
    pub(in super::super) rubber_band_version: String,
}

pub(super) struct RenderFiles {
    pub(super) input: [Vec<f64>; 2],
    pub(super) signalsmith: [Vec<f64>; 2],
    pub(super) rubber_band: [Vec<f64>; 2],
    pub(super) input_hash: u64,
    pub(super) signalsmith_hash: u64,
    pub(super) rubber_band_hash: u64,
}

impl ExternalEngines {
    pub(super) fn discover() -> Self {
        let signalsmith = required_path("SIGNALSMITH_STRETCH_BIN");
        let source = required_path("SIGNALSMITH_STRETCH_SOURCE");
        let rubber_band = env::var_os("RUBBERBAND_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| "rubberband".into());
        let canonical_binary = signalsmith
            .canonicalize()
            .expect("resolve Signalsmith binary");
        let canonical_source = source.canonicalize().expect("resolve Signalsmith source");
        assert!(
            canonical_binary.starts_with(canonical_source),
            "Signalsmith binary must be inside pinned source checkout"
        );
        let signalsmith_revision = command_text(
            "git",
            &[
                "-C".into(),
                source.into_os_string(),
                "rev-parse".into(),
                "HEAD".into(),
            ],
        );
        let signalsmith_version = command_text(&signalsmith, &["-v".into()]);
        let rubber_band_version = command_text(&rubber_band, &["--version".into()]);
        assert_eq!(signalsmith_revision, PINNED_REVISION);
        assert_eq!(signalsmith_version, "1.3.2");
        assert_eq!(rubber_band_version, "4.0.0");
        Self {
            signalsmith,
            rubber_band,
            signalsmith_revision,
            signalsmith_version,
            rubber_band_version,
        }
    }

    pub(super) fn render(
        &self,
        root: &Path,
        stem: &str,
        source: &[Vec<f64>; 2],
        ratio: f64,
        sample_rate: usize,
    ) -> RenderFiles {
        let input_path = root.join(format!("{stem}-input.wav"));
        let signalsmith_path = root.join(format!("{stem}-signalsmith.wav"));
        let rubber_path = root.join(format!("{stem}-rubber-band.wav"));
        write_stereo(&input_path, source, sample_rate as u32);
        let target = (source[0].len() as f64 * ratio).round() as usize;
        run(
            &self.signalsmith,
            &[
                input_path.as_os_str().into(),
                signalsmith_path.as_os_str().into(),
                format!("--time={ratio:.9}").into(),
            ],
        );
        run(
            &self.rubber_band,
            &[
                "-q".into(),
                "-3".into(),
                "-t".into(),
                format!("{ratio:.9}").into(),
                input_path.as_os_str().into(),
                rubber_path.as_os_str().into(),
            ],
        );
        RenderFiles {
            input: read_stereo(&input_path, source[0].len(), sample_rate as u32),
            signalsmith: read_stereo(&signalsmith_path, target, sample_rate as u32),
            rubber_band: read_stereo(&rubber_path, target, sample_rate as u32),
            input_hash: file_hash(&input_path),
            signalsmith_hash: file_hash(&signalsmith_path),
            rubber_band_hash: file_hash(&rubber_path),
        }
    }
}

pub(super) fn replace_directory(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path)
            .unwrap_or_else(|error| panic!("remove {}: {error}", path.display()));
    }
    fs::create_dir_all(path).unwrap_or_else(|error| panic!("create {}: {error}", path.display()));
}

pub(super) fn write_stereo(path: &Path, channels: &[Vec<f64>; 2], sample_rate: u32) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).expect("create stereo calibration WAV");
    for index in 0..channels[0].len() {
        for channel in channels {
            writer
                .write_sample((channel[index].clamp(-1.0, 1.0) * i16::MAX as f64).round() as i16)
                .expect("write stereo sample");
        }
    }
    writer.finalize().expect("finalize stereo calibration WAV");
}

pub(super) fn read_stereo(path: &Path, frames: usize, sample_rate: u32) -> [Vec<f64>; 2] {
    let mut reader = hound::WavReader::open(path)
        .unwrap_or_else(|error| panic!("open {}: {error}", path.display()));
    let spec = reader.spec();
    assert_eq!([spec.channels as u32, spec.sample_rate], [2, sample_rate]);
    let scale = 2_f64.powi(spec.bits_per_sample as i32 - 1);
    let samples = reader
        .samples::<i32>()
        .map(|sample| sample.expect("read stereo sample") as f64 / scale)
        .collect::<Vec<_>>();
    assert_eq!(
        samples.len(),
        frames * 2,
        "exact stereo output length for {}",
        path.display()
    );
    [
        samples.iter().step_by(2).copied().collect(),
        samples.iter().skip(1).step_by(2).copied().collect(),
    ]
}

fn required_path(name: &str) -> PathBuf {
    env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("set {name}"))
}

pub(super) fn run(program: &Path, args: &[OsString]) {
    let status = Command::new(program)
        .args(args)
        .status()
        .unwrap_or_else(|error| panic!("run {}: {error}", program.display()));
    assert!(status.success(), "{} failed", program.display());
}

pub(super) fn command_text(program: impl AsRef<std::ffi::OsStr>, args: &[OsString]) -> String {
    let output = Command::new(program)
        .args(args)
        .output()
        .expect("run version command");
    assert!(output.status.success(), "version command failed");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    String::from_utf8_lossy(bytes).trim().to_string()
}

pub(super) fn file_hash(path: &Path) -> u64 {
    fs::read(path)
        .expect("read calibration file")
        .into_iter()
        .fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ byte as u64).wrapping_mul(0x100000001b3)
        })
}
