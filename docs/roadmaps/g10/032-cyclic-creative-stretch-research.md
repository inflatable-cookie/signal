# 032 - Cyclic Creative Stretch Research

Status: active; Batch 32.20 stopped pre-acoustic; Batch 32.21 ready
Owner: dsp
Updated: 2026-07-24
Contracts: `046`, `085`

## Problem

Signal has no Akai-style `Cyclic` renderer. The earlier program treated fixed
cyclic overlap and similarity-aligned overlap as one repair sequence. Original
Akai manuals and current systems show two distinct modes: fixed `CYCLIC` and
material-adaptive `INTELL`.

The first Signal candidate stopped on an absolute pitch threshold before
comparator listening. The second entered the `INTELL` family and failed its
frozen search reachability. Neither result establishes which source schedule
creates the desired ReaReaRea-like effect.

## Goal

Resolve the complete cyclic mechanism before another renderer is specified:

- source-progress clock
- cycle selection
- ratio accumulation
- local read and join law
- event placement
- linked-channel ownership
- exact boundaries
- honest user controls
- comparator-relative diagnostics and listening

Target expansion remains continuous above `1x` through `8x`, with mandatory
`2x`, `4x`, and `8x`.

## Non-Goals

- candidate DSP on `main`
- repair of `CyclicGrain` or `SimilarityAlignedCyclic`
- automatic routing
- Dream, Cloud, transparent, dynamic-ratio, cache, or product integration
- copied external source expression or constants
- a parameter sweep used to choose Signal implementation constants

## Batch 32.1 - Source Architecture Survey

Status: complete; later found non-executable

- [x] re-read both rejected Signal cyclic briefs and receipts
- [x] separate original Akai `CYCLIC` from `INTELL`
- [x] pin Potenza slow-anchor two-grain source
- [x] pin SickoCV repeat/jump cycle source
- [x] pin Sonic period insertion as specialist automatic-cycle evidence
- [x] retain ReaReaRea as the primary behavioral target
- [x] classify Akaizer and TAL-Sampler as optional proprietary behavior
- [x] reinterpret the old absolute pitch stop as unresolved target-relative
  evidence
- [x] publish one canonical source dossier

Authority:

- [Cyclic Time-Stretch Source Architecture](../../research/specimen-dossiers/cyclic-time-stretch-source-architecture.md)

## Batch 32.2 - Source-Faithful Executable Forensics

Status: complete

Use ignored `target/` and disposable external build state only.

- [x] build source-faithful probes for pinned Potenza and SickoCV schedules
- [x] build the pinned Sonic specialist contrast
- [x] capture ReaReaRea at `2x` for the five retained musical sources
- [x] capture ReaReaRea synthetic impulse, tone, chord, noise, and stereo rows
- [x] render `2x`, `4x`, and `8x` across short, medium, and long cycle regions
- [x] record exact source revisions, commands, request semantics, and output
  hashes
- [x] measure event replicas, cycle cadence, pitch delta, join width, energy,
  boundaries, and stereo relation
- [x] keep every artifact ignored and out of public or hidden Signal surfaces

Stop after the forensic receipt. Do not select a Signal renderer in this
batch.

## Batch 32.3 - Behavioral Synthesis And Gate Correction

Status: complete

- [x] distinguish compressed-anchor and repeat/jump behavior
- [x] decide whether fixed cycle length is sufficient
- [x] classify automatic cycle selection as Cyclic assistance, `INTELL`, or
  unsupported
- [x] freeze the minimum useful UI vocabulary
- [x] replace arbitrary absolute quality thresholds with hard integrity and
  comparator-relative diagnostics
- [x] record remaining uncertainty explicitly

This batch is docs only. If evidence cannot distinguish a complete schedule,
close or extend research. Do not choose by convenience.

Decision:

- centred compressed-anchor Cyclic behavior selected
- raw whole-cycle repeat/jump rejected as the primary target
- fixed manual cycle sufficient for the first character
- optional automatic cycle classified as later specialist assistance
- `duration`, `character=Cyclic`, and `cycle` frozen as the minimum UI
- integrity remains hard; finite character metrics diagnose; listening
  promotes

Authority:

- [Offline Creative Cyclic Behavioral Synthesis](../../architecture/offline-creative-cyclic-behavioral-synthesis.md)

