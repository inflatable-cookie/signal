# A19 - Use After Unmap In The Shm Round-Trip Test

Status: complete
Created: 2026-08-05
Scope: finding `A19`, closed with a mechanism

## How It Surfaced

The `test` release gate failed with a segfault, not an assertion:

```
process didn't exit successfully: signal_plugin_bridge-cf6261bae37c786c
  (signal: 11, SIGSEGV: invalid memory reference)
```

Three immediate reruns were clean and macOS produced no crash report. Twelve
runs found it twice, both times naming
`shm::tests::served_request_round_trips_through_the_region` — which is `A19`,
carried since `g10.038` as "no mechanism found".

## The Mechanism

`A19` presented as an intermittent assertion failure *and* an intermittent
segfault. It was both, from one cause.

The test builds `child_view` as a raw pointer into `region`'s shared mapping and
moves it into a spawned server thread that emulates the sandbox child. The
client then retries `handle.process` and asserts the round trip. `server.join()`
sat *after* those assertions.

The retry loop was bounded by iteration count:

```rust
for _ in 0..200 {
    if handle.process(&mut scratch, 32, 2) { processed = true; break; }
}
assert!(processed, "server should have answered within retries");
```

Two hundred iterations of this thread is not a bound on the other thread being
scheduled. It is an assumption about host contention, and under a loaded
machine it loses.

When it loses, `assert!(processed, ...)` panics. Unwinding drops `region`, which
unmaps the backing memory while the server thread is still spinning on
`child_view.request_seq()`. The thread dereferences unmapped memory and the
whole test binary dies with `SIGSEGV` — taking the assertion message with it,
which is why the failure looked like two unrelated flakes.

## The Fix

Both halves are corrected, because either alone leaves a real defect.

The retry loop is bounded by a five-second deadline rather than an iteration
count, so a slow host waits longer instead of failing.

`server.join()` moves *ahead* of every assertion. The mapping now cannot be
unmapped while the thread that reads it is alive, whatever any assertion does.
The join result is captured and asserted after `processed`, so a genuine server
failure is still reported.

The server thread's own deadline goes from `2s` to `10s`, longer than the
client's `5s`, so the client decides the outcome. A shorter server deadline
would make that thread panic first under contention and report "server never saw
a request" when the real answer is "the host was busy".

## The Second Half, Found On CI

The first fix stopped the segfault — CI now failed the assertion cleanly, which
proved the join reordering worked — but the test still failed. Two more defects
sat underneath, and one of them was introduced by that first fix.

**Serve-once against a retrying client.** The fake child served exactly one
request and returned. Every client retry issues a *new* request sequence, so a
server answering request `N` while the client has moved to `N+1` leaves a stale
response and then exits. No later request can ever be answered and the client
spins to its deadline. The original 200-retry loop had this race too; its window
was just short enough to usually win. The server now serves until the client
signals it is done, and the count of requests served is asserted.

**A poll interval slower than the response window.** Replacing the hot spin with
`sleep(1ms)` on both sides looked like the fix for CI contention. It was wrong on
the server: `plugin_process_wait_budget` gives the client half a block, which at
`32` frames and `48 kHz` is `333us`. A server sleeping `1ms` polls three times
slower than the entire window it has to answer within, so it misses almost every
request. The server is back to `yield_now`, which stays inside the window while
still giving up the CPU voluntarily. The client keeps its sleep, because
`process` already spends its whole budget spinning and looping that hot is what
starves the server on a machine with few cores.

The deadline moved from `5s` to `30s` as well, but that was never the fix — it
only stopped a slow runner from being mistaken for a hang.

## The Retry Loop Was Futile By Construction

CI failed the same test again, at the full `30s`, against `30` clean local runs.
Retrying harder could never have worked, and the reason is in production code
rather than in the test.

`PLUGIN_PROCESS_CONSECUTIVE_TIMEOUT_LIMIT` is `3`. After three consecutive
misses `ShmPluginProcessor` clears its `alive` flag, and every later `process`
returns `false` immediately without waiting. So on CI the first three attempts
missed the wait budget, the epoch retired, and the remaining `30s` of retries
could not have succeeded however long they ran. The original `200`-iteration
loop had the same property; it just never reached three misses locally, where
the first attempt almost always wins.

The budget those three attempts had to hit is `min(1ms, half a block)` — `333us`
at `32` frames and `48 kHz`. A contended three-core runner can miss that three
times in a row through no fault of the protocol.

The test now re-attaches the processor when `is_alive()` goes false, giving the
round trip a fresh epoch, and reports how many epochs it needed. This is the
first fix in this sequence that addresses why retrying failed rather than how
long it retried for.

The same flaw exists in `signal-plugin-sandbox`'s `process_with_retries`, which
retried for `60s` against a handle with the identical retirement behaviour — so
two of the three failures in that binary were also unrecoverable rather than
slow. Serialising the child-spawning tests keeps the misses from stacking, and
the deadline is cut to `20s` so a retired epoch fails in tens of seconds instead
of a minute. The re-attach fix is not applied there: the twelve call sites each
hold their own handle and the lease needed to re-attach is not threaded through
them. That limit is now written at the constant.

## Measurement

- Before: `2` failures in `12` runs of `signal-plugin-bridge --lib`, plus the
  gate segfault.
- After the join fix alone: no more segfaults, but still failing — `2` in `10`
  locally and once on CI.
- After all three: `0` failures in `30` runs — but still failing on CI, because
  none of them addressed epoch retirement.
- After the re-attach: `0` failures in `6` runs under `2x` core oversubscription
  (`36` busy processes on `18` cores).

## Findings State

`A19` closed. `A22` closed earlier today. `A20` and `A21` are relocated to the
soak lane rather than fixed — their assertions are host-speed claims and are
only meaningful there. `A18`, the low-mid pops on ticks, remains open and needs
a direct transient probe.

## Next Task

Dispatch CI. If green, tag `v0.1.0`.
