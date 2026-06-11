# 009 - Workspace Consolidation And Truthful Front Doors

Status: complete (g10 remains open for continuation packets)
Owner: core-product
Created: 2026-06-11
Depends on: g10.004, g10.005, g10.006, g10.007, g10.008
Vision tags: `HYGIENE`, `CI`, `DOCS-TRUTH`

## Problem

The workspace has no CI, no `[workspace.dependencies]` (version strings
already skew across crates), no workspace lints, no toolchain/rustfmt/clippy/
deny configuration. The README's repository layout omits 11 crates —
including signal-render-plane and signal-hardware-output-cpal, the two that
are the production audio path. Test counts are inverted relative to value
(supervisor-tools 149 tests vs render-plane 7, output-cpal 0). After the
demolition lanes land, the front doors must describe what actually remains.

## Goals

- [x] `[workspace.dependencies]` for shared deps; per-crate versions unified
- [x] `[workspace.lints]` + rustfmt config; clippy clean or explicitly
      allowed with reasons
- [x] CI: build + test + fmt + clippy on push (host-device-dependent tests
      skippable)
- [x] README and system inventory rewritten to the post-g10 crate set, with
      the production audio path documented first
- [x] CHANGELOG entry summarizing the g10 program
- [x] test-coverage rebalance: smoke tests where the production path is thin
      (output-cpal), delete suites that died with their subjects
- [x] edition review (2021 → 2024 decision recorded either way)

## Non-Goals

- [ ] no new features
- [ ] no docs beyond truth-restoration (no speculative architecture prose —
      that pattern is what g10 removed)

## Execution Plan

### Batch 9.1 - Cargo Hygiene

- [x] workspace deps, lints, fmt, toolchain file; fix skew; clippy pass

### Batch 9.2 - CI

- [x] pipeline running build/test/fmt/clippy; device-dependent tests gated

### Batch 9.3 - Front Doors

- [x] README, system inventory, CHANGELOG, roadmap front doors (no closure
      record — g10 stays open for the continuation packets)

## Acceptance Criteria

- [x] fresh clone: one command builds and tests green; CI enforces it
- [x] every crate in the workspace appears in the README with an honest
      one-line description
- [x] generation-index: g10 explicitly continued (stays active for packets
  010+; no closure record)

## Risks and Mitigations

- Risk: clippy avalanche on legacy code.
- Mitigation: warn-level baseline first, deny on new code; recorded follow-up.

## Evidence Requirements

- [x] local gate outputs recorded below; first CI run lands with the push

## Progress (2026-06-11)

- Batch 9.1 (Cargo hygiene): `[workspace.dependencies]` now carries every
  internal path dep and all 12 external deps (serde, serde_json, hound,
  libloading, plist shared by 2+ crates; clap-sys, cpal, json5, memmap2,
  rayon, rustfft, symphonia centralised too); member manifests are
  `workspace = true` only, version skew fixed (serde "1"/"1.0",
  libloading "0.8"/"0.8.9"). `[workspace.lints]`: rust
  `unsafe_op_in_unsafe_fn = warn`, clippy `all = warn`; `[lints] workspace =
  true` in all 24 crates. rustfmt.toml (default style pinned) and
  rust-toolchain.toml (stable, unpinned minor) added. Clippy 44 warnings → 0:
  auto-fixed let-and-return/needless-borrow/unused-imports etc; manual fixes
  for `&PathBuf`→`&Path` (8), AU identical-if-blocks, dead struct-update;
  deleted dead test-only host-local code (offline_render.rs local delegated
  executor + scale_audio_buffer, zero callers); explicit allows with reasons:
  clap discovery `unsafe_op_in_unsafe_fn` (dense FFI, deferred to hosting
  rebuild), 2x too_many_arguments, 3x enum_variant_names (frozen public
  names), 1x module_inception (test file include pattern).
- Batch 9.2 (CI): `.github/workflows/ci.yml` — fmt --check, build, clippy
  `-D warnings`, `cargo test --workspace` (parallel; the keepsake scan
  hazard died with g10.007's discovery fix). cpal smoke tests verified to
  self-skip when no output device exists; no gating needed.
- Batch 9.3 (front doors): README layout rewritten — all 24 crates with
  honest one-liners, production audio path first; narration wall and
  supervisor-tools/host-server/coreaudio references gone; responsibilities
  list made honest (no MIDI/hosting claims). system-inventory.md rewritten to
  the post-g10 layer inventory. CHANGELOG: three entries summarizing g10
  002-008 (~98k LoC removed) and this packet. effigy `dev` task removed
  (signal-host-local has no binary; runnable examples documented instead).
  dsp-analysis-feature-reference.md: superseded/corrected banners on the
  continuity, embed model-registry, and loudness sections with git-history
  pointers; bounded pass only.
- Carried items from earlier packets: signal-dsp-resample comparison-report
  ceremony deleted (g10.008 item; crate docs now point RT-path work at
  `signal_dsp::PolyphaseInterpolationTable`, crate scoped to offline/
  streaming analysis input prep). signal-analysis corpus/harness moved behind
  a `test-support` feature (default off), sibling analysis crates consume it
  via dev-dependency with the feature — chosen over cfg(test) because the
  harness is cross-crate test infra; sibling tests compile unchanged.
- Edition review: staying on 2021. Nothing in the workspace needs 2024
  semantics, and the migration churn (unsafe-attr, expr fragment changes)
  buys nothing while the hosting rebuild is pending. Revisit when a real
  need appears.
- Gates: cargo build/test --workspace green (parallel), fmt --check clean,
  clippy zero warnings, `RUSTFLAGS='-D missing-docs' cargo check --workspace
  --lib` (effigy check:docs) green, pulse `cargo test --lib` green, aura
  src-tauri `cargo check` green.

## Next Task

g10 REMAINS OPEN: no closure record was written and the generation-index
keeps g10 active — the owner is extending the generation with continuation
packets (010+). Rebuild-on-demand items still pull from
`docs/roadmaps/backlog/post-g10-rebuild-on-demand.md` when Loophole schedules
the corresponding product features.
