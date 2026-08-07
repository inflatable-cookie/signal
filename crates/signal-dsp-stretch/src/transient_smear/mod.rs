mod detect;
mod features;
mod measure;
mod types;

#[cfg(any(test, feature = "evidence"))]
pub use detect::detect_stretch_transients;
pub use detect::detect_stretch_transients_with_policy;
#[cfg(any(test, feature = "evidence"))]
pub use measure::measure_transient_smear;
#[cfg(any(test, feature = "evidence"))]
pub(crate) use measure::transient_smear_nan;
pub(crate) use measure::{
    measure_selector_transient_smear, measure_selector_transient_smear_with_input_events,
};
pub use types::StretchTransientDetectorPolicy;
#[cfg(any(test, feature = "evidence"))]
pub use types::StretchTransientEvent;
#[cfg(any(test, feature = "evidence"))]
pub use types::{StretchTransientSmearMeasurement, StretchTransientSmearPolicies};
