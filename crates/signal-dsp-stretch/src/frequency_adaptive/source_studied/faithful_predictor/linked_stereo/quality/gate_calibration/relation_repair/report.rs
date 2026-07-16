use std::{fs, path::Path};

use super::{ExternalEngines, RelationRepairDirection, RelationRepairRow};

pub(super) fn write(
    root: &Path,
    engines: &ExternalEngines,
    rows: &[RelationRepairRow],
    mechanics_errors: [f64; 5],
    silent_peer_peak: f64,
    reference_failures: usize,
    repaired_failures: usize,
    localization_failures: usize,
    repeated: bool,
    direction: RelationRepairDirection,
) {
    let mut report = format!(
        "signalsmith_revision\t{}\nrubber_band_version\t{}\nrepeated\t{repeated}\ndirection\t{direction:?}\nreference_failures\t{reference_failures}\nrepaired_failures\t{repaired_failures}\nlocalization_failures\t{localization_failures}\nmechanics_errors\t{:e},{:e},{:e},{:e},{:e}\nsilent_peer_peak\t{silent_peer_peak:e}\nratio\tframes\tphase\tbin_aligned\tcontrol\tapplied\tm00\tm01\tm10\tm11\tscope\tcurrent_ipd\trepaired_ipd\tideal_ipd\trubber_ipd\tcurrent_mid_side\trepaired_mid_side\tideal_mid_side\trubber_mid_side\tcurrent_correlation\trepaired_correlation\tcurrent_relation\trepaired_relation\tideal_relation\trubber_relation\tlocal_improved\tlocal_windows\tlocal_before\tlocal_after\tenergy_error\n",
        engines.signalsmith_revision,
        engines.rubber_band_version,
        mechanics_errors[0],
        mechanics_errors[1],
        mechanics_errors[2],
        mechanics_errors[3],
        mechanics_errors[4],
    );
    for row in rows {
        for scope in 0..2 {
            let current = row.current[scope];
            let repaired = row.repaired[scope];
            let ideal = row.ideal[scope];
            let rubber = row.rubber_band[scope];
            report.push_str(&format!("{:.2}\t{}\t{:.2}\t{}\t{}\t{}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{}\t{}\t{:.12e}\t{:.12e}\t{:.12e}\n", row.ratio, row.source_frames, row.phase, row.bin_aligned, row.control, row.applied, row.matrix[0][0], row.matrix[0][1], row.matrix[1][0], row.matrix[1][1], if scope == 0 { "whole" } else { "interior" }, current.ipd_error_radians, repaired.ipd_error_radians, ideal.ipd_error_radians, rubber.ipd_error_radians, current.mid_side_delta_db, repaired.mid_side_delta_db, ideal.mid_side_delta_db, rubber.mid_side_delta_db, current.correlation_delta, repaired.correlation_delta, current.relation_residual, repaired.relation_residual, ideal.relation_residual, rubber.relation_residual, row.local_windows_improved, row.local_windows, row.maximum_local_residuals[0], row.maximum_local_residuals[1], row.energy_error));
        }
    }
    fs::write(root.join("relation-repair.tsv"), report).expect("write relation repair report");
}
