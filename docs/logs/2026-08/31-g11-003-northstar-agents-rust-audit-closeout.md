# g11.003 Northstar AGENTS And Rust Audit Closeout

Status: complete; one review wave applied after orchestrator review
Date: 2026-08-31
Updated: 2026-09-01
Owner: core-product
Card: `docs/roadmaps/g11/batch-cards/008-g11-003-northstar-agents-rust-audit.md`
Milestone: `docs/roadmaps/g11/003-northstar-instruction-and-rust-quality-audit.md`

## Summary

One repository-scope Northstar Rust explicit audit over all 28 crates, plus a
reader-journey review of `AGENTS.md` and its `CLAUDE.md` bridge. The audit found
97 findings, repaired the 89 the recorder authorized, and left the 8 unsafe
findings report-only. 82 files changed: 28 Cargo manifests, 53 Rust files, and
`AGENTS.md`.

The headline result is that Signal's Rust is in good shape and the gaps were
specific rather than systemic. Zero production `unwrap`, zero
`missing_safety_doc`, a warning-free enforced clippy run, and zero exact
pass-through wrappers across 826 files. The three real gaps were a declared MSRV
no member inherited, 47 public types without `Debug`, and 13 panic-capable
public operations whose invariant was not visible.

## Rust Audit

- Audit ID: `signal-g11-003-repository-audit`
- Scope: `repository`, 14 units, 857 owned files, 0 exclusions
- Status: `degraded` (retained report-only findings, recorded reading depth,
  one unavailable and one warning-bearing evidence record)
- Discovery: `63634165119c5ff4600a936e1796c71fa2938ed2d553e7be8459a6476ec308e3`
- Scope plan: `a99e83df27aaa8676ba42c0c2e9fb0176c0e81f2bab441c682be9b9fecbbe48d`
- Projection (`strict-audit.json`):
  `c2ccfbe0d28c7a938abe33f6455b3788d9f65442a3e8fb9a58cfd545f4539a2c`
- Catalogue (`catalogue.json`):
  `0640a7cea66bdc0c64f460f8a47e9dd4f8b912b56218328c27717a32fa95e316`
- Profile: `53c54482d38d77cb3cf5b548de1b1b851455de13db541fbd7042245ebe31e424`
- Deviations: `d6d876aeb6e70da9fec368201350b6d16f345a7363309dde4169284c51c2fcd0`
  (empty; no deviation was claimed)
- `report.md`: `417f40d38e42e468f406ce26268da266cdf9fbd2c2b58a32c99cb32dce55d26e`
- `result.json`: `bf7dc353a760a2dead465150e2fbb20b9c30c4aaa32df6cf5c6dd7236f84f706`

Records live in this worktree's Git metadata under
`.git/worktrees/northstar-agents-rust-audit/northstar/rust-quality/audits/signal-g11-003-repository-audit/`.

These hashes seal the **first wave only**. The review wave below changed one
file that the sealed result already lists among its 81, so for
`crates/signal-plugin-au/src/au_host_adapter/hosting/process.rs` the recorder's
`plugin-formats` evidence describes the first-wave content, not the current
head. The sealed result was not regenerated, rewritten, or re-finalized, and
`result.json` still hashes to
`bf7dc353a760a2dead465150e2fbb20b9c30c4aaa32df6cf5c6dd7236f84f706`. The
recorder's changed-file set is unchanged; only that one file's content moved
past it.

### Units

`workspace-manifest`, `primitives`, `dsp-core`, `dsp-stretch`, `analysis`,
`graph`, `runtime`, `ipc`, `plugin-core`, `plugin-formats`, `plugin-sandbox`,
`hardware`, `render-plane`, `host-local`. Every one of the 28 packages, every
discovered target source path, all three declared features
(`signal-analysis:test-support`, `signal-dsp-stretch:default`,
`signal-dsp-stretch:evidence`), and the workspace build files have an owned
disposition. Nothing is excluded.

### Verdicts

Six normative rules × 14 units = 84 verdicts. `RUST-SLOP-001` is prototype and
evaluation-only, so it carries no verdict; its ledger is below.

