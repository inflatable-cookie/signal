# 023 Generic MIDI, Note-Expression, And Plugin-Event Model Contract

Status: complete
Owner: core-product
Updated: 2026-03-15
Related contracts: `docs/contracts/008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`, `docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md`, `docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md`, `docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`, `docs/architecture/package-map.md`

## Purpose

Freeze the widened generic event contract for `g06.012` so later runtime and
adapter work can deepen MIDI, note-expression, and plugin-event behavior
without reopening packet ownership, CLAP-first assumptions, or adapter-private
event taxonomies as if they were already portable.

## Authority hierarchy

Generic plugin-event meaning has one authority chain:

1. `signal-plugin` owns the format-neutral event vocabulary for:
   - parameter value, modulation, and gesture events
   - note on/off identity and timing fields
   - bounded note-expression identity and value meaning
   - bounded three-byte MIDI event identity
   - packet and shared-memory encoding shape through `EventPacket`
2. `signal-runtime` owns runtime interpretation and delivery for:
   - block-local event timing and scheduling semantics
   - render, transport, automation, and recovery alignment
   - observation, supervisor, and future export/report surfaces
3. adapter crates such as `signal-plugin-clap`, `signal-plugin-vst3`, and
   `signal-plugin-au` own protocol-specific translation and realization detail:
   - event packet translation
   - backend-native event capability negotiation
   - format-specific event families not yet promoted into the shared contract
4. host crates may broker transport and supervisor delivery, but they must not
   become the authority for:
   - generic event meaning
   - portable versus adapter-private event claims
   - event fallback taxonomies that duplicate shared runtime or adapter
     contracts

If an event claim cannot be explained through `signal-plugin`,
`signal-runtime`, and additive Signal-owned receipts, it is not yet part of the
shared contract.

## Shared event vocabulary

This contract freezes the first bounded shared event vocabulary:

- `PluginEvent::ParameterValue`
- `PluginEvent::ParameterModulation`
- `PluginEvent::ParameterGesture`
- `PluginEvent::Note`
- `PluginEvent::NoteExpression`
- `PluginEvent::Midi`

The shared meaning attached to that vocabulary is:

- `offset_frames` is block-local timing intent owned by Signal, not an
  adapter-private scheduling hint
- parameter events remain parameter-id based and do not imply format-specific
  gesture or automation packet ownership
- note events keep one shared identity model:
  - `note_id`
  - `port_index`
  - `channel`
  - `key`
  - `velocity`
  - note `kind` as `NoteOn` or `NoteOff`
- note-expression events keep one bounded shared model:
  - `Pressure`
  - `Timbre`
  - `Tuning`
- MIDI events are intentionally bounded to three-byte message delivery through
  `status`, `data1`, and `data2`

Portable meaning here does not imply every adapter can yet realize every event
family with full fidelity. It means Signal now has one shared vocabulary that
later runtime and adapter work must target.

## Parity bands

This contract freezes four event-parity bands.

### Portable now

Portable event meaning currently includes:

- the six `PluginEvent` variants above
- block-local `offset_frames` timing semantics
- `EventPacket` as the bounded shared packet carrier
- shared-memory/event-region encoding owned by `signal-plugin`
- public note and note-expression identity fields where they already exist in
  the generic DTO family

### Portable with format guard

Guarded event meaning currently includes:

- translation from generic events into format-specific packets
- event families whose shared meaning exists, but whose end-to-end realization
  still depends on adapter support depth
- Linux or macOS availability where event-capable plugins depend on the
  adapter/platform combination already frozen in `g06.011`

Guarded means consumers may rely on one shared vocabulary, but must not assume
every adapter currently realizes the same depth.

### Adapter-private

Adapter-private event scope currently includes:

- CLAP-specific extension event families beyond the shared DTOs
- VST3 unit, program-list, controller-side editing, and richer note-expression
  or event-bus detail
- AU event-model, parameter-tree, and host-context detail that is not yet
  promoted into the shared DTO family
- packet-level translation decisions that do not yet change shared consumer
  meaning

### Unsupported or deferred

Unsupported or deferred event scope currently includes:

- SysEx, NRPN, or richer MIDI message families beyond the bounded three-byte
  `Midi` event
