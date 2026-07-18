use std::{env, ffi::OsString, path::PathBuf, process::Command};

use super::super::external::{command_text, file_hash};

const VERSION: &str = "4.0.0";

pub(super) struct RubberBand {
    pub(super) binary: PathBuf,
    pub(super) version: String,
    pub(super) binary_hash: u64,
}

pub(super) struct Render {
    pub(super) input: Vec<Vec<f64>>,
    pub(super) channels: Vec<Vec<f64>>,
    pub(super) input_hash: u64,
    pub(super) output_hash: u64,
    pub(super) command_hash: u64,
}

impl RubberBand {
    pub(super) fn discover() -> Self {
        let binary = env::var_os("RUBBERBAND_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| "rubberband".into());
        let binary = if binary.components().count() == 1 {
            PathBuf::from(command_text("which", &[binary.into_os_string()]))
        } else {
            binary
        };
        let binary = binary.canonicalize().expect("resolve Rubber Band binary");
        let version = command_text(&binary, &["--version".into()]);
        assert_eq!(version, VERSION);
        let binary_hash = file_hash(&binary);
        Self {
            binary,
            version,
            binary_hash,
        }
    }

    pub(super) fn render(
        &self,
        root: &std::path::Path,
        stem: &str,
        channels: &[&[f64]],
        ratio: f64,
        sample_rate: usize,
    ) -> Render {
        assert!(matches!(channels.len(), 1 | 2));
        let frames = channels[0].len();
        assert!(channels.iter().all(|channel| channel.len() == frames));
        let input_path = root.join(format!("{stem}-input.wav"));
        let output_path = root.join(format!("{stem}-output.wav"));
        write_wav(&input_path, channels, sample_rate as u32);
        let args: [OsString; 6] = [
            "-q".into(),
            "-3".into(),
            "-t".into(),
            format!("{ratio:.9}").into(),
            input_path.as_os_str().into(),
            output_path.as_os_str().into(),
        ];
        let status = Command::new(&self.binary)
            .args(&args)
            .status()
            .expect("run Rubber Band comparator");
        assert!(status.success());
        let target = (frames as f64 * ratio).round() as usize;
        let input = read_wav(&input_path, frames, channels.len(), sample_rate as u32);
        let output = read_wav(&output_path, target, channels.len(), sample_rate as u32);
        Render {
            input,
            channels: output,
            input_hash: file_hash(&input_path),
            output_hash: file_hash(&output_path),
            command_hash: command_hash(
                self.binary_hash,
                ratio,
                frames,
                channels.len(),
                sample_rate,
            ),
        }
    }
}

fn write_wav(path: &std::path::Path, channels: &[&[f64]], sample_rate: u32) {
    let spec = hound::WavSpec {
        channels: channels.len() as u16,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).expect("create comparator input");
    for frame in 0..channels[0].len() {
        for channel in channels {
            writer
                .write_sample((channel[frame].clamp(-1.0, 1.0) * i16::MAX as f64).round() as i16)
                .expect("write comparator input sample");
        }
    }
    writer.finalize().expect("finalize comparator input");
}

fn read_wav(
    path: &std::path::Path,
    frames: usize,
    channels: usize,
    sample_rate: u32,
) -> Vec<Vec<f64>> {
    let mut reader = hound::WavReader::open(path)
        .unwrap_or_else(|error| panic!("open {}: {error}", path.display()));
    let spec = reader.spec();
    assert_eq!(spec.channels as usize, channels);
    assert_eq!(spec.sample_rate, sample_rate);
    let scale = 2_f64.powi(spec.bits_per_sample as i32 - 1);
    let samples = reader
        .samples::<i32>()
        .map(|sample| sample.expect("read comparator sample") as f64 / scale)
        .collect::<Vec<_>>();
    assert_eq!(samples.len(), frames * channels);
    (0..channels)
        .map(|channel| {
            samples
                .iter()
                .skip(channel)
                .step_by(channels)
                .copied()
                .collect()
        })
        .collect()
}

fn command_hash(
    binary_hash: u64,
    ratio: f64,
    frames: usize,
    channels: usize,
    sample_rate: usize,
) -> u64 {
    let contract = format!(
        "rubberband:{binary_hash:016x} -q -3 -t {ratio:.9} frames={frames} channels={channels} sample_rate={sample_rate}"
    );
    contract.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ byte as u64).wrapping_mul(0x100_0000_01b3)
    })
}
