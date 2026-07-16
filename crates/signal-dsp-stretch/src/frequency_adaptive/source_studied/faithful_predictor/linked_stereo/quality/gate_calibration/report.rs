use std::{fs, path::Path};

use super::{external::ExternalEngines, CalibrationRow, GateCalibrationDirection};

pub(super) fn write(
    root: &Path,
    engines: &ExternalEngines,
    rows: &[CalibrationRow],
    repeated: bool,
    sensitive: bool,
    negative_residuals: [f64; 2],
    direction: GateCalibrationDirection,
) {
    let mut report = format!(
        "signalsmith_revision\t{}\nsignalsmith_version\t{}\nrubber_band_version\t{}\nrepeated\t{repeated}\nnegative_control_sensitive\t{sensitive}\ncollapsed_relation_residual\t{:.12e}\ncrossfed_relation_residual\t{:.12e}\ndirection\t{direction:?}\nratio\tframes\tphase\tfrequency\tbin_aligned\tcontrol\tengine\tscope\tipd_error\tmid_side_delta_db\tcorrelation_delta\trelation_residual\tstructural_failures\tinput_hash\toutput_hash\n",
        engines.signalsmith_revision,
        engines.signalsmith_version,
        engines.rubber_band_version,
        negative_residuals[0],
        negative_residuals[1],
    );
    for row in rows {
        for (scope, metrics) in [("whole", row.whole), ("interior", row.interior)] {
            report.push_str(&format!("{:.2}\t{}\t{:.2}\t{:.6}\t{}\t{}\t{}\t{}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{}\t{:016x}\t{:016x}\n", row.ratio, row.source_frames, row.phase, row.frequency_hz, row.bin_aligned, row.control, row.engine, scope, metrics.ipd_error_radians, metrics.mid_side_delta_db, metrics.correlation_delta, metrics.relation_residual, row.structural_failures, row.hashes[0], row.hashes[1]));
        }
    }
    fs::write(root.join("calibration.tsv"), report).expect("write stereo calibration report");
}
