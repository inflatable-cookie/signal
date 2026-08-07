//! Audio-thread helpers that fill stage scratch from compiled clips.

mod clips;
mod envelope;
mod events;
mod interpolate;

pub(crate) use clips::render_clips_into_scratch;
pub(crate) use envelope::{clip_window_gain, sample_envelope};
pub(crate) use events::insertion_sort_events_by_offset;
pub(crate) use interpolate::interpolate_source_frame;
