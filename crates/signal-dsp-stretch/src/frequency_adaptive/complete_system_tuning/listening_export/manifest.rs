use std::path::PathBuf;

use super::super::{Configuration, ResetScope, Sensitivity, HASH_OFFSET};

pub(super) struct Row {
    pub id: &'static str,
    pub source: String,
    pub rubber_band: String,
    pub ratio: f64,
}

pub(super) fn candidate_configurations() -> [Configuration; 3] {
    [
        config(Sensitivity::Responsive, 0, true),
        config(Sensitivity::Responsive, 1, true),
        config(Sensitivity::Conservative, 1, false),
    ]
}

fn config(
    sensitivity: Sensitivity,
    unity_strength_index: usize,
    vertical_alignment: bool,
) -> Configuration {
    Configuration {
        geometry: [512, 2_048, 8_192],
        sensitivity,
        unity_strength_index,
        reset_scope: ResetScope::ConfidenceOwned,
        vertical_alignment,
    }
}

pub(super) fn assignment(row: &str, count: usize) -> Vec<usize> {
    let mut values = (0..count).collect::<Vec<_>>();
    values.sort_by_key(|index| stable_hash(format!("{row}:{index}").as_bytes()));
    values
}

pub(super) fn rows() -> [Row; 9] {
    [
        row("L001", "0000-drums_percussion-000002", "0p750000", 0.75),
        row("L002", "0000-drums_percussion-000002", "1p250000", 1.25),
        row("L004", "0004-bass-000236", "0p750000", 0.75),
        row("L005", "0004-bass-000236", "1p250000", 1.25),
        row("L007", "0008-vocals-000010", "0p750000", 0.75),
        row("L008", "0008-vocals-000010", "1p250000", 1.25),
        row("L010", "0012-pads_sustains-000423", "0p750000", 0.75),
        row("L013", "0016-full_mix-000144", "0p750000", 0.75),
        row("L014", "0016-full_mix-000144", "1p250000", 1.25),
    ]
}

fn row(id: &'static str, stem: &'static str, ratio_name: &'static str, ratio: f64) -> Row {
    Row {
        id,
        source: format!("{stem}.wav"),
        rubber_band: format!("{stem}-ratio-{ratio_name}.wav"),
        ratio,
    }
}

fn base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-corpus-external-benchmark-pack-fma-broad")
}
pub(super) fn source_root() -> PathBuf {
    base().join("sources")
}
pub(super) fn render_root() -> PathBuf {
    base().join("renders")
}
pub(super) fn export_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-successor-bk-development-pack")
}
fn stable_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(HASH_OFFSET, |state, byte| {
        (state ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3)
    })
}
pub(super) fn readme() -> &'static str {
    "# Stretch Successor Development Pack\n\nStatus: ready for concealed operator listening\n\nNine mono rows. Each row has source plus candidates A-E. Compare transient integrity, tonal stability, grain/ringing, and boundaries. Record one preference and any repeatable broad defect. Keep `development-listening-key.tsv` closed until every notes row is complete. Candidates are three Signal successor configurations, current Signal, and Rubber Band R3. All candidates in a row share RMS target and peak ceiling. Holdout audio is absent.\n"
}
