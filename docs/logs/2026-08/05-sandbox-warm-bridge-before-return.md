# Sandbox Bridge Warm-Up Before Return

Status: warm-up landed but insufficient; the real fix is not done
Created: 2026-08-05
Scope: `signal-plugin-sandbox` `plugin_hosting`, the `A22` family residue

## Reproduced First

The previous note recorded this flake without its condition and had to be
corrected. This time it was reproduced deliberately: `5` failures in `10` runs
under load, against `20` consecutive passes idle.

Three different-looking failures, all the same cause:

```
sample 128: wet 0.5 vs dry 0.5 * 0.5
child never answered a process request within 20s (a retired epoch cannot
  recover by retrying -- see CHILD_FIRST_RESPONSE_DEADLINE)
assertion failed: handle.process_with_events(...)
```

A bare `process_with_events` returning false, the retry deadline firing, and a
render where the insert never applied because the miss bypassed it. The middle
message is the one written earlier in the day predicting exactly this, which is
the first time this generation a diagnosis has been confirmed by a message
written before the failure.

## The Fix

`ShmPluginProcessor` clears `alive` after
`PLUGIN_PROCESS_CONSECUTIVE_TIMEOUT_LIMIT` consecutive misses, and every later
`process` returns false immediately. Under load the child's audio thread can
miss its first three requests while it is still being scheduled, and the epoch is
then retired before the test has done anything.

`spawn_processing_session_for` now warms the bridge before returning: it issues
process requests until one succeeds, re-attaching if the epoch retires while
warming.

That location is the point. The lease is in scope there and nowhere downstream,
which is why the `A19` re-attach could not be applied at the twelve call sites.
Warming at construction means callers start from an epoch that has already
answered, rather than spending their own budget discovering the child was not
ready.

## The Warm-Up Is Not The Fix

Re-measured under verified load: **`5` failures in `6` runs**. The warm-up did
not help.

Two intermediate measurements said otherwise and both were worthless:

- `0/8` "under load" where the load was launched as `( timeout 240 yes & )` in a
  subshell and `pgrep` found nothing. macOS has no `timeout` — it is GNU
  coreutils — so the load never started and the runs were idle.
- `0/10` idle, which was never evidence about a load-dependent failure.

Only the third measurement, which checked `1710%` CPU across `18` cores *before*
running anything, is worth reading. The rule this generation keeps relearning
again: a measurement has to be shown capable of seeing the thing, and that
includes checking the *conditions* were applied, not just the result.

## Why It Failed

The failing assertions say it. `child never answered a process request within
20s (a retired epoch cannot recover by retrying)` fires inside
`process_with_retries`, which runs long after setup, and `wet 0.5 vs dry 0.5 *
0.5` is a miss mid-render.

The epoch retires *during* the test, not during setup. Sustained load makes the
child miss three in a row at any point, and warming only proves it answered once
beforehand. The warm-up addresses setup-time retirement, which was never the
dominant mode.

## The Actual Fix, Not Done

The `A19` re-attach, applied at the point of use: re-attach when `is_alive` goes
false inside `process_with_retries` and at the bare `process_with_events` call
sites. That needs the lease threaded to twelve call sites, which is real work.

The warm-up is kept because it is harmless and does close setup-time retirement,
but it is documented as insufficient rather than as a fix.

## Next Task

Thread the lease to the twelve call sites and re-attach on retirement at the
point of use. Verify under load with the conditions checked before the run —
shell busy-loops that self-terminate, since macOS has no `timeout`.
