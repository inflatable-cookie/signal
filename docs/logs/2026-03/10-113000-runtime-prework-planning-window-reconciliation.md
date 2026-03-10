# 2026-03-10 11:30 UTC — Runtime prework planning-window reconciliation

## Summary

- promoted the bounded anticipative prework queue from host-driven horizon filling into a runtime-owned planning-window API
- added explicit `PlanningWindowRevised` retirement/invalidation semantics when queued future targets fall out of the declared window
- kept the earlier future-state reuse/replacement rules intact inside that planning window
- moved future block-sequence planning for that window into `signal-runtime`, so hosts no longer carry a target-block deque
- made runtime stop/recovery retire queued prework before restart, and prevented disabled prework paths from allocating future block IDs
- updated the local timeout-recovery proof to reflect the real planning-window scheduler behavior

## Implementation notes

- `signal-runtime` now exposes `RuntimePreworkWindowTarget` plus `prepare_engine_prework_window(...)`
- hosts now declare their small future-block horizon to runtime in one call instead of issuing repeated single-target admissions
- runtime now plans the future block sequences for that horizon itself, so host code only constructs future-state targets for runtime-provided block IDs
- runtime now reports planning-window target count and target block sequences in `RuntimeEngineBlockSnapshot`
- revised windows proactively retire queued future work that is no longer part of the declared horizon
- stopping the runtime now retires queued prework before restart, and disabled prework paths no longer allocate future block sequences they cannot consume

## Validation

- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_planning_window_retires_future_entries_not_in_revised_window -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`

## Next

- move the planning window from host-declared horizon sizing toward more runtime-owned scheduler policy, most likely by letting runtime maintain a target future-block window size and compare revised parameter/transport plans against that window without the host having to carry the target block deque itself
