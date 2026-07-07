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

## Next Task

Implement the real-corpus report runner from `g10.021` Batch 21.2.
