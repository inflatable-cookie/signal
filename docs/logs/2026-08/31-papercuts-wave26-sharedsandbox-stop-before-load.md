# Papercuts wave 26 SharedSandbox stop-before-load

Status: closeout
Date: 2026-08-31
Owner: papercuts worker
Handoff: `docs/handoffs/20260831-183500-papercuts-wave26-sharedsandbox-stop-before-load.md`
Branch: `worker/papercuts-wave26-sharedsandbox-stop-before-load`

## Summary

Host SharedSandbox sequential prepare already stops the broker before
`load-plugin-instance`, activates the new member, then restarts processing.
The broker still refuses lifecycle mutation while its audio thread is live
(`already_processing`). Live add-while-processing stays a v1 non-goal.

Proof was incomplete: host tests exercised two sequential prepares, but no
test asserted the refusal token or the stop-then-load unlock. Added
`load_while_processing_rejects_until_boundary_stop` in
`crates/signal-plugin-sandbox/tests/plugin_hosting/multiplex.rs`. Closed the
matching `PAPERCUTS.md` entry. No host/broker behavior change beyond the
regression pin.

## Validation

```text
cargo test -p signal-plugin-sandbox --test plugin_hosting \
  load_while_processing_rejects_until_boundary_stop -- --test-threads=1
# ok 1 passed

cargo test -p signal-host-local --test prepare_plugin_processor \
  prepare_plugin_processor_shared_sandbox -- --test-threads=1
# ok 2 passed
# prepare_plugin_processor_shared_sandbox_shares_one_child
# prepare_plugin_processor_shared_sandbox_child_crash_fans_out

git diff --check
# clean

effigy qa:docs
# exit 0 — link, forbidden, heading, index, next-action passed

effigy qa:northstar
# exit 0 — heading, index, next-action passed
```

## Next Task

Orchestrator reviews the worker PR head and merges when the gate passes.
Do not merge from this worker lane.
