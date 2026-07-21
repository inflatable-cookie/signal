# g10.031 Linked STN Renderer Brief

Date: 2026-07-21
Batch: 31.42
Status: complete; one isolated candidate ready

## Scope

Freeze one self-contained clean-room `LinkedStnNoiseMorph` renderer brief.
Change documentation only. Stop before candidate DSP, tests, evidence output,
product routing, Loophole, or Chorus work.

## Baseline

- branch: `main`
- starting commit: `aec57ed7 Select linked STN creative stretch owner`
- worktree: clean
- rejected renewal branches, checkpoints, and source: absent by required
  cleanup

## Frozen Renderer

- one exact signed-rational map and shared output lattice
- sample-rate-normalized long tonal and short transient reconstructing masks
- persistent linked tonal peak and bin phases with dormancy/reactivation
- shared transient detection, class, segment, exact anchor, unit-rate native
  emission, and one-emission ledger
- continuous counter excitation shaped by interpolated residual covariance in
  a swap- and polarity-equivariant mid/side factor
- residual-only preserve-to-widen `space`; tonal and events stay native
- mapped source-envelope correction, normalized component WOLA, zero exterior,
  no renderer fade, and exact crop
- `96 MiB` duration-independent working-state cap, fixed traversal, and no
  processing allocation

Authority:

- `docs/architecture/offline-creative-linked-stn-noise-morph-brief.md`

## Evidence And Cleanup

The candidate owns one compile-linked table with `18` structural and `10`
synthetic owners. Sources, supports, metrics, comparator values, thresholds,
seed, counter vectors, and assertions are self-contained. The receipt records
checkpoint, tree, file, toolchain, row, and output digests before cleanup.

Objective order remains construction, structural, synthetic, concealed
long-form mono, then eligible independent stereo. Listening remains creative
promotion authority. The operator's speaker review can reject but cannot
satisfy independent stereo admission.

Any terminal miss deletes the complete candidate without repair or rerun.
Only a later batch may minimally admit a complete pass.

## Repository Result

- docs, roadmap, contract, architecture, and log authority updated
- no DSP, test, harness, fixture, dependency, API, route, cache, artifact, or
  product code changed
- `OfflineHighQuality`, Contract `084`, RealtimePreview, Loophole, and Chorus
  unchanged
- no external production dependency introduced

## Risk

Source evidence stops at short mono `8x`. Component leakage, persistent tonal
quality, one-shot transient character, residual low-end noise, linked residual
image, entry/tail energy, long-form music, `16x`, and cost remain terminal
Signal risks. The brief is buildable; it is not a parity or quality claim.

## Validation

- `git diff --check`: pass
- `effigy qa:docs`: pass
- `effigy qa:northstar`: pass
- `effigy health`: pass
- `effigy validate`: pass
- `effigy doctor`: expected pre-existing god-file and attention-marker
  findings only

## Next Task

Run Batch 31.43 only in `signal-candidate-31-43` on
`candidate/g10-031-linked-stn-noise-morph`. Implement the frozen brief once,
complete construction, freeze one checkpoint, then run structural and
synthetic admission. Stop before listening on any miss. Do not change `main`,
merge, expose the product, touch Loophole or Chorus, or push.
