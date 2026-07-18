use std::{fs, path::PathBuf};

use super::{MONO_FRAMES, MONO_RATIOS, MONO_SAMPLE_RATE};
use crate::frequency_adaptive::source_studied::{confirmation, long_form};

use super::super::{
    external::{file_hash, read_stereo, write_stereo},
    metrics::{control, ControlKind},
    ALIGNMENTS, LENGTHS, PHASES, RATIOS, SAMPLE_RATE,
};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::coherent_representation;

#[derive(Clone)]
pub(super) struct PreparedStereoRow {
    pub(super) ratio: f64,
    pub(super) source_frames: usize,
    pub(super) phase: f64,
    pub(super) frequency: f64,
    pub(super) bin_aligned: bool,
    pub(super) kind: ControlKind,
    pub(super) source: [Vec<f64>; 2],
}

#[derive(Clone)]
pub(super) struct PreparedMonoRow {
    pub(super) id: &'static str,
    pub(super) ratio: f64,
    pub(super) source: Vec<f64>,
    pub(super) rubber_band: Option<Vec<f64>>,
}

pub(super) struct PreparedInputs {
    pub(super) stereo: Vec<PreparedStereoRow>,
    pub(super) mono: Vec<PreparedMonoRow>,
}

pub(super) fn prepare(root: &std::path::Path) -> PreparedInputs {
    let input_root = root.join("frozen-inputs");
    fs::create_dir_all(&input_root).expect("create SBSMS frozen input root");
    let geometry = coherent_representation::source_geometry(SAMPLE_RATE);
    let spacing = SAMPLE_RATE as f64 / geometry[2] as f64;
    let mut stereo = Vec::with_capacity(48);
    let mut manifest = String::from("family\tid\tratio\tframes\tchannels\tinput_hash\n");
    for source_frames in LENGTHS {
        for phase in PHASES {
            for bin_aligned in ALIGNMENTS {
                let frequency = (31.5 + if bin_aligned { 0.0 } else { 0.37 }) * spacing;
                for kind in [ControlKind::Tone, ControlKind::Image] {
                    let source = control(kind, source_frames, frequency, phase);
                    for ratio in RATIOS {
                        let id = format!(
                            "{}-{source_frames}-{phase:.2}-{bin_aligned}-{ratio:.2}",
                            kind.name()
                        );
                        let path = input_root.join(format!("stereo-{id}.wav"));
                        write_stereo(&path, &source, SAMPLE_RATE as u32);
                        let source = read_stereo(&path, source_frames, SAMPLE_RATE as u32);
                        manifest.push_str(&format!(
                            "stereo\t{id}\t{ratio:.2}\t{source_frames}\t2\t{:016x}\n",
                            file_hash(&path)
                        ));
                        stereo.push(PreparedStereoRow {
                            ratio,
                            source_frames,
                            phase,
                            frequency,
                            bin_aligned,
                            kind,
                            source,
                        });
                    }
                }
            }
        }
    }
    assert_eq!(stereo.len(), 48);

    let mut mono = Vec::new();
    for (id, source) in mono_controls() {
        for ratio in MONO_RATIOS {
            manifest.push_str(&format!(
                "synthetic\t{id}\t{ratio:.2}\t{}\t1\t{:016x}\n",
                source.len(),
                sample_hash(&source)
            ));
            mono.push(PreparedMonoRow {
                id,
                ratio,
                source: source.clone(),
                rubber_band: None,
            });
        }
    }

    let development_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-source-studied-db-exact-source");
    for case in long_form::cases() {
        let source = confirmation::read_exact(
            &development_root
                .join("inputs")
                .join(format!("{}.wav", case.id)),
            220_500,
            16,
        );
        let target = (source.len() as f64 * case.ratio).round() as usize;
        let rubber_band = confirmation::read_exact(
            &development_root
                .join("rubber-band-r3")
                .join(format!("{}.wav", case.id)),
            target,
            0,
        );
        manifest.push_str(&format!(
            "development\t{}\t{:.2}\t{}\t1\t{:016x}\n",
            case.id,
            case.ratio,
            source.len(),
            sample_hash(&source)
        ));
        mono.push(PreparedMonoRow {
            id: case.id,
            ratio: case.ratio,
            source,
            rubber_band: Some(rubber_band),
        });
    }
    fs::write(input_root.join("manifest.tsv"), manifest).expect("write frozen input manifest");
    PreparedInputs { stereo, mono }
}

fn mono_controls() -> Vec<(&'static str, Vec<f64>)> {
    let mut isolated = vec![0.0; MONO_FRAMES];
    isolated[MONO_FRAMES / 2] = 0.9;
    let mut dense = vec![0.0; MONO_FRAMES];
    for (index, gain) in [0.9, 0.72, 0.84, 0.68, 0.8].into_iter().enumerate() {
        dense[MONO_FRAMES * (index + 2) / 8] = gain;
    }
    let tone = (0..MONO_FRAMES)
        .map(|index| {
            0.3 * (std::f64::consts::TAU * 372.37 * index as f64 / MONO_SAMPLE_RATE as f64).sin()
        })
        .collect();
    let chord = (0..MONO_FRAMES)
        .map(|index| {
            [110.0, 164.8138, 246.9417, 369.9944]
                .into_iter()
                .enumerate()
                .map(|(tone, frequency)| {
                    0.11 * (std::f64::consts::TAU * frequency * index as f64
                        / MONO_SAMPLE_RATE as f64
                        + tone as f64 * 0.41)
                        .sin()
                })
                .sum()
        })
        .collect();
    let crossing = (0..MONO_FRAMES)
        .map(|index| {
            let progress = index as f64 / (MONO_FRAMES - 1) as f64;
            let first = 180.0 + 420.0 * progress;
            let second = 600.0 - 420.0 * progress;
            0.2 * (std::f64::consts::TAU * first * index as f64 / MONO_SAMPLE_RATE as f64).sin()
                + 0.2
                    * (std::f64::consts::TAU * second * index as f64 / MONO_SAMPLE_RATE as f64
                        + 0.7)
                        .sin()
        })
        .collect();
    let decay = (0..MONO_FRAMES)
        .map(|index| {
            let envelope = (-6.0 * index as f64 / MONO_FRAMES as f64).exp();
            envelope
                * 0.55
                * (std::f64::consts::TAU * 220.3 * index as f64 / MONO_SAMPLE_RATE as f64).sin()
        })
        .collect();
    let mut state = 0x6d2b_79f5_a4c3_1e87_u64;
    let noise = (0..MONO_FRAMES)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (((state >> 11) as f64 / ((1_u64 << 53) as f64)) * 2.0 - 1.0) * 0.28
        })
        .collect();
    vec![
        ("tone", tone),
        ("chord", chord),
        ("crossing", crossing),
        ("decay", decay),
        ("isolated-transient", isolated),
        ("dense-transient", dense),
        ("noise", noise),
    ]
}

fn sample_hash(samples: &[f64]) -> u64 {
    samples.iter().fold(0xcbf2_9ce4_8422_2325, |hash, sample| {
        (hash ^ (*sample as f32).to_bits() as u64).wrapping_mul(0x100_0000_01b3)
    })
}
