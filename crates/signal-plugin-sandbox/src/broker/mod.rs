//! Out-of-process sandbox broker.
//!
//! This binary is the plugin-hosting child process: it exercises the
//! plumbing that out-of-process hosting needs — child-process spawn, a
//! line-oriented stdio control protocol, file-backed shared-memory block
//! transport — and REAL plugin instance hosting (CLAP per g11.012, VST3 per
//! g11.031, AU per g11.032): `load-plugin` selects the format by library
//! path extension (`.clap` → `signal-plugin-clap`'s hosting FFI, `.vst3` →
//! `signal-plugin-vst3`'s COM FFI, `.component` → `signal-plugin-au`'s
//! AudioToolbox FFI, where the registry sentinel path is never opened;
//! second argument = format-native load
//! key), `activate` activates the instance and leases a shared-memory audio
//! block region, and `start-processing` spawns the child's audio thread,
//! which spin/yield-waits on each member region's request stamp and runs the
//! plugin's `process()` for every block the parent posts. SharedSandbox
//! hosts N instances in one child (`load-plugin-instance`); omitted
//! `instance_id` still means `sandbox_id` (DedicatedSandbox single-slot).
//!
//! Wire format: one receipt per line,
//! `signal-plugin-sandbox state=<token> sandbox_id=... instance_id=... epoch=... lease_id=... region_id=... [key=value ...] detail=...`
//! with the legacy states `starting`, `ready`, `attached`, `running`,
//! `timed_out`, `crashed`, `teardown_complete`, `shutdown` plus the plugin
//! lifecycle states `plugin_loaded`, `plugin_activated`,
//! `layout_unsupported`, `processing_started`, `processing_stopped`,
//! `plugin_deactivated`, `plugin_unloaded`. Extra `key=value` tokens carry
//! structured payloads (parameter inventory, shm coordinates); values are
//! percent-encoded where they may contain spaces or separators.
//!
//! Command arguments are whitespace-separated; library paths containing
//! whitespace are not supported by the v1 wire format (the shared-memory
//! root and standard plugin dirs avoid them).

mod hosted;
mod lifecycle;
mod process;
mod shm;
mod types;

pub use process::SandboxBrokerProcess;
pub use types::{encode_wire_token, SandboxBrokerReceipt, SandboxBrokerState};

#[cfg(test)]
mod tests;
