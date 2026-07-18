use std::{env, ffi::OsString, fs, path::PathBuf, process::Command};

use super::super::external::file_hash;
use super::{ADAPTER_VERSION, PINNED_REVISION};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct ResourceSummary {
    pub(super) renders: u64,
    pub(super) source_frames: u64,
    pub(super) output_frames: u64,
    pub(super) synthesis_frames: u64,
    pub(super) time_groups: u64,
    pub(super) track_visits: u64,
    pub(super) track_births: u64,
    pub(super) track_deaths: u64,
    pub(super) maximum_tracks_per_time: u64,
    pub(super) maximum_track_visits_per_output_read: u64,
    pub(super) maximum_peak_rss_bytes: u64,
    pub(super) elapsed_seconds: f64,
}

impl ResourceSummary {
    pub(super) fn add(&mut self, stats: SpecimenStats) {
        self.renders += 1;
        self.source_frames += stats.source_frames;
        self.output_frames += stats.output_frames;
        self.synthesis_frames += stats.synthesis_frames;
        self.time_groups += stats.time_groups;
        self.track_visits += stats.track_visits;
        self.track_births += stats.track_births;
        self.track_deaths += stats.track_deaths;
        self.maximum_tracks_per_time = self
            .maximum_tracks_per_time
            .max(stats.maximum_tracks_per_time);
        self.maximum_track_visits_per_output_read = self
            .maximum_track_visits_per_output_read
            .max(stats.maximum_track_visits_per_output_read);
        self.maximum_peak_rss_bytes = self.maximum_peak_rss_bytes.max(stats.peak_rss_bytes);
        self.elapsed_seconds += stats.elapsed_seconds;
    }

