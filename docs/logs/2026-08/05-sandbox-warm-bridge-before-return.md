# Sandbox Bridge Warm-Up Before Return

Status: fix landed; under-load rate not re-measured
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

## What Is Not Verified

The under-load rate after the change. The re-measurement was cut short and is not
claimed. Idle, `10` consecutive runs pass and the full gate set is green, which
is weaker evidence than the reproduction that motivated the fix.

## A Note On How That Happened

The load was generated with `yes` processes whose cleanup was the last line of a
command that hit its ten-minute timeout, so the cleanup never ran and they
outlived the measurement. They were killed once noticed.

Self-limiting load — processes that terminate on their own regardless of what
happens to the harness — is the only safe form of this technique.

## Next Task

Re-measure under load when convenient, using self-limiting load processes. If the
warm-up holds, the remaining `process_with_retries` limit is documentation rather
than a defect.
