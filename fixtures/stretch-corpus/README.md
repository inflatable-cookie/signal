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
  --output target/stretch-corpus-v1-local.txt
```

The report includes missing licensed-asset rows, synthetic objective comparison
rows, ratio/pitch curve fields, and listening-note slots. The `--output` path is
local evidence and should not be committed unless a future roadmap explicitly
defines an artifact location.

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

## Next Task

Use the completed `g10.021` evidence runner to collect operator-supplied real
listening material and optional external rendered-output comparisons.
