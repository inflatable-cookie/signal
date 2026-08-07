//! In-child LV2 instance hosting: bundle re-parse, dlopen +
//! `lv2_descriptor(index)` walk, instance lifecycle (instantiate at
//! activate / connect ports / run / deactivate / cleanup), control ports as
//! the parameter inventory, and a raw process session for the sandbox
//! audio thread — the LV2 mirror of the CLAP/VST3/AU hosting modules.
//!
//! # FFI design
//!
//! The LV2 C ABI is tiny and handwritten here (house precedent — no
//! `lv2`-crate dependency): one `#[repr(C)]` descriptor struct returned by
//! the library's `lv2_descriptor(uint32_t)` export, walked until the entry
//! whose `URI` matches the load key. The binary self-describes nothing but
//! that URI — the port model comes from re-parsing the bundle TTL at load
//! (`introspection::parse_lv2_bundle`, the same functions discovery uses),
//! paralleling AU's rebuild-description-from-load-key.
//!
//! LV2 is a pure push model: no COM, no pull callback, no
//! start/stop-processing handshake. `instantiate` fixes the sample rate,
//! so it runs at ACTIVATE (the wire delivers the rate there), not load.
//! Activation connects every port once — audio ports to preallocated
//! planar buffers, control ports to boxed slots holding their TTL
//! defaults — and `run(n)` per block does the rest. Drop order is
//! deactivate → cleanup → dlclose (the `Library` field is declared last).
//!
//! # Features
//!
//! The host provides `urid:map` ONLY (packet g11.033 decision 3): an
//! interned string→u32 map behind a `Mutex`, handed over as a boxed
//! feature in a NULL-terminated array kept alive for the instance's
//! lifetime. Any other `lv2:requiredFeature` fails the load with the typed
//! `unsupported_required_feature` token (scan pre-filters the same set).

mod instance;
mod process;
mod support;

pub use instance::{Lv2HostedInstance, Lv2HostedPortLayout};
pub use process::Lv2ProcessSession;
pub use support::Lv2HostingError;
