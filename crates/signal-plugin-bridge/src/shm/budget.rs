use std::time::Duration;

/// Hard ceiling of the per-block wait budget, in microseconds.
///
/// The effective budget is `min(1 ms, 50 % of the block duration at the
/// plan rate)` — see [`plugin_process_wait_budget`]. Rationale: at typical
/// block sizes (128–1024 frames at 44.1–96 kHz, ~1.3–23 ms) half a block
/// leaves the rest of the callback comfortably inside its deadline even if
/// EVERY insert misses, and 1 ms caps the damage on very large blocks where
/// half a block would be a pointless long stall.
pub const PLUGIN_PROCESS_WAIT_BUDGET_MAX_MICROS: u64 = 1_000;

/// A single scheduling miss is not evidence that the sandbox child died.
/// Keep bypass bounded, but require a short run of misses before retiring
/// the backend generation.
pub const PLUGIN_PROCESS_CONSECUTIVE_TIMEOUT_LIMIT: u32 = 3;

/// Per-block wait used when the backend is driven offline
/// ([`signal_render_plane::PluginBlockProcessor::set_offline_waiting`]).
///
/// Not a latency target. An offline render has no output buffer to drain, so
/// there is nothing for a short budget to protect: a miss there does not
/// arrive late, it drops the insert for that block and writes the wrong
/// render. This only has to be long enough that a child which lost its
/// scheduling slot on a loaded machine is not mistaken for a dead one, while
/// still bounding a genuinely dead child at
/// `PLUGIN_PROCESS_CONSECUTIVE_TIMEOUT_LIMIT` × this.
pub const PLUGIN_PROCESS_OFFLINE_WAIT_BUDGET: Duration = Duration::from_secs(5);

/// Effective bounded wait for one block: `min(1 ms, 50 % of the block
/// duration at `sample_rate_hz`)`.
pub fn plugin_process_wait_budget(frame_count: usize, sample_rate_hz: u32) -> Duration {
    let half_block_micros =
        (frame_count as u64).saturating_mul(500_000) / u64::from(sample_rate_hz.max(1));
    Duration::from_micros(half_block_micros.min(PLUGIN_PROCESS_WAIT_BUDGET_MAX_MICROS))
}