## Batch 32.4 - Complete Renderer Brief

Status: complete

Freeze one buildable renderer only if the source and behavioral evidence
select it. The brief must jointly own:

- [x] exact source/output map and cycle grammar
- [x] continuous fixed ratios and exact target length
- [x] cycle control and any automatic guidance
- [x] local read, interpolation, and join law
- [x] event placement and commanded-replica definition
- [x] linked stereo
- [x] boundaries, memory, determinism, and cost
- [x] structural, synthetic, comparator, and listening order
- [x] rejection, cleanup, and minimal admission

No implementation belongs in this batch.

Authority:

- [Offline Creative CenteredCompressedAnchorCyclic Renderer Brief](../../architecture/offline-creative-centered-compressed-anchor-cyclic-brief.md)

## Batch 32.5 - Isolated Candidate And Conformance

Status: complete; later evidence-invalid

- [x] create only the frozen disposable worktree, branch, private module, ledger,
  runner config, ignored evidence root, and local ref namespace
- [x] prepare and bind the exact `63`-row comparator manifest before candidate
  acoustic work
- [x] implement the frozen renderer and all `15` evidence owners
- [x] run nominal compile, construction `1/1`, and structural `9/9` twice from
  one clean tree
- [x] require byte-identical per-owner structural receipts across both rounds
- [x] create the immutable acoustic ref only after complete two-round
  conformance
- [x] stop before `Y01` or any candidate listening render

Checkpoint:

- commit `4600d228286797d22e4f4d5ca4efa997835fc4b2`
- tree `fa1fc8031a4aab4302b778474702e658784d8a64`
- ref
  `refs/signal-evidence/creative/centered-compressed-anchor-cyclic/32-5-acoustic`
- comparator manifest
  `eb5384681767dfd36e8daf81809a95d51a79f6cb178f0705fe4cffce9ecccacd`

No candidate source enters `main`.

## Batch 32.6 - Acoustic And Listening Admission

Status: stopped; evidence-invalid

Run `Y01..Y06`, exact `16x` rejection, concealed mono, cycle-direction review,
long-form stereo objectives, speaker pre-screen, and eligible independent
stereo review from the one immutable ref. Stop on first failure.

No code, source, formula, constant, metric, threshold, comparator, or helper
change is allowed after the ref.

Actual result:

- [x] run `Y01` once from checkpoint `4600d228`
- [x] stop on `unexpected dropout 1`
- [x] do not run `Y02..Y06`, exact `16x`, or listening
- [x] record that no `Y01` receipt or receipt directory exists
- [x] classify the result as incomplete executable evidence, not an acoustic
  pass or rejection

The owner accumulated rows in memory and called its receipt writer only after
complete success. Its early error therefore discarded the failed row, prior
rows, hashes, and summary.

## Batch 32.7 - Evidence-Integrity Reassessment

Status: complete

- [x] inspect the immutable source without rerunning DSP
- [x] prove that receipt status is hardcoded to `pass`
- [x] prove that row persistence occurs only after complete owner success
- [x] prove that input/comparator receipt hashes and per-row assertions are not
  owned
- [x] record missing `Y04`, `Y05`, and `Y06` diagnostics
- [x] record missing five-source exact-`16x` and listening executors
- [x] retain the surfaced dropout only as unreceipted failure output
- [x] authorize one fresh audited identity under Contract `085` Rule 11
- [x] delete the rejected worktree, branch, build state, generated copies, and
  local evidence ref after this docs closeout

This is the first incomplete-evidence checkpoint for the centred
compressed-anchor identity. No valid quality or listening decision exists.
The renderer formula may not change from the unreceipted output.

## Batch 32.8 - Fresh Audited Authority

Status: complete

Freeze one docs-only `AuditedCenteredCompressedAnchorCyclic` brief from the
canonical architecture. It must make each structural and acoustic row its own
enforced execution and receipt boundary, bind failed-row persistence and every
receipt field, and own exact `16x`, comparator preparation, concealment, level
matching, listening output, and cleanup before implementation.

No candidate source, output, harness, comparator recapture, or implementation
belongs in this batch.

- [x] retain the complete renderer without using unreceipted `Y01` output
- [x] freeze fresh worktree, branch, module, evidence root, runner, and ref
- [x] make every structural, acoustic, exact-`16x`, and listening row a
  separate one-shot process
