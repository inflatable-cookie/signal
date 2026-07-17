use std::fs;

use super::{Run, TrajectoryAttributionDirection};

#[allow(clippy::too_many_arguments)]
pub(super) fn write(
    root: &std::path::Path,
    run: &Run,
    repeated: bool,
    failures: [usize; 3],
    baseline_to_independent: [usize; 2],
    independent_to_shared: [usize; 2],
    local_regressions: [usize; 2],
    direction: TrajectoryAttributionDirection,
) {
    let mut report = format!(
        "repeated\t{repeated}\nfailures\t{},{},{}\nbaseline_to_independent\t{},{}\nindependent_to_shared\t{},{}\nlocal_regressions\t{},{}\npeak_region_counts\t{},{},{},{}\nevidence_hash\t{:016x}\ndirection\t{direction:?}\nratio\tframes\tphase\tbin_aligned\tcontrol\tscope\tbaseline_ipd\tindependent_ipd\tshared_ipd\tbaseline_mid_side\tindependent_mid_side\tshared_mid_side\tbaseline_correlation\tindependent_correlation\tshared_correlation\tbaseline_relation\tindependent_relation\tshared_relation\tindependent_structural\tshared_structural\tbaseline_to_independent_local\tindependent_to_shared_local\tbaseline_local\tindependent_local\tshared_local\tregions\teligible\tshared_bins\tindependent_bins\tbaseline_hash\tindependent_hash\tshared_hash\n",
        failures[0], failures[1], failures[2],
        baseline_to_independent[0], baseline_to_independent[1],
        independent_to_shared[0], independent_to_shared[1],
        local_regressions[0], local_regressions[1],
        run.peak_region_counts[0], run.peak_region_counts[1],
        run.peak_region_counts[2], run.peak_region_counts[3], run.evidence_hash,
    );
    for row in &run.rows {
        for scope in 0..2 {
            let metrics = row.metrics.map(|stage| stage[scope]);
            report.push_str(&format!(
                "{:.2}\t{}\t{:.2}\t{}\t{}\t{}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{}\t{}\t{}\t{}\t{:.12e}\t{:.12e}\t{:.12e}\t{}\t{}\t{}\t{}\t{:016x}\t{:016x}\t{:016x}\n",
                row.ratio, row.source_frames, row.phase, row.bin_aligned, row.control,
                ["whole", "interior"][scope], metrics[0].ipd_error_radians,
                metrics[1].ipd_error_radians, metrics[2].ipd_error_radians,
                metrics[0].mid_side_delta_db, metrics[1].mid_side_delta_db,
                metrics[2].mid_side_delta_db, metrics[0].correlation_delta,
                metrics[1].correlation_delta, metrics[2].correlation_delta,
                metrics[0].relation_residual, metrics[1].relation_residual,
                metrics[2].relation_residual, row.structural_failures[0],
                row.structural_failures[1], row.local_windows_improved[0],
                row.local_windows_improved[1], row.maximum_local_residuals[0],
                row.maximum_local_residuals[1], row.maximum_local_residuals[2],
                row.peak_region_counts[0], row.peak_region_counts[1],
                row.peak_region_counts[2], row.peak_region_counts[3], row.hashes[0],
                row.hashes[1], row.hashes[2],
            ));
        }
    }
    fs::write(root.join("trajectory-attribution.tsv"), report)
        .expect("write trajectory attribution report");
}
