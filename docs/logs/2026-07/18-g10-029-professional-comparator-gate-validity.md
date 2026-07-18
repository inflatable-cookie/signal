# g10.029 Professional-Comparator Gate Validity

Date: 2026-07-18
Batch: 29.7AI
Contract: `082`, Rule 31P
Decision: revise local and exact-mechanics vetoes

## Scope

Test pinned Rubber Band R3 `4.0.0` against the exact rules that rejected
Signal and SBSMS candidates. Freeze all inputs and thresholds before the first
render. Do not tune or implement a renderer, listen, read the holdout, or open
product work.

## Frozen Evidence

- `48` stereo rows
- whole-render and interior calibrated metrics
- eight local normalized-Gram windows per row, `384` total
- duplicate channels
- duplicate stereo versus mono
- hard pan with silent peer
- channel swap
- polarity inversion
- quarter gain
- command: `rubberband -q -3 -t <ratio> <input.wav> <output.wav>`
- version: `4.0.0`

All inputs and mechanics were prepared before Rubber Band discovery or the
first render. Two complete passes produced identical rows, mechanics, and
hashes.

## Result

| Field | Result |
| --- | ---: |
| Calibrated stereo failures | `0/48` |
| Old Signal-relative local failures | `13/48` |
| Local windows improved over Signal | `245/384` |
| Rubber Band maximum local residual | `0.01744693815260` |
| Signal maximum local residual | `0.02522090848652` |
| Duplicate-channel error | `0` |
| Duplicate-stereo/mono error | `0` |
| Silent-peer error | `0` |
| Swap error | `0` |
| Polarity error | `0.950164794921875` |
| Quarter-gain error | `0.04590606689453125` |

The local veto is invalid as written. It requires a professional comparator to
improve on current Signal in at least half the windows of every row. Rubber
Band does not, despite passing every calibrated row and having the better
global local residual.

Polarity and gain are not exact invariants of a professional nonlinear
renderer. Duplicate equality, mono parity, silent-peer isolation, and swap are
genuine structural invariants in this specimen and remain hard.

## Hashes

| Surface | Hash |
| --- | --- |
| Rubber Band binary | `1c4b0c5b9f8fb803` |
| Frozen inputs | `4712bef6ac17870e` |
| Comparator outputs | `95752edc43fc6997` |
| Command contracts | `628f977d4361ad21` |
| Measurements | `8ec1d7158d1209ca` |
| Exact local envelope | `9574e5e2e53d1a63` |
| Complete evidence | `b9331f0858326f19` |

The binary hash identifies the exact local Homebrew specimen. Version remains
the portable pin; output and measurement hashes detect specimen drift.

## Contract Decision

Rule 31Q keeps the `48`-row calibrated gate unchanged. It keeps duplicate,
mono-parity, silent-peer, and swap mechanics hard at `1e-6`. Polarity and gain
remain diagnostics.

The old local veto is replaced by a professional-comparator boundary:

- at least `245/384` windows improve on current Signal
- at most `13/48` old-rule row failures
- global maximum local relation residual at most `0.01744693815260`

The exact `384`-cell envelope remains retained for attribution, not waveform
optimization.

No closed renderer reopens. SBSMS still fails genuine structural mechanics,
mono integrity, long-development quality, and bounded state.

## Reproduction

```bash
cargo test -p signal-dsp-stretch --release \
  source_studied_professional_comparator_gate_validity -- \
  --ignored --nocapture
```

Generated evidence lives under ignored
`target/stretch-professional-comparator-gate-validity/`.

## Next Task

Run Batch 29.7AJ under Rule 31Q. Research the shared-decision and channel-
equivariant synthesis boundary in source-backed professional renderers. Select
at most one complete topology or stop. Keep the holdout, listening, Batch
29.8, and product surfaces closed.