- [x] require durable `started` plus `pass`, `fail`, or `panic` receipts
- [x] bind timeout, kill, retry, duplicate-file, schema, assertion, diagnostic,
  source, comparator, output, and artifact behavior
- [x] freeze construction sentinels that execute the real failure and panic
  receipt paths
- [x] freeze all `339` structural and `183` synthetic row expansions
- [x] complete `Y04`, `Y05`, and `Y06` diagnostic algorithms
- [x] freeze the five-source exact-`16x` executor
- [x] freeze comparator recapture, level matching, terminal fade, concealment,
  mono decisions, stereo decisions, cleanup, and minimal admission

Authority:

- [Offline Creative EventLedgerAuditedCenteredCompressedAnchorCyclic Brief](../../architecture/offline-creative-event-ledger-audited-centered-compressed-anchor-cyclic-brief.md)

Batch 32.9 proved that the checked prose coverage did not freeze the exact
machine-readable identities and bytes required by Rule 11. The authority is
not implementation-ready until Batch 32.10 closes those gaps.

## Batch 32.9 - Fresh Isolated Candidate And Conformance

Status: stopped before isolation or source

Start from the exact Batch 32.8 closeout commit. Create only the frozen fresh
identity. Prepare and bind the comparator manifest, implement the unchanged
renderer and one-shot row protocol, then run:

1. release compile without test execution
2. construction `1/1`
3. all `339` structural rows as separate processes
4. a second unchanged compile, construction, and structural round
5. byte-identical receipt and summary comparison
6. immutable acoustic ref creation

Stop before `Y01`, exact `16x`, long-form render, or listening execution. Any
missing authority, evidence edge, or choice stops for docs-level reassessment.
Do not recover rejected source.

Actual result:

- [x] confirm clean `main` at Batch 32.8 closeout
  `aaa0a7913fd41020c73788d87f9e8cf41bfa197d`
- [x] confirm fresh worktree, branch, build root, and evidence ref are absent
- [x] inspect Effigy test shape and the private crate integration surface
- [x] stop before creating the worktree, recapturing comparators, or writing
  candidate source
- [x] record that the brief does not freeze the comparator manifest bytes,
  exact comparator row set, or project/source generator schema
- [x] record that the brief does not freeze exact expanded row IDs,
  per-row assertion/diagnostic IDs, summary schema, runner environment, or
  sentinel protocol bytes
- [x] do not recover those values from the deleted checkpoint

The renderer remains selected. The executable evidence authority is not
implementation-ready. This is a pre-source docs gap, not a second checkpoint
and not a renderer rejection.

## Batch 32.10 - Executable Manifest Reconciliation

Status: complete

Freeze the missing authority in the same canonical brief before another
isolation attempt:

- exact deterministic row-ID grammar and ordered expansion for every
  structural, synthetic, exact-`16x`, long-form, and listening row
- exact assertion and diagnostic IDs, applicability, expected values, units,
  and terminal `not_run` representation
- exact row-manifest and summary schemas, canonical bytes, and hash rules
- exact runner environment names, process invocation, receipt-root grammar,
  stop behavior, and child-sentinel handshake
- exact `63` comparator rows, source formulas, project generator, TSV schema,
  ordering, and fresh recapture receipt
- explicit replacement of the unreachable deleted-checkpoint manifest hash
  without using deleted source
- exact long-form pack, concealment-key, decision, reveal, and stereo-review
  manifest schemas
- construction bindings that prove the concrete executors rather than prose

Docs only. Do not create a candidate worktree, recover rejected state,
recapture comparators, or write implementation.

Result:

- [x] freeze canonical TSV/JSON encoding and exact manifest headers
- [x] freeze all `588` executable rows and deterministic row-ID expansion
- [x] freeze per-row assertion and diagnostic identities, expected-value
  vocabulary, and units
- [x] freeze summary schemas and receipt aggregation hashes
- [x] freeze Effigy invocation, environment, receipt roots, timeout, stop, and
  no-retry behavior
- [x] freeze real child-process failure, panic, kill, ACK, and duplicate-file
  sentinels
- [x] replace the unreachable comparator-manifest hash with one reproducible
  `63`-row native-stereo/mono/stereo-synthetic set
