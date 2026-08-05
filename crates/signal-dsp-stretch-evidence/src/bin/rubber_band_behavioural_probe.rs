#[path = "rubber_band_behavioural_probe/controls.rs"]
mod controls;
#[path = "rubber_band_behavioural_probe/measure.rs"]
mod measure;

use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::Command,
};

use controls::{controls, Control};
use measure::{hash_samples, measure};

const TOOL: &str = "/opt/homebrew/bin/rubberband";
const RATIOS: [f64; 4] = [1.0, 0.75, 1.25, 1.5];

#[derive(Clone, Copy)]
struct Mode {
    id: &'static str,
    args: &'static [&'static str],
    stereo: bool,
}

const MODES: [Mode; 5] = [
    Mode {
        id: "r2-default",
        args: &["--fast", "--crisp", "5"],
        stereo: true,
    },
    Mode {
        id: "r2-no-reset",
        args: &["--fast", "--no-transients"],
        stereo: false,
    },
    Mode {
        id: "r2-no-lamination",
        args: &["--fast", "--no-lamination"],
        stereo: false,
    },
    Mode {
        id: "r3-standard",
        args: &["--fine"],
        stereo: true,
    },
    Mode {
        id: "r3-short",
        args: &["--fine", "--window-short"],
        stereo: true,
    },
];

