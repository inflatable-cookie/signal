//! Audio-thread CLAP process session.

mod buffers;
mod events;
mod session;

pub use session::ClapProcessSession;

pub(crate) use events::{
    param_in_events_get, param_in_events_size, param_out_events_try_push, InEventSlot,
    ParamEventList, ParamOutCapture,
};