- [x] retire old mono comparator hashes to historical provenance only
- [x] freeze normalized REAPER project semantics and source-container bytes
- [x] freeze listening pack, key, decision, pre-screen, reveal, and listener
  identity schemas

The renderer is unchanged. Deleted state was not inspected or recovered.

## Batch 32.11 - Fresh Isolated Candidate And Conformance

Status: complete

Start from the exact Batch 32.10 closeout commit:

1. create only the frozen fresh worktree, branch, evidence root, module, runner,
   profile, and ref namespace
2. implement evidence/source/project generators before candidate DSP
3. recapture and bind the exact new `63`-row comparator manifest
4. implement the unchanged renderer and every compile-linked owner
5. commit one clean candidate checkpoint
6. run release compile, construction `1/1`, and all `339` structural rows
7. repeat unchanged and require byte-identical receipts and summaries
8. create the immutable acoustic ref

Stop before `Y01`, exact `16x`, long-form rendering, or listening. Any
remaining manifest choice stops again before checkpoint creation.

Result:

- [x] generated the `588`-row evidence manifest before candidate source
- [x] captured and bound all `63` comparator rows
- [x] implemented the unchanged private renderer and every compile-linked
  evidence owner
- [x] passed release compile and construction `1/1`
- [x] passed all `339` structural rows and `168` planned renders twice
- [x] obtained byte-identical receipts and summaries across both rounds
- [x] froze clean checkpoint `74a6d6d9` at evidence ref
  `refs/signal-evidence/creative/audited-centered-compressed-anchor-cyclic/32-11-acoustic`
- [x] stopped before every acoustic, exact-`16x`, long-form, and listening row

The structural summary hash is
`75a2e5ed5c1406d9790a5ba904d7ce8d8e5c4dc459787b8e0dac6a1d761b43c2`.
The candidate remains isolated; no DSP or evidence harness entered `main`.

## Batch 32.12 - First Acoustic Gate

Status: stopped; evidence invalid

Run only the `30` frozen `Y01` rows from the Batch 32.11 acoustic ref, in
manifest order:

1. verify the ref, checkpoint, clean tree, manifest hashes, comparator hashes,
   and empty fresh evidence root
2. execute each one-shot row with fail-durable receipts and no retry
3. stop on the first terminal failure or incomplete receipt
4. write the `Y01` summary only when all `30` rows pass
5. close the batch in docs without changing the candidate checkpoint

Stop after `Y01`. Do not execute `Y02` through `Y06`, exact `16x`, long-form
rendering, listening, product routing, or public admission in this batch.

Result:

- [x] verified checkpoint `74a6d6d9`, tree `d519e2d8`, all manifest hashes,
  clean worktree, and `30` planned `Y01` rows
- [x] invoked `Y01` once and stopped on row
  `Y01-000-low-tone-r2-c048000`
- [x] recorded runner exit `66`: the intended receipt was missing
- [x] found one two-line passing receipt under a crate-relative duplicate root
- [x] confirmed the intended root contains only environment identity files
- [x] confirmed no `Y01` summary or later acoustic row exists
- [x] did not retry

The ignored root was supplied as the frozen repo-relative
`target/creative-stretch-audited-centered-compressed-anchor-cyclic-32-11`.
The shell runner resolved it from the repository root. Nextest resolved the
same environment value from `crates/signal-dsp-stretch`, so the row receipt
landed under that crate instead. The runner then failed its in-root receipt
check.

The misplaced receipt hash is
`f9c12e26ca6d7e727749ae12e70e86262816715abad66850396ea6fdc4596d91`.
Its passing assertions are out-of-root and do not admit the row. This is an
evidence-path ownership failure, not an acoustic quality result.

## Batch 32.13 - Second Evidence Failure Closure

Status: complete

Docs only:

1. classify the split-root `Y01` stop under Contract `085` Rule 11
2. record that the fresh audited identity is the second incomplete-evidence
   checkpoint for centred compressed-anchor Cyclic
3. close that identity without acoustic, long-form, stereo, or listening claim
4. delete the retained acoustic ref after the closure commit
5. leave admitted `Dream`, transparent stretch, routing, cache, Loophole, and
   Chorus unchanged

Do not repair the runner, retry `Y01`, authorize a third identity, recover
isolated state, implement candidate DSP, or start another Cyclic mechanism.