| Rule | pass | finding | not_applicable | degraded |
| --- | --- | --- | --- | --- |
| `RUST-READ-001` | 1 | 0 | 1 | 12 |
| `RUST-API-001` | 3 | 9 | 2 | 0 |
| `RUST-ERR-001` | 8 | 5 | 1 | 0 |
| `RUST-UNSAFE-001` | 2 | 8 | 4 | 0 |
| `RUST-ASYNC-001` | 0 | 0 | 14 | 0 |
| `RUST-MSRV-001` | 1 | 13 | 0 | 0 |

### Findings and repairs

**`RUST-MSRV-001` — 28 findings, all repaired.** `[workspace.package]` declares
`rust-version = "1.95"` but no member inherited it, so `cargo metadata` reported
`rust_version: null` for all 28 packages. Cargo could not enforce the floor
during resolution and `clippy::incompatible_msrv` had no MSRV to check against —
a workspace run reported 0 whether or not the code was compatible. A
`CLIPPY_CONF_DIR` probe at `msrv = "1.95"` produced 0 diagnostics and the same
probe at `msrv = "1.63"` produced 288, proving the lint was inactive rather than
passing. No non-workspace dependency declares a `rust-version` above 1.95. The
repair adds `rust-version.workspace = true` to each manifest; it inherits the
declared floor and does not change it.

**`RUST-API-001` — 47 findings, all repaired.** 47 public types implemented no
`Debug`. All 28 crates are `publish = false` and consumed as a git source
(`scripts/verify-source-consumer.sh`), so `pub` is the real consumer contract
and downstream code can neither add a foreign impl nor derive `Debug` on a
struct holding one of these. 27 took a derive. 20 took a manual impl, in two
classes: types holding a foreign trait object or FFI internals that are not
`Debug` (`Arc<dyn Fft>`, `Box<dyn InputStreamHandle>`, `Box<dyn
HardwareBackend>`, CLAP/VST3/AU/LV2 session internals), and the three lock-free
rings — `SpscRing`, `MidiEventRing`, `PluginParamChangeQueue` — where deriving
would format storage owned by the other side of an SPSC discipline. Every manual
impl reports identity and shape through published atomics or safe accessors and
uses `finish_non_exhaustive`; none takes a lock, and none reads a slot outside
its synchronisation. One of the 20 was wrong off macOS and was corrected in the
review wave below. No other common trait was added: these are sessions,
engines, plans, and rings whose equality, ordering, hashing, and default values
would be misleading.

**`RUST-ERR-001` — 14 findings, all repaired.** 13 public panic-capable
operations had no `# Panics` section. Each function was opened and its panic
source identified before the finding was recorded; the retained panic stays a
panic in every case, because each one is a real programming invariant rather
than a recoverable input condition, and no signature or error type changed. The
14th is `error_token` in `signal-plugin-au`'s hosting tests, where
`result.err().expect(...)` became a `clippy::err_expect` failure once
`AuHostedInstance` gained `Debug`; it is now `expect_err` with the same meaning.

**`RUST-UNSAFE-001` — 8 findings, all report-only, none repaired.** 214 unsafe
blocks in the production surface carry no per-operation `SAFETY` comment, against
24 `// SAFETY:` comments in 11 files repository-wide. The public boundary is in a
different state: `clippy::missing_safety_doc` reports 0 and 22 rustdoc
`# Safety` sections are present, so caller obligations are documented even where
the block-local argument is not. Some blocks carry nearby prose instead
(`signal-ipc`'s `unsafe impl Send/Sync` argument, `signal-plugin-clap/src/gui.rs`
module threading contract). The projection gives this rule `report_only`
remediation authority; discharging 214 obligations correctly needs per-operation
review against each foreign API's documented contract (CLAP, VST3 COM,
AudioUnit, LV2, CoreMIDI, Rubber Band, mmap) and is an operator-scheduled
unsafe-hardening lane, not audit repair.

