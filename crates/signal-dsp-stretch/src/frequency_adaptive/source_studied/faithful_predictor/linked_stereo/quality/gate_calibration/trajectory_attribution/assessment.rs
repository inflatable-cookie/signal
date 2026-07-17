use super::{Metrics, TrajectoryAttributionRow};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::quality::gate_calibration::{
    CALIBRATED_IMAGE_CORRELATION, CALIBRATED_IMAGE_MID_SIDE_DB,
    CALIBRATED_IMAGE_RELATION_RESIDUAL, CALIBRATED_TONE_IPD_RADIANS,
};

pub(super) fn comparisons(
    rows: &[TrajectoryAttributionRow],
    before: usize,
    after: usize,
) -> [usize; 2] {
    let improved = rows
        .iter()
        .filter(|row| comparison(row, before, after).0 && comparison(row, before, after).1)
        .count();
    let regressed = rows
        .iter()
        .filter(|row| !comparison(row, before, after).0)
        .count();
    [improved, regressed]
}

fn comparison(row: &TrajectoryAttributionRow, before: usize, after: usize) -> (bool, bool) {
    let pairs = row.metrics[before].into_iter().zip(row.metrics[after]);
    let values = if row.control == "tone" {
        pairs
            .map(|(before, after)| (before.ipd_error_radians, after.ipd_error_radians))
            .collect::<Vec<_>>()
    } else {
        pairs
            .flat_map(|(before, after)| {
                [
                    (before.mid_side_delta_db, after.mid_side_delta_db),
                    (before.correlation_delta, after.correlation_delta),
                    (before.relation_residual, after.relation_residual),
                ]
            })
            .collect::<Vec<_>>()
    };
    (
        values
            .iter()
            .all(|(before, after)| *after <= *before + 1.0e-12),
        values.iter().any(|(before, after)| *after < *before),
    )
}

pub(super) fn gate(control: &str, metrics: [Metrics; 2]) -> bool {
    metrics.into_iter().all(|metrics| {
        if control == "tone" {
            metrics.ipd_error_radians <= CALIBRATED_TONE_IPD_RADIANS
        } else {
            metrics.mid_side_delta_db <= CALIBRATED_IMAGE_MID_SIDE_DB
                && metrics.correlation_delta <= CALIBRATED_IMAGE_CORRELATION
                && metrics.relation_residual <= CALIBRATED_IMAGE_RELATION_RESIDUAL
        }
    })
}