Result:

- [x] classified Batch 32.12 as the second incomplete-evidence checkpoint
  under Contract `085` Rule 11
- [x] closed centred compressed-anchor Cyclic without an acoustic pass or
  rejection
- [x] confirmed no `Y02..Y06`, exact-`16x`, long-form, stereo, or listening
  evidence exists
- [x] retained no worktree, branch, build state, generated evidence, candidate
  source, or harness on `main`
- [x] rejected runner repair, `Y01` retry, and a third audited identity
- [x] scheduled deletion of the local acoustic ref after this closure commit

`g10.032` is complete. Signal still has no admitted `Cyclic` renderer. Exact
fixed `4x`, `8x`, and `16x` `Dream` remains the only public creative stretch
surface.

## Batch 32.14 - Operator Correction And Exact Replay Authority

Status: complete

The operator rejects Batch 32.13 closure. The renderer did not fail: the first
row passed every assertion. The caller supplied a relative evidence root, and
nextest resolved it from a different working directory than the shell runner.

This is a docs-only authority correction:

- restore exact checkpoint `74a6d6d9`, tree `d519e2d8`
- change no candidate, test, runner, manifest, comparator, or dependency byte
- recreate the same acoustic ref and isolated worktree identity
- use the exact absolute ignored root frozen in Contract `085`
- execute the complete `Y01` gate once
- stop on the first valid failure or after the `Y01` summary

The replay is the same checkpoint, not a third candidate. The invalid
relative-root run cannot select a change. Do not begin execution in this batch.

## Batch 32.15 - Absolute-Root Y01 Replay

Status: complete; stopped before DSP

1. verify commit `74a6d6d9`, tree `d519e2d8`, all manifest hashes, and an
   absent absolute evidence root
2. restore the exact worktree, branch, and acoustic ref
3. invoke the unchanged runner with the frozen absolute root
4. run all `30` `Y01` rows once, in manifest order
5. stop on the first valid terminal failure or after the `Y01` summary
6. close the result without changing the checkpoint

Do not execute `Y02..Y06`, exact `16x`, long-form, stereo, listening, product,
or routing work.

Result:

- exact checkpoint, tree, ref, worktree, manifests, and absolute root passed
  preflight
- the first row wrote a two-line terminal receipt with every assertion
  `not_run`
- source loading stopped on missing canonical
  `comparator/sources/low-tone.wav`
- no renderer invocation, candidate render, acoustic assertion, or summary
  occurred

Batch 32.13 cleanup removed the ignored comparator assets as well as rejected
candidate state. The recovery authority failed to restore that prerequisite.
This is an evidence-environment stop, not a valid `Y01` failure.

## Batch 32.16 - Synthetic Comparator Restoration And Y01 Replay

Status: complete; checkpoint rejected

1. preserve the complete Batch 32.15 execution directory as
   `Y01-invalid-missing-comparator-assets-32-15` without changing its bytes
2. regenerate the exact `16` synthetic sources at the checkpoint's canonical
   comparator root
3. render only the `30` frozen `C-Y-*` ReaReaRea comparator rows required by
   `Y01`
4. verify source, project semantics, project container, output container, and
   output PCM hashes against the frozen comparator manifest
5. verify checkpoint, tree, ref, tracked hashes, and clean worktree again
6. invoke the unchanged runner once at the same absolute root
7. stop on the first valid terminal failure or after all `30` rows and the
   `Y01` summary

Do not change candidate, test, runner, manifest, comparator, metric, threshold,
dependency, or DSP bytes. Do not execute `Y02..Y06`, exact `16x`, long-form,
stereo, listening, product, or routing work.

Result:

- all `30` Y01 comparator source, project, container, and PCM identities
  matched the frozen manifest
- REAPER's wall-clock BWF field was restored to the unique original value
  selected by each frozen container hash; PCM was already exact
- rows `Y01-000` through `Y01-011` passed
- `Y01-012-impulse-r2-c048000` failed with `unexpected dropout 1`
- no Y01 summary or later gate exists
- candidate, test, runner, manifest, comparator, metric, threshold, dependency,
  and DSP bytes remained unchanged

This is a valid acoustic rejection of checkpoint `74a6d6d9`. It is not closure
of the operator's Cyclic target.

