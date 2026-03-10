---
title: Runtime Prework Cache Lifecycle And Invalidation
status: done
owner: codex
updated: 2026-03-09
tags: [signal, runtime, graph, execution, anticipative, cache, invalidation]
---

## Summary

Promoted the short-lived prework cache from “reusable slot” to an explicit
runtime lifecycle surface.

`signal-runtime` now:

- tracks `prework_cache_state`
- counts invalidations
- records `last_prework_invalidation_reason`
- invalidates cached prework on:
  - runtime reconfigure
  - graph projection change
  - transport projection change
  - non-empty parameter batch application
  - processing-epoch expiry
  - input-signature mismatch

This means cached anticipative work is now retired by real runtime control
state changes instead of only falling out of use implicitly on the next block.

## Validation

- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_invalidates_prework_cache_on_parameter_and_transport_changes -- --nocapture`
- `cargo test -p signal-runtime runtime_reuses_prework_cache_for_matching_adjacent_block -- --nocapture`

## Notes

The current cache is still intentionally small and conservative. The important
change is not more retention depth; it is that cache lifecycle is now owned by
runtime control flow, so future scheduler work has a real invalidation model
to build on.

## Next Task

Separate prework admission from prework consumption, for example by promoting
the current single prepared-result cache into a background-lane state machine
or by adding a stronger freshness policy than the current adjacent-block epoch
window.