- MPE-specific policy or controller mapping surfaces
- MIDI editor, arranger, or product-local workflow semantics
- any claim of full CLAP, VST3, and AU parity for richer event-model depth

Unsupported or deferred scope must remain explicit in roadmap, contract, and
descriptor surfaces rather than being implied by adapter existence.

## Translation rule

This milestone freezes one translation rule:

- adapters must translate into and out of the shared `PluginEvent` vocabulary
  before any later public or runtime-owned event receipt claims become stable
- hosts and downstream consumers must not parse adapter-native packets to
  recover meaning that shared DTOs already define
- later runtime or export surfaces must build on `PluginEvent` and additive
  runtime-owned receipts rather than format-specific packet dumps

## Scheduling and transport rule

This milestone also freezes one runtime rule:

- event timing and transport semantics remain runtime-owned once events enter
  Signal execution
- adapter crates may realize packets, but must not redefine timing ownership
  that `offset_frames`, block-local execution, and future runtime event receipts
  already describe
- later event-depth work must stay aligned with existing block-timing,
  critical-path, deferred-work, and plugin-lifecycle contracts rather than
  creating a parallel event scheduler taxonomy

## `signal-midi` runway rule

`g06.012` does not require a new crate in Batch 12.1. It does require one
explicit rule:

- any later `signal-midi`-class crate or module must extend this shared
  vocabulary instead of inventing a second product-facing MIDI/event model
- reusable MIDI services must remain additive over `signal-plugin` and
  `signal-runtime`, not parallel to them

## Explicit deferred scope

Batch 12.1 intentionally does not claim:

- full runtime-owned event receipts or export surfaces yet
- full CLAP, VST3, and AU translation parity
- richer controller mapping, editor, preset, or performance-articulation depth
- product-local MIDI editing or control-surface workflow behavior

Those belong to later `g06.012` batches and follow-on milestones.

## Batch 12.1 outcome

Batch 12.1 freezes the widened generic event boundary:

- Signal now has one shared MIDI, note-expression, and plugin-event vocabulary
  instead of a CLAP-first implicit event model
- portable, guarded, adapter-private, and deferred event scope are now
  separated clearly enough for deeper runtime and adapter work
- later runtime and adapter event depth can widen on top of one fixed shared
  event model instead of reopening packet ownership and timing meaning

## Batch 12.2 outcome

Batch 12.2 binds the shared event vocabulary to real runtime-owned depth:

- `PluginProcessingContract` now carries explicit `supports_note_expression`
  capability instead of inferring note-expression depth from MIDI or note input
  support
- `RuntimePluginCapabilityCoverageSummary` and
  `RuntimePluginFormatCoverageRecord` now surface note-expression breadth as
  shared runtime-owned discovery truth
- `RuntimePluginEventSnapshot` now carries bounded generic event continuity,
  last-batch event mix, generated-event bytes, and lease rollover evidence on
  observation and supervisor surfaces
- local and server hosts now feed generic event summaries back into
  `signal-runtime` rather than keeping note, note-expression, and MIDI counts
  purely host-private once packet translation completes

Batch 12.2 still stops short of the public consumer proof. The widened event
surface is now real inside runtime and stable host-edge delivery, but Batch
12.3 is still required to prove downstream consumers can rely on it without
adapter-local reconstruction.

## Batch 12.3 outcome

Batch 12.3 closes the first generic-event consumer proof surface:

- downstream-style runtime tests now consume widened generic event continuity,
  note-expression capability breadth, and bounded MIDI output receipts through
  public `signal-runtime` reports
- both stable host edges now prove that `supervisor_report()` exports the same
  runtime-owned generic event truth without CLAP, VST3, or AU packet
  reconstruction
- `signal-supervisor-tools` now exposes the
  `signal.runtime.generic-event-boundary` descriptor and repo-owned
  `effigy acceptance:generic-event-boundary` task so consumers can inspect the
  proof surface without adapter crate internals
- later preset-state, portable recall, and ARA-context work can build on one
  closed generic event baseline instead of reopening shared event ownership

## Next Task

Continue `g06.013` with Batch 13.1 by freezing plugin preset-state
interchange, portable recall, and ARA-capable context vocabulary before
runtime recall/export depth begins.