    pub(super) fn deterministic_fields(self) -> [u64; 10] {
        [
            self.renders,
            self.source_frames,
            self.output_frames,
            self.synthesis_frames,
            self.time_groups,
            self.track_visits,
            self.track_births,
            self.track_deaths,
            self.maximum_tracks_per_time,
            self.maximum_track_visits_per_output_read,
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SpecimenStats {
    source_frames: u64,
    output_frames: u64,
    elapsed_seconds: f64,
    peak_rss_bytes: u64,
    synthesis_frames: u64,
    time_groups: u64,
    track_visits: u64,
    track_births: u64,
    track_deaths: u64,
    maximum_tracks_per_time: u64,
    maximum_track_visits_per_output_read: u64,
}

pub(super) struct SpecimenRender {
    pub(super) channels: Vec<Vec<f64>>,
    pub(super) output_hash: u64,
    pub(super) stats: SpecimenStats,
}

pub(super) struct Specimen {
    binary: PathBuf,
    pub(super) source: PathBuf,
    pub(super) revision: String,
}

impl Specimen {
    pub(super) fn discover() -> Self {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let target = manifest.join("../../target");
        let source = env::var_os("SBSMS_SPECIMEN_SOURCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| target.join("sbsms-2.3.0-source"));
        let binary = env::var_os("SBSMS_SPECIMEN_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| target.join("sbsms-2.3.0-build/sbsms-specimen-adapter"));
        let canonical_target = target.canonicalize().expect("resolve target");
        let canonical_source = source.canonicalize().expect("resolve SBSMS source");
        let canonical_binary = binary.canonicalize().expect("resolve SBSMS adapter");
        assert!(canonical_source.starts_with(&canonical_target));
        assert!(canonical_binary.starts_with(&canonical_target));
        let revision = command_text(
            "git",
            &[
                "-C".into(),
                canonical_source.as_os_str().into(),
                "rev-parse".into(),
                "HEAD".into(),
            ],
        );
        assert_eq!(revision, PINNED_REVISION);
        assert_eq!(
            command_text(&canonical_binary, &["--version".into()]),
            ADAPTER_VERSION
        );
        assert!(command_text(
            "git",
            &[
                "-C".into(),
                canonical_source.as_os_str().into(),
                "status".into(),
                "--short".into(),
            ],
        )
        .is_empty());
        Self {
            binary: canonical_binary,
            source: canonical_source,
            revision,
        }
    }

    pub(super) fn render(
        &self,
        root: &std::path::Path,
        stem: &str,
        channels: &[&[f64]],
        ratio: f64,
    ) -> SpecimenRender {
        assert!(matches!(channels.len(), 1 | 2));
        let frames = channels[0].len();
        assert!(channels.iter().all(|channel| channel.len() == frames));
        let input = root.join(format!("{stem}-input.raw"));
        let output = root.join(format!("{stem}-output.raw"));
        let stats = root.join(format!("{stem}-stats.tsv"));
        write_raw(&input, channels);
        let args = [
            input.as_os_str().to_owned(),
            output.as_os_str().to_owned(),
            frames.to_string().into(),
            channels.len().to_string().into(),
            format!("{ratio:.9}").into(),
            stats.as_os_str().to_owned(),
        ];
        let status = Command::new(&self.binary)
            .args(args)
            .status()
            .expect("run SBSMS specimen adapter");
        assert!(status.success());
        let target = (frames as f64 * ratio).round() as usize;
        SpecimenRender {
            channels: read_raw(&output, target, channels.len()),
            output_hash: file_hash(&output),
            stats: read_stats(&stats),
        }
    }
}

fn write_raw(path: &std::path::Path, channels: &[&[f64]]) {
    let mut bytes = Vec::with_capacity(channels[0].len() * channels.len() * 4);
    for frame in 0..channels[0].len() {
        for channel in channels {
            bytes.extend_from_slice(&(channel[frame] as f32).to_le_bytes());
        }
    }
    fs::write(path, bytes).expect("write specimen raw input");
}

fn read_raw(path: &std::path::Path, frames: usize, channels: usize) -> Vec<Vec<f64>> {
    let bytes = fs::read(path).expect("read specimen raw output");
    assert_eq!(bytes.len(), frames * channels * 4);
    let samples = bytes
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().expect("f32 bytes")) as f64)
        .collect::<Vec<_>>();
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

fn read_stats(path: &std::path::Path) -> SpecimenStats {
    let text = fs::read_to_string(path).expect("read specimen stats");
    let value = |name: &str| {
        text.lines()
            .find_map(|line| {
                line.split_once('\t')
                    .filter(|(key, _)| *key == name)
                    .map(|(_, value)| value)
            })
            .unwrap_or_else(|| panic!("missing specimen stat {name}"))
    };
    assert_eq!(value("specimen_revision"), PINNED_REVISION);
    SpecimenStats {
        source_frames: value("source_frames").parse().expect("source frames"),
        output_frames: value("output_frames").parse().expect("output frames"),
        elapsed_seconds: value("elapsed_seconds").parse().expect("elapsed seconds"),
        peak_rss_bytes: value("peak_rss_bytes").parse().expect("peak RSS"),
        synthesis_frames: value("synthesis_frames").parse().expect("synthesis frames"),
        time_groups: value("time_groups").parse().expect("time groups"),
        track_visits: value("track_visits").parse().expect("track visits"),
        track_births: value("track_births").parse().expect("track births"),
        track_deaths: value("track_deaths").parse().expect("track deaths"),
        maximum_tracks_per_time: value("maximum_tracks_per_time")
            .parse()
            .expect("maximum tracks"),
        maximum_track_visits_per_output_read: value("maximum_track_visits_per_output_read")
            .parse()
            .expect("maximum output-read visits"),
    }
}

fn command_text(program: impl AsRef<std::ffi::OsStr>, args: &[OsString]) -> String {
    let output = Command::new(program)
        .args(args)
        .output()
        .expect("run command");
    assert!(output.status.success());
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    String::from_utf8_lossy(bytes).trim().to_owned()
}
