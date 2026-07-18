use std::{fmt::Write, fs};

use super::{ProfessionalComparatorGateDirection, RubberBand, Run};

#[allow(clippy::too_many_arguments)]
pub(super) fn write(
    root: &std::path::Path,
    specimen: &RubberBand,
    run: &Run,
    repeated: bool,
    calibrated_failures: usize,
    local_failures: usize,
    exact_mechanics_failures: usize,
    direction: ProfessionalComparatorGateDirection,
) {
    let mut text = format!(
        "rubber_band_version\t{}\nbinary_path\t{}\nbinary_hash\t{:016x}\ncommand_contract\trubberband -q -3 -t <ratio> <input.wav> <output.wav>\nrepeated\t{repeated}\nstereo_rows\t{}\ncalibrated_failures\t{calibrated_failures}\nsignal_relative_local_failures\t{local_failures}\nmechanics_errors\t{:e},{:e},{:e},{:e},{:e},{:e}\nexact_mechanics_failures\t{exact_mechanics_failures}\ninput_hash\t{:016x}\noutput_hash\t{:016x}\ncommand_hash\t{:016x}\nmeasurement_hash\t{:016x}\ncomparator_envelope_hash\t{:016x}\nevidence_hash\t{:016x}\ndirection\t{direction:?}\nratio\tframes\tphase\tbin_aligned\tcontrol\tscope\tipd_error\tmid_side_delta_db\tcorrelation_delta\trelation_residual\tstructural_failures\tlocal_improved\tlocal_before_max\tlocal_after_max\tinput_hash\toutput_hash\n",
        specimen.version,
        specimen.binary.display(),
        specimen.binary_hash,
        run.rows.len(),
        run.mechanics_errors[0],
        run.mechanics_errors[1],
        run.mechanics_errors[2],
        run.mechanics_errors[3],
        run.mechanics_errors[4],
        run.mechanics_errors[5],
        run.input_hash,
        run.output_hash,
        run.command_hash,
        run.measurement_hash,
        super::comparator_envelope_hash(&run.rows),
        run.evidence_hash,
    );
    for row in &run.rows {
        for (scope, metrics) in [("whole", row.whole), ("interior", row.interior)] {
            writeln!(
                text,
                "{:.2}\t{}\t{:.2}\t{}\t{}\t{scope}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{}\t{}\t{:.12e}\t{:.12e}\t{:016x}\t{:016x}",
                row.ratio,
                row.source_frames,
                row.phase,
                row.bin_aligned,
                row.control,
                metrics.ipd_error_radians,
                metrics.mid_side_delta_db,
                metrics.correlation_delta,
                metrics.relation_residual,
                row.structural_failures,
                row.local_windows_improved,
                row.maximum_local_residuals[0],
                row.maximum_local_residuals[1],
                row.input_hash,
                row.output_hash,
            )
            .expect("format comparator report row");
        }
    }
    fs::write(root.join("report.tsv"), text).expect("write comparator gate-validity report");

    let mut windows = String::from(
        "ratio\tframes\tphase\tbin_aligned\tcontrol\twindow\tsignal_residual\trubber_band_residual\trubber_band_improves\n",
    );
    for row in &run.rows {
        for window in 0..8 {
            writeln!(
                windows,
                "{:.2}\t{}\t{:.2}\t{}\t{}\t{}\t{:.12e}\t{:.12e}\t{}",
                row.ratio,
                row.source_frames,
                row.phase,
                row.bin_aligned,
                row.control,
                window,
                row.local_residuals[0][window],
                row.local_residuals[1][window],
                row.local_residuals[1][window] < row.local_residuals[0][window],
            )
            .expect("format comparator local window");
        }
    }
    fs::write(root.join("local-windows.tsv"), windows)
        .expect("write comparator local-window report");
}
