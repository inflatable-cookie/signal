use std::collections::BTreeSet;

use super::HASH_OFFSET;

mod reachability;
pub(super) use reachability::reachability_review;
mod objective_grid;
pub(super) use objective_grid::objective_grid_review;
mod listening_export;
pub(super) use listening_export::export_development_pack;
mod smear_attribution;
pub(super) use smear_attribution::{
    smear_attribution_review, Direction as SmearAttributionDirection,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Sensitivity {
    Responsive,
    Conservative,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ResetScope {
    ShortOnly,
    ConfidenceOwned,
    FrequencyLimited,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Configuration {
    pub geometry: [usize; 3],
    pub sensitivity: Sensitivity,
    pub unity_strength_index: usize,
    pub reset_scope: ResetScope,
    pub vertical_alignment: bool,
}

impl Configuration {
    pub(super) fn unity_strength(self) -> f64 {
        [0.0, 0.5, 1.0][self.unity_strength_index]
    }

    pub(super) fn stable_id(self) -> String {
        format!(
            "g{}-s{}-u{}-r{}-v{}",
            self.geometry[0],
            match self.sensitivity {
                Sensitivity::Responsive => "r",
                Sensitivity::Conservative => "c",
            },
            self.unity_strength_index,
            match self.reset_scope {
                ResetScope::ShortOnly => "s",
                ResetScope::ConfidenceOwned => "c",
                ResetScope::FrequencyLimited => "f",
            },
            usize::from(self.vertical_alignment),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Direction {
    ExecuteObjectiveGrid,
    ConfigurationContractRedesign,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Review {
    pub configuration_count: usize,
    pub unique_configuration_count: usize,
    pub dimension_counts: [usize; 5],
    pub development_rows: Vec<&'static str>,
    pub holdout_rows: Vec<&'static str>,
    pub family_counts: [[usize; 5]; 2],
    pub hashes: [u64; 3],
    pub direction: Direction,
}

pub(super) fn configurations() -> Vec<Configuration> {
    let geometries = [
        [256, 1_024, 4_096],
        [512, 2_048, 8_192],
        [1_024, 4_096, 16_384],
    ];
    let sensitivities = [Sensitivity::Responsive, Sensitivity::Conservative];
    let scopes = [
        ResetScope::ShortOnly,
        ResetScope::ConfidenceOwned,
        ResetScope::FrequencyLimited,
    ];
    let mut result = Vec::with_capacity(108);
    for geometry in geometries {
        for sensitivity in sensitivities {
            for unity_strength_index in 0..3 {
                for reset_scope in scopes {
                    for vertical_alignment in [false, true] {
                        result.push(Configuration {
                            geometry,
                            sensitivity,
                            unity_strength_index,
                            reset_scope,
                            vertical_alignment,
                        });
                    }
                }
            }
        }
    }
    result
}

pub(super) fn review() -> Review {
    let configurations = configurations();
    let unique = configurations.iter().copied().collect::<BTreeSet<_>>();
    let development_rows = development_rows();
    let holdout_rows = holdout_rows();
    let family_counts = [
        family_counts(&development_rows),
        family_counts(&holdout_rows),
    ];
    let mut hashes = [HASH_OFFSET; 3];
    for configuration in &configurations {
        hash_bytes(&mut hashes[0], configuration.stable_id().as_bytes());
        hash_bytes(
            &mut hashes[1],
            &configuration.unity_strength().to_bits().to_le_bytes(),
        );
    }
    for row in development_rows.iter().chain(&holdout_rows) {
        hash_bytes(&mut hashes[2], row.as_bytes());
    }
    let pass = configurations.len() == 108
        && unique.len() == 108
        && development_rows.len() == 9
        && holdout_rows.len() == 6
        && development_rows
            .iter()
            .all(|row| !holdout_rows.contains(row))
        && family_counts == [[2, 2, 2, 1, 2], [1, 1, 1, 2, 1]];
    Review {
        configuration_count: configurations.len(),
        unique_configuration_count: unique.len(),
        dimension_counts: [3, 2, 3, 3, 2],
        development_rows,
        holdout_rows,
        family_counts,
        hashes,
        direction: if pass {
            Direction::ExecuteObjectiveGrid
        } else {
            Direction::ConfigurationContractRedesign
        },
    }
}

pub(super) fn development_rows() -> Vec<&'static str> {
    vec![
        "L001", "L002", "L004", "L005", "L007", "L008", "L010", "L013", "L014",
    ]
}

fn holdout_rows() -> Vec<&'static str> {
    vec!["L003", "L006", "L009", "L011", "L012", "L015"]
}

fn family_counts(rows: &[&str]) -> [usize; 5] {
    let mut counts = [0; 5];
    for row in rows {
        let number = row[1..].parse::<usize>().expect("frozen row id");
        counts[(number - 1) / 3] += 1;
    }
    counts
}

fn hash_bytes(state: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *state = (*state ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
}
