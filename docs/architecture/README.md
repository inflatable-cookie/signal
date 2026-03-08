# Architecture

Architecture docs define the system shape and invariants derived from vision.
They set constraints roadmap batches must honor.

## Files

- `system-architecture.md`
- `package-map.md`
- `docs/contracts/001-shared-dsp-and-host-boundary.md`
- future `docs/contracts/00n-<slug>.md` files as needed

## Writing rules

- Link architecture updates to current vision artifact(s).
- Keep milestone execution lists in roadmap files, not architecture files.
- Use contract docs for explicit technical boundaries that need validation and migration notes.

## Next task

Keep `system-architecture.md` current as Signal crate and runtime-host
boundaries firm up, then add focused contracts only where ambiguity would
otherwise create duplicate DSP ownership or unstable host boundaries.
