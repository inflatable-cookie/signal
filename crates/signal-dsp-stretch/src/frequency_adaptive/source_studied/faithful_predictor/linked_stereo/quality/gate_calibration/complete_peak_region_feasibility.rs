use super::peak_region_feasibility;
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::render;

pub(in crate::frequency_adaptive) use peak_region_feasibility::{
    PeakRegionDirection as CompletePeakRegionDirection,
    PeakRegionReview as CompletePeakRegionReview,
};

pub(in crate::frequency_adaptive) fn review() -> CompletePeakRegionReview {
    peak_region_feasibility::review_candidate(
        "stretch-linked-stereo-complete-peak-region",
        render::linked_peak_owned_regions,
    )
}