## Batch 32.17 - Impulse Dropout Architecture Reassessment

Status: complete

1. preserve the checkpoint ref through reassessment and retain the receipt
   hashes in the closeout record
2. trace the complete ownership path that maps an active impulse source window
   to one output window below `-80 dBFS`
3. compare that cause with the retained Cyclic source architectures and prior
   rejected families
4. reject local parameter, cycle, window, gain, threshold, schedule, or
   impulse-only repair
5. freeze one materially different complete renderer only if it jointly owns
   scheduling, boundaries, replica prevention, transient energy, stereo,
   determinism, and bounded state
6. otherwise record an evidence-backed architectural stop without claiming
   the Cyclic product target is unimportant

This batch is docs and static source analysis only. Do not implement a
candidate, change evidence, or run acoustic rows.

Result:

- the failing `[88179,88400)` output window maps across the authored impulse
  but sits exactly between commanded replica groups
- the event remains present in four positive ledger groups; it was not lost
- the `1058.5`-frame spacing matches retained ReaReaRea replica evidence
- continuous mapped-window activity is the wrong hard owner for sparse Cyclic
  events
- select fresh `EventLedgerAuditedCenteredCompressedAnchorCyclic` with
  unchanged DSP, unchanged sustained-source dropout control, and full ledger
  ownership for sparse events
- static audit found placeholder Y02, Y03, and Y04 diagnostic owners in the
  rejected checkpoint, so no implementation or harness is reusable

## Batch 32.18 - Event-Ledger Evidence Authority

Status: complete

Freeze one complete canonical brief before isolation:

- [x] retain every centred compressed-anchor renderer, map, linked-channel,
   boundary, memory, and determinism formula unchanged
- [x] replace sparse Y01 dropout ownership with exact event-ledger assertions and
   measured diagnostics
- [x] retain the continuous-source `221`-frame, `-40/-80 dBFS` dropout rule
   unchanged
- [x] freeze exact row IDs, assertion IDs, diagnostic IDs, counts, order,
   summaries, and receipt schemas
- [x] freeze executable known answers that prove the real FFT, ledger, cadence,
   gap, stereo, level, and comparator owners rather than labels or
   placeholders
- [x] freeze fresh isolation names, cleanup, and evidence order

Do not create a worktree, candidate source, harness, comparator render, or
acoustic receipt in this batch.

Result:

- one canonical
  [EventLedgerAuditedCenteredCompressedAnchorCyclic brief](../../architecture/offline-creative-event-ledger-audited-centered-compressed-anchor-cyclic-brief.md)
  replaces the superseded audited brief
- renderer, map, cycle, linked-channel, boundary, memory, determinism, and
  exact-length formulas are unchanged
- Y01 row IDs `012..017` replace continuous dropout with full ledger
  assertions and measured event diagnostics; the other `24` rows are
  unchanged
- the complete manifest remains `588` rows with `339` structural and `183`
  synthetic rows
- construction now executes exact known answers through the real FFT, ledger,
  cadence, gap/dropout/tail, stereo, level/fade, and comparator owners
- no candidate or evidence implementation entered `main`

## Batch 32.19 - Fresh Isolated Implementation And Conformance

Status: complete

1. create only the frozen worktree, branch, private module, evidence authority,
   ignored root, nextest profile, and local ref namespace
2. generate and bind the exact `588`-row manifest, `63`-row comparator set,
   and listening manifest before candidate source exists
3. implement the unchanged renderer and every evidence owner from the Batch
   32.18 brief without recovering rejected code or harness state
4. run release compile and construction once; require `1/1`
5. run all `339` structural rows in manifest order, then repeat compile,
   construction, and the complete structural round unchanged
6. require byte-identical corresponding receipts and summaries before
   creating the acoustic ref
7. stop before `Y01`, exact `16x`, long-form render, or listening

Any choice missing from the brief, known-answer failure, manifest mismatch,
compile failure, construction failure, structural failure, incomplete
receipt, or non-identical round stops for docs-level reassessment. Do not
repair or rerun an acoustic row in this batch.

Pre-source audit result:

- the first exact isolation remained clean
- no manifest, comparator project/render, candidate source, receipt, or DSP
  output existed
