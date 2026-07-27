# 2026-07-27 g10.038 Batch 38.1 Surface Inventory And Retention Decisions

Status: complete

Documentation only. No crate source changed.

## Count Correction

The 2026-07-27 audit reported `249` public items. That count grepped every
`pub` declaration across the crate's source, including module-internal items
that are never exported. The exported surface is `160`.

The gap the finding described is real and larger in proportion than stated:
`36` of `160` have a consumer outside the crate.

## Inventory

| family | external consumer | own integration tests only | no consumer | total |
| --- | --- | --- | --- | --- |
| evidence | `15` | `0` | `72` | `87` |
| core engine | `6` | `2` | `12` | `20` |
| realtime preview | `0` | `2` | `15` | `17` |
| creative | `0` | `0` | `14` | `14` |
| cache identity | `7` | `0` | `4` | `11` |
| artifact plan | `4` | `0` | `2` | `6` |
| promotion | `4` | `0` | `1` | `5` |
| **total** | **36** | **4** | **120** | **160** |

"External consumer" means `signal-render-plane`, `signal-runtime`,
`signal-dsp-stretch-evidence`, `signal-host-local`, `signal-plugin`, `demos`,
or the workspace `tests` directory. Items used only by the crate's own unit
tests count as having no consumer, because an item used only from inside the
crate does not need to be `pub`.

## Retention Decisions

**Evidence, `87` items, `72` unused.** The reduction target. The evidence
surface exists so the corpus binary and promotion gates can measure renders,
and it grew one public entry point per measurement variant. Batches 38.2 and
38.3 collapse it: one entry point per measurement with an explicit policy
argument, one shared windowing and STFT surface, and `pub` only where the
evidence crate actually calls in.

**Realtime preview, `17` items, `0` external consumers.** Deferred to
`g10.040` in full. This lane does not touch it beyond the one tautological test
helper already removed in `g10.036`. The six never-constructed variants
recorded by the audit stay in place as `g10.040` inputs, because that lane
decides whether the tier is completed or closed, and closure is what removes
them.

**Creative, `14` items, `0` in-repo consumers.** Retained. This is the admitted
public product surface under Contract `085`, and its intended consumer is
outside this repository. Absence of an in-repo caller is expected, not dead
surface. Contract `085` already froze the shape.

**Cache identity, `11` items, `7` external.** Retained in full. `g10.037` made
these the contract surface; the four without a consumer are the schema, engine,
and behavior version constants plus `StretchRenderGeometry`, all of which a
cache consumer needs to read.

**Core engine, `20` items, `12` unused.** Mostly retained: geometry constants,
selector gate constants, tier vocabulary, and `StretchBackendPlan` are Contract
`046` vocabulary that consumers gate behavior on. `PhaseVocoderStretcher` is
the draft baseline every benchmark comparison renders against, so it stays
public for the evidence crate.

**Artifact plan and promotion, `11` items, `3` unused.** Retained. Both are
render-plane and runtime contract surfaces.

## Byte-Exactness Frozen As The Acceptance Proof

Every later batch in this lane is a refactor, not a behavior change. Byte-exact
output against the `g10.036` baselines is the acceptance proof for each of
them, and a batch that cannot prove it stops rather than re-baselines. Contract
`084` Rule 10 governs re-freezing a hash, and nothing in `g10.038` is
authorized to invoke it.

## Process-Global Test State Sweep

Two `A17`-class defects have now been found in two modules, so the sweep was
run rather than assumed.

The pattern is only unsafe when process-global test state is shared by more
than one test in the same binary. Findings:

| location | shape | risk |
| --- | --- | --- |
| `creative_direct_renewal_dream/tests.rs` | global allocator counters | fixed in `g10.036` Batch 36.2 |
| `creative_cyclic/synthesis.rs` | global allocation counter | fixed in `g10.037` Batch 37.4 |
| `signal-dsp-stretch/tests/realtime_preview_callback_alloc.rs` | global allocator counters | safe: one test in the binary |
| `signal-render-plane/tests/live_render_soak.rs` | global allocator counters | safe: one test in the binary |
| `signal-hardware/tests/capture_alloc.rs` | global allocator counters | safe: one test in the binary |
| `stretch-corpus-report/alloc_tracker.rs` | global counters plus a lock | safe: single-threaded binary |
| `signal-ipc/src/shared_memory.rs` | `PROCESS_WIDE_REGION_SEQUENCE` | production code, intentionally process-wide |

The three integration-test allocators are safe by construction today and
fragile by design: adding a second `#[test]` to any of those files breaks the
measurement silently. Batch 38.2 converts them to thread-local state, which
costs nothing and removes the trap.

**This sweep does not explain `A19`.** The `signal-plugin-bridge` shared-memory
failure has no process-global test counter behind it.
`PROCESS_WIDE_REGION_SEQUENCE` is production code doing exactly what its name
says. `A19` remains untriaged and needs a reproduction under parallel workspace
load; a resource or name collision is the more likely lead.

## Validation Run

- surface inventory generated from the crate's export lists and grepped against
  every consumer path
- process-global test state sweep across the workspace
- `effigy qa:docs`

## Next Task

Execute `g10.038` Batch 38.2: reduce the promotion policy to one owner, remove
the `cfg(test)`-only `creative_cyclic` render and identity paths, convert the
three integration-test allocator counters to thread-local state, and apply the
Batch 38.1 removals. Byte-exact output is the acceptance proof.
