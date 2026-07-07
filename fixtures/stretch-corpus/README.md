# Stretch Corpus Fixtures

Status: active
Scope: `g10.021` Signal-native high-quality stretch evidence

## Purpose

This tree defines the first real-audio corpus shape for Signal stretch
evidence. It does not contain licensed source audio.

Signal-generated cases stay inline in `signal-dsp-stretch`. Operator-provided
listening material may be placed under `licensed-listening/` on a local
machine, but those files must not be committed.

## Layout

- `manifests/stretch-corpus-v1.md`: corpus families, source policy, missing
  asset behavior, and path hints
- `licensed-listening/`: local-only operator-provided audio
- `external-benchmark/`: optional external rendered comparison outputs
- `synthetic/`: reserved for generated artifacts if a future runner writes
  them to disk

## Source Rules

- Do not commit licensed source audio.
- Do not add Rubber Band source or library dependencies.
- Rubber Band CLI output may be used only as optional clean-room comparison
  output.
- Missing licensed listening material is reported as a gap and skipped by the
  future real-corpus runner.
- Synthetic cases must remain runnable without file I/O.

## Report Command

Generate the deterministic draft-vs-OfflineHighQuality report:

```bash
cargo run -p signal-dsp-stretch --bin stretch-corpus-report -- \
  --report-name stretch-corpus-v1-local \
  --projection-epoch projection:local \
  --listening-source-manifest target/stretch-corpus-fma-review-seed.tsv \
  --decode-listening-sources \
  --measure-decoded-stretch \
  --output target/stretch-corpus-v1-local.txt
```

The report includes local listening-source provenance rows when a manifest is
supplied, missing licensed-asset rows for still-uncovered cases, synthetic
objective comparison rows, ratio/pitch curve fields, and listening-note slots.
The `--output` path is local evidence and should not be committed unless a
future roadmap explicitly defines an artifact location.

`--decode-listening-sources` appends bounded decoded source-profile rows:
sample rate, channels, analyzed frames, peak/RMS, zero-crossing rate, and
transient density. These rows prove decoder and source-profile coverage. They
are not stretch-quality verdicts.

`--measure-decoded-stretch` appends bounded decoded-source stretch metric rows
for each corpus ratio on each local source. The first metric set compares the
draft phase-vocoder baseline with OfflineHighQuality for timing drift and
transient smear over a short decoded excerpt. Transient rows include input,
output, matched, and missed transient counts for each backend so high penalties
can be separated from true attack widening. They also include mean/max timing
error for matched transients and mean/max nearest-output distance for missed
transients, plus the expected and nearest output frame for the largest missed
distance, so alignment failures can be sorted before DSP changes. Use
`--decoded-stretch-frame-limit N` to change the excerpt size; the default is
ten seconds at 48 kHz. These rows are objective evidence, not a replacement
for listened curation.

Add optional external rendered-output comparisons with repeated
`--external-benchmark-render` groups:

```bash
cargo run -p signal-dsp-stretch --bin stretch-corpus-report -- \
  --report-name stretch-corpus-v1-local \
  --projection-epoch projection:local \
  --external-benchmark-tool rubberband-cli \
  --external-benchmark-render stretch:loop_seam 1.0 \
    fixtures/stretch-corpus/external-benchmark/rubberband-loop-seam-1x.wav \
  --output target/stretch-corpus-v1-local.txt
```

This is a rendered-output-only comparison. Signal reads WAV metadata from the
operator-supplied file and records timing drift when the case maps to a
Signal-generated source. Signal does not run, link, vendor, translate, or depend
on Rubber Band.

## FMA Local Selection

The FMA large bundle can seed local-only listening candidates:

```bash
cargo run -p signal-dsp-stretch --bin fma-stretch-corpus-select -- \
  --fma-root /Users/tom/Downloads/FMA \
  --per-family 5 \
  --output target/stretch-corpus-fma-selection.md \
  --tsv-output target/stretch-corpus-fma-selection.tsv \
  --review-seed-tsv-output target/stretch-corpus-fma-review-seed.tsv \
  --review-seed-per-family 2
```

The Markdown file is for operator review. The TSV file can be passed to
`stretch-corpus-report --listening-source-manifest` after unwanted candidates
are removed. Both files record FMA track ids, local MP3 paths, genre-derived
corpus family, artist/title, track URL, and artist license metadata. They are
local selection aids only. Do not commit FMA audio or generated local evidence
reports.

The review-seed TSV is a no-listening shortcut. It keeps a fixed number of
candidates per family and avoids repeated artists where possible. It is coverage
evidence, not a subjective quality choice.

## Next Task

Run `stretch-corpus-report --measure-decoded-stretch` with the review-seed TSV
when listening review is not available, then replace it with a listened
curation before promotion.
