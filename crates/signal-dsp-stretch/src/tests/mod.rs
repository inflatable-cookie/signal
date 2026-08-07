//! Unit tests for signal-dsp-stretch (split by topic).

pub(super) use super::*;
pub(super) use signal_primitives::SampleRate;

mod support;

mod acceptance_metrics;
mod backend_plan;
mod corpus_benchmark;
mod dynamic_ratio;
mod length_contract;
mod linked_stereo;
mod loop_boundary_metrics;
mod offline_high_quality;
mod passthrough;
mod pitch;
mod selectors;
mod stereo_image_metrics;
mod transient_metrics;
