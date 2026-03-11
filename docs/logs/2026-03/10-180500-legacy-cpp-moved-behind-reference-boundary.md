# 2026-03-10 18:05:00 - Legacy C++ moved behind reference boundary

## Summary

Moved Signal's legacy C++ implementation behind `legacy/cpp` so the active repo
surface is now clearly the Rust workspace plus Northstar-shaped docs.

## Changes

1. Moved:
   - `src/` -> `legacy/cpp/src/`
   - `tests/` -> `legacy/cpp/tests/`
   - root `CMakeLists.txt` -> `legacy/cpp/CMakeLists.txt`
2. Added a root wrapper `CMakeLists.txt` that delegates to `legacy/cpp`.
3. Updated `effigy.toml` dev path for the relocated legacy executable.
4. Added `legacy/README.md`.
5. Updated README and active docs so the library-first active surface is clear.

## Validation Performed

- `git diff --check`
- `effigy signal/health --repo .`
- `effigy signal/validate --repo .`

## Risks

- The legacy C++ tree is still buildable and still present; this batch changes
  repo posture and layout, not final removal.

## Next Task

Keep reducing the active root-facing importance of the legacy tree while the
fresh Pulse, Spark, and Aura skeletons are created around the clean rebuild
architecture.
