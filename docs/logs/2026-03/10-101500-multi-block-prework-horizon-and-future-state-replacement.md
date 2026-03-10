# 2026-03-10 10:15 UTC — Multi-block prework horizon and future-state replacement

## Summary

- extended the bounded runtime prework queue from single-future opportunistic use into a real multi-block priming path
- changed queued admission matching so identical future target-state admissions reuse the existing queued entry instead of churning on a different `admitted_from_block_sequence`
- kept explicit replacement semantics for changed future parameter or transport state on the same target block via `SupersededByAdmission`
- updated local host timeout-recovery assertions to the new multi-block queue behavior

## Implementation notes

- `signal-runtime` now treats queued prework identity as the future execution target state rather than the source block that requested it
- `signal-host-local` and `signal-host-server` now keep a small primed future-block horizon instead of only priming one next block
- the local timeout-recovery path now shows the queue behaving as real scheduler state under transport churn: more admissions and retirements, but fewer final queued consumptions because stale future work is being replaced or retired before use

## Validation

- `cargo test -p signal-runtime runtime_reuses_existing_future_queue_entry_when_target_state_matches -- --nocapture`
- `cargo test -p signal-runtime runtime_replaces_future_queue_entry_when_target_state_changes -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`

## Next

- promote the bounded queue from host-driven horizon priming into a more runtime-owned scheduler path, most likely by letting runtime track more than one future admitted block as an explicit planning window and retire queued work proactively when future parameter/transport plans are revised
