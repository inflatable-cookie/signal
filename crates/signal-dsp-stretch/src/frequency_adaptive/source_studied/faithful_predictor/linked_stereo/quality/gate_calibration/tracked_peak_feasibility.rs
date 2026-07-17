use super::peak_region_feasibility;
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::render;

pub(in crate::frequency_adaptive) use peak_region_feasibility::{
    PeakRegionDirection as TrackedPeakDirection, PeakRegionReview as TrackedPeakReview,
};

pub(in crate::frequency_adaptive) fn review() -> TrackedPeakReview {
    peak_region_feasibility::review_candidate(
        "stretch-linked-stereo-tracked-peak",
        render::linked_tracked_peaks,
    )
}
