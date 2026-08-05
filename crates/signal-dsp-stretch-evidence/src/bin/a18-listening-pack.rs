//! Build the `g10.041` `A18` concealed listening pack.
//!
//! Renders the shipped transient-reset path against the high-band-only
//! candidate on material with sustained low content beneath transients, at the
//! ratios where the artifact is largest. Sides are randomised per case and the
//! key is written separately.
//!
//! Contract `084` Rule 5 makes listening the promotion authority; nothing
//! adopts the candidate until this pack is judged.

use std::fs;
use std::path::{Path, PathBuf};

use signal_dsp_stretch::{a18_candidate_stretch_mono, a18_shipped_stretch_mono};

const RATE: u32 = 48_000;
/// `240 Hz` at `48 kHz`, the crossover Batch 41.3 froze.
const CROSSOVER: f64 = 0.010;

/// Bass note plus a percussive attack every `500 ms`.
///
/// `A18` was reported as low-mid pops on ticks, so the material must carry
/// sustained low content *through* the transients — that is precisely the
/// content the shipped reset discards the phase of.
fn material(seconds: f32, fundamental: f32) -> Vec<f32> {
    let frames = (RATE as f32 * seconds) as usize;
    let period = RATE as usize / 2;
    let mut seed = 0x9E3779B9u32;
    let mut noise = move || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) as f32 / 8_388_608.0 - 1.0
    };
    (0..frames)
        .map(|index| {
            let t = index as f32 / RATE as f32;
            // Fundamental plus two harmonics, so the low content is musical
            // rather than a bare sine.
            let bass = 0.30 * (std::f32::consts::TAU * fundamental * t).sin()
                + 0.12 * (std::f32::consts::TAU * fundamental * 2.0 * t).sin()
                + 0.06 * (std::f32::consts::TAU * fundamental * 3.0 * t).sin();
            let since = index % period;
            let attack = if since < (RATE as f32 * 0.030) as usize {
                0.85 * (-(since as f32) / (RATE as f32 * 0.005)).exp() * noise()
            } else {
                0.0
            };
            (bass + attack).clamp(-1.0, 1.0)
        })
        .collect()
}

fn write_wav(path: &Path, samples: &[f32]) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, spec).expect("create wav");
    for sample in samples {
        writer.write_sample(*sample).expect("write sample");
    }
    writer.finalize().expect("finalize wav");
}

/// Deterministic per-case side assignment from a fixed seed, so the pack is
/// reproducible but the ordering is not guessable from the case list.
fn sides_for(case_index: usize) -> (&'static str, &'static str) {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in b"a18-2026-08-05-rev2".iter().chain(&[case_index as u8]) {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    if hash.is_multiple_of(2) {
        ("shipped", "candidate")
    } else {
        ("candidate", "shipped")
    }
}

fn main() {
    let root = PathBuf::from(std::env::var("HOME").expect("HOME"))
        .join("Downloads")
        .join("signal-listening-pack-41-a18");
    let pairs = root.join("pairs");
    fs::create_dir_all(&pairs).expect("create pack dir");

    // Ratios where the measured artifact is largest: 2.752 rad at 2.0, 0.478
    // at 1.5, against a 0.142 floor.
    let cases: Vec<(&str, f32, f64)> =
        vec![("E1", 55.0, 1.5), ("E2", 55.0, 2.0), ("E3", 82.4, 2.0)];

    let mut key = String::from("case\tA\tB\tfundamental_hz\tratio\n");
    let mut notes = String::from("case\tpreferred\tnote\n");

    for (index, (case, fundamental, ratio)) in cases.iter().enumerate() {
        let source = material(6.0, *fundamental);
        let shipped = a18_shipped_stretch_mono(&source, *ratio);
        let candidate = a18_candidate_stretch_mono(&source, *ratio, CROSSOVER);

        let (side_a, side_b) = sides_for(index);
        let pick = |name: &str| -> &Vec<f32> {
            if name == "shipped" {
                &shipped
            } else {
                &candidate
            }
        };
        write_wav(&pairs.join(format!("{case}-A.wav")), pick(side_a));
        write_wav(&pairs.join(format!("{case}-B.wav")), pick(side_b));
        write_wav(&root.join(format!("{case}-source.wav")), &source);

        key.push_str(&format!(
            "{case}\t{side_a}\t{side_b}\t{fundamental}\t{ratio}\n"
        ));
        notes.push_str(&format!("{case}\t\t\n"));
    }

    fs::write(root.join("key.tsv"), key).expect("write key");
    fs::write(root.join("notes.tsv"), notes).expect("write notes");
    fs::write(root.join("README.md"), README).expect("write readme");
    println!("built {}", root.display());
}

const README: &str = r#"# g10.041 A18 Listening Pack

`A18` is the low-mid pop on ticks you reported twice — once in the `g10.036`
round on `C3`, once in `g10.039` revision 1 on `D2` and `D3`.

It now has a measured mechanism. The offline stretcher resets *every* frequency
bin's phase when it detects a transient. High bins have short periods, so a
phase jump there is a small time shift. Low bins have long periods, so the same
jump is a large waveform discontinuity — the pop. Measured as carrier phase
jump at ratio `2.0`: `2.752 rad` against a `0.142 rad` floor, which is within
`11%` of deliberately flipping the bass note's polarity.

The candidate resets only above `240 Hz`, leaving lower bins to propagate
continuously. That takes the measurement to `0.133 rad`, at the floor, while
transient smear measures identical to shipped on the corpus's own metric.

## What you are comparing

One side of each pair is the shipped renderer, one is the candidate. Assignment
is per case in `key.tsv`. Do not open it until `notes.tsv` is filled.

| case | bass | ratio | why |
| --- | --- | --- | --- |
| `E1` | `55 Hz` (A1) | `1.5` | artifact present but smaller: `0.478 rad` |
| `E2` | `55 Hz` (A1) | `2.0` | artifact peaks here: `2.752 rad` |
| `E3` | `82.4 Hz` (E2) | `2.0` | different fundamental, same ratio |

Each `*-source.wav` is the unprocessed input, for reference.

## What to listen for

The pop on each tick, in the bass register. The attacks themselves should sound
the same on both sides — if one side's transients are duller or smearier, that
matters and is worth noting, because it is the risk the fix was designed
against.

Material is a bass note with a percussive attack every `500 ms`, six seconds,
rendered mono.

## Decision

Fill `notes.tsv`, then open `key.tsv`. The candidate is admitted only if no case
prefers the shipped side.

Objective evidence says the artifact is gone at no measured cost. That is not
the same as sounding better, which is what this pack is for.
"#;