**`RUST-ASYNC-001` — not applicable, 14 units.** No async executor or futures
crate in any manifest; no `async fn` and no `.await` in any of the 826 tracked
`.rs` files; `clippy::await_holding_lock` and `await_holding_refcell_ref` report
0. Concurrency is thread- and channel-based only.

### `RUST-SLOP-001` exact-forwarder ledger

stopslop 0.5.1 `SLOP039` over all 826 tracked `.rs` files returned **0**
candidates. The detector was control-verified three ways: a synthetic
two-function crate was reported on both a file path and a directory walk, and a
forwarder injected into a copy of
`crates/signal-render-plane/src/plugin_events.rs` was reported at the correct
line. Per-unit scanner evidence is recorded for all 13 Rust units.

stopslop excludes test-like, generated, and vendored paths, so a manual sweep
added in-scope candidates: 163 functions whose sole statement is a call whose
arguments equal the parameter list in order. Every one is dispositioned
**retain**. They fall into two groups:

- field accessors (`self.x.len()`, `self.x.clone()`, `self.x.is_some()`), which
  own an encapsulation boundary rather than forwarding between peers;
- documented delegation façades: `signal-host-local/src/host_api.rs` (Contract
  `009`, the Pulse-facing consumer edge), `signal-runtime/src/runtime_api.rs` and
  `runtime_observation_surface/` (Contract `075`, public interface
  decomposition), `signal-runtime/src/interfaces/supervisor_report_family.rs`
  (Contract `002`, supervisor export schema), and
  `signal-render-plane/src/plugin_processor.rs` (the placement-agnostic
  processor handle).

Each was assessed independently under `RUST-READ-001`; none is a wrapper without
a responsibility, and the rule never authorizes repair in any case.

## AGENTS Review

`CLAUDE.md` is unchanged and remains exactly `@AGENTS.md`. No Claude-only rule
was evidenced.

Advisory before/after (context cost, not a quality score):

| | before | after |
| --- | --- | --- |
| non-blank lines | 111 | 110 |
| bytes | 6288 | 6320 |
| approx tokens | 1572 | 1580 |
| headings | 8 | 8 |
| placement leads | 6 | 6 |
| procedure leads | 12 | 9 |
| freshness leads | 6 | 6 |

All eight sections survive with their reader need intact. Dispositions:

| Section | Disposition |
| --- | --- |
| `# Signal` intro | retain — orientation plus the explicit not-owned list |
| `## Non-negotiable boundaries` | rewrite for intent — first bullet only |
| `## Work and authority` | retain, including the Northstar SHA provenance |
| `## Route work by job` | rewrite for intent — de-duplicated |
| `## Canonical surfaces` | rewrite for intent — Chorus absence |
| `## Completion` | retain |
| `## Effigy Agent Contract` | retain — marker-delimited, tool-owned |
| `## Northstar Rust Quality` | retain — marker-delimited, tool-owned |

Three changes, each answering a question the file previously left to
archaeology:

1. "Preserve realtime safety on audio-thread paths" did not say which paths.
   It now names the `signal-render-plane` executor, the `signal-hardware`
   capture/output callbacks, and the DSP kernels they call, and says that
   `signal-runtime` allocates by design.
2. `## Route work by job` restated `graph`, `tasks`, `doctor`, `test --plan`,
   `--json`, and the repo-override prohibition that the marker-delimited Effigy
   Agent Contract already states verbatim at the end of the same file. The
   duplicate is gone; what remains is Signal's own (`validate`, `qa`, `qa:docs`,
   `qa:northstar`, the raw-Cargo fallback rule, and the AGENTS-review command).
   This is where the three recovered procedure leads came from.
3. The Chorus pointer said "when available" without saying what to do when it is
   not. It now says to record the limitation and stop rather than infer the
   contract — which is exactly what this lane had to do.

No safety, authority, worktree, completion, or validation boundary was removed.
The line count moved by one; that was never the goal.

## Validation

| Command | Result |
| --- | --- |
| `effigy qa` | exit 0 |
| `effigy qa:docs` | exit 0 |
| `effigy qa:northstar` | exit 0 |
| `git diff --check` | exit 0 |
| `cargo fmt --all -- --check` | clean |
| installed Northstar `check:agent-instructions` | advisory, before and after |

