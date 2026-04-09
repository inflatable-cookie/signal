# 2026-04-08 19:36:11 - g09.002 sandbox broker process tranche

## Summary

Started `g09.002` Batch 2.3 by replacing the synthetic `signal-plugin-sandbox`
shell with a long-lived request-serving broker process.

The sandbox binary now exposes a typed broker lifecycle surface in
`crates/signal-plugin-sandbox/src/broker.rs` and serves commands over stdin
instead of running one synthetic lifecycle and printing one summary line.

## Files

- `crates/signal-plugin-sandbox/src/broker.rs`
- `crates/signal-plugin-sandbox/src/main.rs`
- `docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
- `docs/logs/2026-04/08-193611-g09-002-sandbox-broker-process-tranche.md`

## Validation

- `cargo check -p signal-plugin-sandbox`
- `cargo test -p signal-plugin-sandbox`
- `effigy health`

## Outcome

This tranche completed the first two Batch 2.3 checklist items:

- the synthetic lifecycle shell is replaced by a real request-serving process
  boundary
- startup, ready, attach, running, teardown, crash, timeout, and shutdown are
  now represented as typed broker receipts

What remains open:

- deeper shared-memory lease and ownership hardening
- host-side adoption of the broker boundary
- broader format-specific execution through the new process

## Next Task

Continue `g09.002` Batch 2.3 by hardening broker lease attachment and cleanup
outcomes, then thread one host-side smoke path through the new broker process
before expanding further format behavior.
