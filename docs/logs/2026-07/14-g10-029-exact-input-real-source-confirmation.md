# Exact-Input Real-Source Confirmation

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CZ
Scope: report-only coherent Signal versus pinned Signalsmith confirmation

## Decision

Pass the frozen objective decision rule. Open one concealed musical comparison.
Do not select or promote the coherent representation yet.

## Method

The six existing five-second mono sources are written once as exact 16-bit
inputs. Coherent Signal and pinned Signalsmith Stretch `1.3.2` consume those
same files at `1.5x` or `2.0x`. The comparator uses the library's public seed-
`0` constructor. The stock CLI seeds from `std::random_device`, which makes its
`2x` phase-randomization path vary across processes. Both engines render each
row twice. The report checks exact output length,
finiteness, boundary integrity, peak growth, event timing, transient replicas,
static spectral residual, and deterministic hashes.

Apply the [fixed-seed comparator patch](../../../crates/signal-dsp-stretch/tests/fixtures/signalsmith_stretch_fixed_seed.patch)
to the pinned Signalsmith checkout before building the confirmation CLI.

The source-derived `44.1 kHz` geometry is:

- support/interval: `5292/1323`
- transform/bands: `6144/3072`
- periodic-Kaiser hash: `70ba1688509b2915`

## Evidence

- structural failures: `[0, 0, 0, 0, 0]`
- coherent Signal hard failures: `0`
- pinned Signalsmith hard failures: `0`
- coherent Signal regression rows for timing/replicas/static/boundary:
  `[2, 3, 2, 6]`
- exact coherent and pinned repeat: yes
- input manifest hash: `8ede75dbae2254b2`
- coherent aggregate hash: `7ec654eb414041ce`
- pinned aggregate hash: `ee39390a1e17d923`
- measurement hash: `d9f2228661af1e53`
- report hash: `7a6b1e7dd7ba5c13`
- seeded comparator binary SHA-256:
  `d967fea75caba0303243d328341ff91514000969e50003c523cad83c647dfc93`

Signal has lower event-offset error on four rows and lower static residual on
four. It has lower replica ratio on three. Boundary-growth is worse on all
six. The boundary metric becomes extreme when exterior source steps approach
zero, so magnitude alone is not a reliable audible ranking. The direction is
still consistent and cannot be dismissed. Starts, ends, and transient-edge
artifacts are mandatory targets in the concealed comparison.

## Closed Lanes

- predictor-law changes, parameter sweeps, and third mechanisms
- stereo and dynamic ratio
- product routing and promotion

## Next Task

Run Batch 29.6DA. Export source references plus deterministically concealed
coherent-Signal and pinned-Signalsmith candidates for all six rows. Freeze the
manifest and mapping before listening.
