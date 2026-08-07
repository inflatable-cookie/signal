//! In-child CLAP instance hosting: entry/factory loading, instance
//! lifecycle (create/init/activate/start-processing), parameter inventory,
//! and a raw process session for the sandbox audio thread.
//!
//! This module is the FFI half of phase-1 plugin hosting. It runs inside the
//! sandbox child process only — the parent never touches plugin code. The
//! entry/factory loading is shared with discovery (`entry_loading` below), so
//! hosting and scanning speak the same dlopen path.

mod entry;
mod host;
mod instance;
mod process;

pub use entry::{ClapHostingError, LoadedClapEntry};
pub use host::ClapHostParamsEvent;
pub use instance::{ClapHostedInstance, ClapHostedPortLayout};
pub use process::ClapProcessSession;