- audit found unresolved ramp endpoints, numeric evidence primitives, band and
  correlation aggregation, summary assertion IDs, and known-answer vectors
- the canonical brief now freezes those choices together
- delete the empty isolation and recreate the same name from the correction
  commit before evidence generation

Result:

- fresh evidence commit `6a909c74` bound all `588` rows, `63` comparator
  rows, `51` listening rows, and regenerated comparator audio before candidate
  source
- private candidate implementation commit `08e5c57c` added the exact
  two-read centred compressed-anchor renderer and compile-linked evidence
  owners without changing a public or product surface
- checkpoint `08e5c57c` stopped before construction because the mandated
  `/usr/bin/python3` rejected Python 3.11-only `zip(strict=True)` during
  manifest parsing; zero row receipts and zero candidate renders exist
- evidence-only compatibility commit `995ea516` replaced that call with an
  explicit field-count check; renderer and test logic were unchanged
- checkpoint `995ea516`, tree `fd42543b`, passed release compile,
  construction `1/1`, and structural `339/339` twice with `168/168` renders
- both complete receipt trees and summaries are byte-identical; structural
  summary SHA-256 is
  `f1e90cd36557d1c1b6ef3be70175b7f025cdd00f1d7405a7a7958cf8a91cb08b`
- local acoustic ref
  `refs/signal-evidence/creative/event-ledger-audited-centered-compressed-anchor-cyclic/32-19-acoustic`
  points to `995ea516`
- no `Y01`, exact `16x`, long-form, or listening row ran
- candidate source, evidence code, comparator audio, receipts, and artifacts
  remain isolated from `main`

## Batch 32.20 - Event-Ledger Y01 Admission

Status: complete; stopped before row execution

1. resolve the Batch 32.19 acoustic ref to exact checkpoint `995ea516` and
   tree `fd42543b`; require a clean unchanged candidate worktree
2. run only the `30` frozen `Y01` rows, once, in manifest order
3. stop on the first terminal failure, panic, timeout, missing receipt, or
   incomplete receipt
4. if all rows pass, write and bind the `Y01` summary
5. stop before `Y02`, exact `16x`, long-form rendering, or listening

No renderer, evidence owner, manifest, comparator asset, threshold, or runner
change is authorized. A valid Y01 failure rejects this checkpoint. An
evidence-environment stop returns to docs-level classification without
rerunning a completed row.

Result:

- acoustic ref, checkpoint, tree, and clean worktree matched exactly
- the only tracked runner accepts `conformance-round` with
  `conformance-round-1` or `conformance-round-2`; it has no acoustic execution
  ID or Y01 row-selection path
- the compile-linked summary owner ignores the frozen summary scope, selects
  only `stage == "conformance"`, and can create only
  `summary/structural.json`
- zero Y01 receipts, summaries, or candidate renders exist
- no runner invocation or acoustic row occurred
- checkpoint `995ea516` is incomplete executable evidence, not an acoustic
  pass or rejection
- keep the immutable ref and isolated state unchanged for docs-only
  reassessment

## Batch 32.21 - Event-Ledger Evidence-Integrity Reassessment

Status: ready; docs only

1. audit every frozen post-checkpoint runner transition, row selector, summary
   scope, receipt boundary, decision owner, reveal owner, and stop condition
   against the exact checkpoint
2. enumerate every missing or non-executable boundary, not only Y01
3. classify the checkpoint under Contract `085` Rule 11, including whether the
   bounded fresh-audited-identity exception remains available
4. freeze either one complete corrective authority or explicit family closure
5. leave checkpoint, ref, candidate source, evidence, comparator assets, and
   ignored receipts unchanged

Do not run a synthetic, exact-`16x`, long-form, or listening row. Do not patch
the frozen candidate or infer a direct command around the tracked runner.

## Completion Gate

- source families are pinned and clean-room boundaries recorded
- behavioral forensics distinguish or explicitly fail to distinguish schedules
- operator listening owns musical character
- integrity metrics reject clicks, invalid duration, broken stereo, and
  uncommanded replicas
- one complete renderer and executable-evidence brief exists before
  implementation
- rejected candidate code and evidence scaffolding stay out of `main`

## Next Task

Execute Batch 32.21 only. Audit the complete post-checkpoint evidence path and
make the Rule 11 eligibility decision without changing or running the
candidate.