Per-unit recorder evidence: 85 immutable records over `compiler`, `lint`,
`docs`, `test`, `scanner`, and `graph`. 67 passed, 16 unrun, 1 unavailable,
1 warning. Enforced `cargo clippy --all-targets --all-features -- -D warnings`
is warning-free for every unit.

All of that evidence is **host-local on macOS at the default target**. It
therefore proved nothing about the non-macOS shape of a `cfg`-split public type,
which is exactly the gap Linux CI found — see the review wave below.

Formatting was applied only to files an authorized repair had already changed,
never to a whole crate or the worktree.

## Review Wave — Linux CI Falsified One First-Wave Repair

Orchestrator review of exact head `ddba8e4960be13c1e209ab7ab7629daae10b3c56`
requested changes. The required **Build plugin crates (Linux)** job failed with
three `E0609` errors at
`crates/signal-plugin-au/src/au_host_adapter/hosting/process.rs:98-100`.

The first-wave `RUST-API-001` repair gave `AuProcessSession` a manual `Debug`
that formats `self.unit`, `self.output_left`, and `self.sample_time`. All six of
that struct's other fields are `#[cfg(target_os = "macos")]`; off macOS the
public type is only `processing`. The audit's `plugin-formats` compiler and lint
evidence ran on macOS at the default target, so it compiled the only shape in
which that implementation was valid. The finding and its repair authority were
sound; the implementation was not portable, and the audit's own evidence plan
could not have caught it.

Repair: the macOS-only fields are now guarded in the formatter exactly as they
are declared, and `processing` is rendered on every target.

Verified before pushing:

- **Reproduction.** Reverting to the first-wave implementation and running
  `cargo check -p signal-plugin-au --target wasm32-unknown-unknown` reproduces
  the same three `E0609` errors at lines 98-100 that Linux CI reported.
  `wasm32-unknown-unknown` was already installed and takes the same
  `not(target_os = "macos")` branch, so no toolchain was changed to get this.
- **Correction.** The same command on the corrected file exits 0.
- **No new warnings.** That target reports 7 warnings on the corrected file and
  the identical 7 on `main`, so the review wave adds none.
- **Whole-class check.** All 27 manual `Debug` implementations in the repository
  (24 from this audit, 3 pre-existing) were re-scanned for reads of a
  `cfg`-gated field. `AuProcessSession` was the only one. Derived implementations
  are not at risk, because `derive` follows the field's own `cfg`.
- **macOS proof retained.** `cargo clippy -p signal-plugin-clap -p
  signal-plugin-au -p signal-plugin-lv2 -p signal-plugin-vst3 --all-targets
  --all-features -- -D warnings` is warning-free, `cargo test -p
  signal-plugin-au --all-features` passes 19 tests across 5 binaries, and
  `effigy qa`, `effigy qa:docs`, `effigy qa:northstar`, `cargo fmt --all --
  --check`, and `git diff --check` all pass.
- **Authoritative proof** is the required Linux job on the new exact head.

This is a review-wave change outside the finalized recorder. It was not fed
back through `assess`/`complete`/`finalize`, no sealed record was edited, and no
hash in this closeout was recomputed to accommodate it.

## Retained Limitations

- **8 report-only unsafe findings.** 214 undocumented unsafe blocks stay
  undocumented. Repair needs operator direction and per-API contract review.
- **12 `assessment_depth` limitations, one per unit larger than
  `signal-primitives`.** 123,422 lines were not read line by line. Every crate's
  module documentation, public entry points, and every mechanical complexity
  lead were inspected; the unread remainder is recorded as depth rather than
  converted into findings. `RUST-READ-001` is `degraded` in those 12 units for
  exactly this reason.
- **The 828 `indexing_slicing` sites were triaged by class, not traced one by
  one.** They are dominated by DSP and adapter loops whose bounds come from
  config-derived window, partition, channel, and block sizes. Representative
  sites were traced individually; the rest were not.
