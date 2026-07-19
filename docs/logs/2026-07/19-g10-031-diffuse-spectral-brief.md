# g10.031 DiffuseSpectral Brief

Date: 2026-07-19
Status: Batch 31.3 complete
Scope: documentation and architecture only

## Decision

Freeze one complete creative renderer in
`docs/architecture/offline-creative-diffuse-spectral-brief.md`.

The renderer uses one sample-rate-scaled long-window STFT, quarter-window
synthesis hop, sample-centred fractional source map, native-channel magnitudes,
linked instantaneous-frequency carrier, deterministic correlated phase
diffusion, bounded log-magnitude evolution, and rolling normalized
overlap-add.

`Dream`, `Spectral`, and `Rough` use the same equations with frozen coefficient
profiles. `Dream` is the smooth PaulXStretch-centred default. `Spectral` exposes
stable vocoder-like separation. `Rough` exposes less correlated, less smoothed
novelty. `Cloud` and `Cyclic` fail before rendering and retain separate later
owner seams.

The brief freezes:

- request domain, window law, scheduler, source map, and interpolation
- linked reference, carrier, diffusion generator, and dormant state
- magnitude smoothing, shared power envelope, character macros, and `space`
- exact boundaries, rolling normalization, `32 MiB` state cap, determinism,
  and cost
- structural, synthetic, long-form mono, and independent stereo gates
- candidate file ownership, minimal admission, rejection, and deletion

There is no transient detector, reset, duplicate source read, cyclic buffer,
attack layer, or tail layer. Creative smear comes from the one spectral field;
replicas and stutter remain failures.

## Boundary

No Rust, DSP, harness, fixture, report mode, public API, artifact schema,
Loophole, or Chorus surface changed. The frozen OfflineHighQuality renderer and
Contract `084` closure remain untouched.

## Readiness

Batch 31.4 is genuinely ready. It has one bounded implementation target,
explicit gate order, exact stop conditions, and deletion behavior. The work
must happen in one disposable worktree. Structural failure stops before
long-form capture.

## Next Task

Execute Batch 31.4 only. Create one disposable worktree, implement the frozen
`DiffuseSpectral` brief, and run its structural gate. Keep `Cloud`, `Cyclic`,
overlap routing, dynamic ratio, cache, and product API work closed.