fn main() {
    if let Err(error) = run() {
        eprintln!("rubber-band behavioural probe failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let output_dir = output_dir();
    fs::create_dir_all(&output_dir)?;
    let version = command_output(TOOL, &["--version"])?;
    let all_controls = controls();
    let mut manifest = tsv(&output_dir.join("manifest.tsv"))?;
    let mut report = tsv(&output_dir.join("waveform-report.tsv"))?;
    let mut capability = tsv(&output_dir.join("capability.tsv"))?;
    writeln!(
        capability,
        "tool_path\ttool_version\tcli_modes\tdirect_state_status\tdirect_state_reason"
    )?;
    writeln!(
        capability,
        "{TOOL}\t{}\tsupported\tunsupported\tpublic-api adapter not implemented in Batch 29.6BE",
        version.trim()
    )?;
    writeln!(manifest, "probe_id\tfamily\tchannels\tsample_rate_hz\tsource_frames\tevent_frames\tratio\tmode\tcommand_args\tsource_hash")?;
    writeln!(report, "probe_id\tfamily\tchannels\tratio\tmode\toutput_frames\texpected_frames\tlength_error\tpeak\tnon_finite\tclipped\tevent_offsets\tmean_abs_event_offset\tcrest_db\treplica_ratio\tendpoint_energy\tadded_silence\tvertical_coherence\tmean_spectral_residual\ttonal_movement_delta\tunsupported_mass\tstereo_image_delta\toutput_hash\tmeasurement_hash\trepeat_match")?;

    let mut rows = 0;
    for control in &all_controls {
        let ratios: &[f64] = if control.channels == 1 {
            &RATIOS
        } else {
            &[0.75, 1.5]
        };
        for ratio in ratios {
            for mode in MODES
                .iter()
                .filter(|mode| control.channels == 1 || mode.stereo)
            {
                let probe_id = format!("{}-{:.2}-{}", control.id, ratio, mode.id);
                let source_path = output_dir.join(format!("source-{}.wav", control.id));
                if !source_path.exists() {
                    write_wav(&source_path, control)?;
                }
                let source_hash = hash_samples(&control.samples);
                writeln!(
                    manifest,
                    "{}\t{}\t{}\t48000\t{}\t{}\t{:.2}\t{}\t{}\t{:016x}",
                    probe_id,
                    control.family,
                    control.channels,
                    control.frames(),
                    event_list(&control.events),
                    ratio,
                    mode.id,
                    command_args(mode, *ratio),
                    source_hash
                )?;
                let first_path = output_dir.join(format!("{probe_id}-a.wav"));
                let second_path = output_dir.join(format!("{probe_id}-b.wav"));
                render(&source_path, &first_path, mode, *ratio)?;
                render(&source_path, &second_path, mode, *ratio)?;
                let first = read_wav(&first_path)?;
                let second = read_wav(&second_path)?;
                let repeat_match = first == second;
                if !repeat_match {
                    return Err(format!("repeat mismatch for {probe_id}").into());
                }
                let evidence = measure(control, &first, *ratio);
                writeln!(report, "{}\t{}\t{}\t{:.2}\t{}\t{}\t{}\t{}\t{:.9}\t{}\t{}\t{}\t{:.6}\t{:.6}\t{:.6}\t{:.9}\t{}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:016x}\t{:016x}\t{}", probe_id, control.family, control.channels, ratio, mode.id, evidence.output_frames, evidence.expected_frames, evidence.length_error, evidence.peak, evidence.non_finite, evidence.clipped, evidence.event_offsets, evidence.mean_abs_event_offset, evidence.crest_db, evidence.replica_ratio, evidence.endpoint_energy, evidence.added_silence, evidence.vertical_coherence, evidence.mean_spectral_residual, evidence.tonal_movement_delta, evidence.unsupported_mass, evidence.stereo_image_delta, evidence.output_hash, evidence.measurement_hash, repeat_match)?;
                rows += 1;
            }
        }
    }
    if rows != 264 {
        return Err(format!("expected 264 rows, got {rows}").into());
    }
    writeln!(
        capability,
        "rows\t264\trepeat\tpassed\tall rendered sample hashes repeat"
    )?;
    println!(
        "rubber_band_behavioural_probe rows={rows} output={}",
        output_dir.display()
    );
    Ok(())
}

fn render(source: &Path, output: &Path, mode: &Mode, ratio: f64) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new(TOOL);
    command
        .args(mode.args)
        .arg("--quiet")
        .arg("--time")
        .arg(format!("{ratio:.12}"));
    let status = command.arg(source).arg(output).status()?;
    if !status.success() {
        return Err(format!("rubberband {:?} exited {status}", mode.id).into());
    }
    Ok(())
}

fn write_wav(path: &Path, control: &Control) -> Result<(), Box<dyn Error>> {
    let spec = hound::WavSpec {
        channels: control.channels as u16,
        sample_rate: 48_000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for sample in &control.samples {
        writer.write_sample(*sample)?;
    }
    writer.finalize()?;
    Ok(())
}

fn read_wav(path: &Path) -> Result<Vec<f32>, Box<dyn Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => {
            let scale = 2_f32.powi(spec.bits_per_sample as i32 - 1);
            reader
                .samples::<i32>()
                .map(|sample| sample.map(|value| value as f32 / scale))
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    Ok(samples)
}

fn output_dir() -> PathBuf {
    env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/rubber-band-behavioural-probe"))
}

fn tsv(path: &Path) -> Result<BufWriter<File>, Box<dyn Error>> {
    Ok(BufWriter::new(File::create(path)?))
}
fn event_list(events: &[usize]) -> String {
    events
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
fn command_args(mode: &Mode, ratio: f64) -> String {
    format!("{} --quiet --time {:.12}", mode.args.join(" "), ratio)
}
fn command_output(tool: &str, args: &[&str]) -> Result<String, Box<dyn Error>> {
    let output = Command::new(tool).args(args).output()?;
    if !output.status.success() {
        return Err(format!("{tool} {args:?} failed").into());
    }
    let bytes = if output.stdout.is_empty() {
        output.stderr
    } else {
        output.stdout
    };
    Ok(String::from_utf8(bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_matrix_has_264_rows() {
        let rows: usize = controls()
            .iter()
            .map(|control| if control.channels == 1 { 4 * 5 } else { 2 * 3 })
            .sum();
        assert_eq!(rows, 264);
        assert_eq!(MODES.iter().filter(|mode| mode.stereo).count(), 3);
    }
}