- **The 222 `missing_errors_doc` sites were not promoted to findings.** Every one
  already returns `Result` with a typed domain error, which is what the rule
  requires; the lint is evaluation-only and grants no repair authority. They
  remain a real documentation backlog.
- **`test-hardware-device` is `unavailable`.** `signal-hardware-cpal`'s tests
  open a real CoreAudio device; macOS gates input access behind a TCC prompt no
  one can answer in a non-interactive worker session, so the binary blocks rather
  than fails. Observed directly: the run printed
  `test input::tests::enumerates_input_devices_when_present ...` and then
  accumulated 0.1s of CPU over 13 minutes. `signal-hardware` and
  `signal-hardware-coremidi` tests pass, and compiler, lint, and docs evidence
  passed for all three packages. `effigy validate` runs
  `cargo test --workspace --no-run`, so the repository's own gate is unaffected.
- **`test-dsp-stretch` is `warning`, not `passed`.** Exit 0, 194 tests, 0
  failures. The generic evidence adapter classified the test name
  `metric_assessment_aggregates_warnings_and_failures` as a warning-level
  diagnostic. It is an adapter artifact, not a real warning.
- **`graph` evidence is `unrun` for all 14 units.** No finding was derived from a
  graph-index query; package, target, feature, and public-surface inventory came
  from `cargo metadata` and the crate roots. `effigy doctor` also reports the
  index as stale, and this lane did not authorize a reindex.
- **`docs-plugin-sandbox` is `unrun`.** `signal-plugin-sandbox` is bin-only and
  the repository's `check:docs` task builds `-D missing-docs` over `--lib`. It
  also has no crate-level `//!` documentation, which no repository policy
  currently requires of a binary.
- **Pre-existing `effigy doctor` baselines are untouched.** 80 god-file findings
  (79 warning, 1 error: `crates/signal-plugin-sandbox/src/broker/lifecycle.rs` at
  459 code lines) and 6 attention-marker warnings, all of which are `// Note:`
  prose matched as `[NOTE]` deferred-work markers. Threshold-led splitting is out
  of scope for this card, and the markers are false positives.
- **Recorder evidence is single-platform.** Every `compiler` and `lint` record
  was collected on macOS at the default target. For crates with a `cfg` split —
  `signal-plugin-au`, `signal-hardware-coremidi`, and the macOS-gated paths in
  `signal-plugin-vst3` and `signal-plugin-clap` — the sealed evidence covers one
  shape only. Linux CI is the cross-platform gate, and it is what caught the
  `AuProcessSession` defect above.
- **No Chorus checkout is present.** No IPC contract question arose that
  Signal's own contracts could not settle, so nothing was blocked on it.
- **Superseded audit records.** Three earlier audit IDs exist in Git metadata and
  were abandoned before finalization: `signal-g11-003-repo` (initialized only —
  its plan owned Cargo manifests as read-only context, which would have made the
  MSRV finding unrepairable); `signal-g11-003-repository` (abandoned after a
  second `collect` call whose plan omitted already-recorded classes caused the
  tool to fabricate 42 `unrun-<class>-<unit>` records contradicting the passing
  ones); and `signal-g11-003-rust` (abandoned when completion showed two plans in
  one unit owning the same file, which the recorder correctly rejects). None was
  finalized, none produced a `result.json`, and the worktree was reverted to the
  baseline before each re-initialization — the identical discovery snapshot
  `63634165…` on every `inspect` is the proof. `signal-g11-003-repository-audit`
  is the only finalized audit and the only one this closeout claims.

## Operator Stops

None reached. No repair required a new public API, foreign error policy, unsafe
boundary, realtime invariant, compatibility policy, dependency, or version-policy
decision. The MSRV repair inherits an already-declared floor rather than changing
one, and the recorder derived `review_required` authority for all 89 applied
repairs.

## Next Task

Stop for orchestrator exact-head review of the PR. Do not start another card.
The two candidates this audit surfaced but did not open are the unsafe-hardening
lane (214 undocumented blocks, operator decision) and the `missing_errors_doc`
documentation backlog (222 sites); neither is authorized here.
