# 2026-03-17 - g07.004 Batch 4.1 complex plugin-I/O contract opening tranche

## What changed

- froze the backend-neutral complex plugin-I/O boundary in
  `docs/contracts/035-plugin-complex-io-topology-and-multi-output-instrument-contract.md`
- defined bounded shared vocabulary for plugin port class, complex plugin-I/O
  topology, multi-output instrument identity, bus-capable FX class,
  plugin-facing attachment policy, and fallback outcome
- aligned the active roadmap, contract index, architecture reference, and
  generation pointers so the next runtime batch now targets Batch 4.2 instead
  of leaving `g07.004` as a prose-only handoff

## Why it matters

Complex plugin bus behavior is now constrained by one Signal-owned contract
before CLAP, VST3, and AU runtime realization widens. That keeps later
multi-output and bus-capable plugin work from falling back into adapter-local
pin naming or host-local routing interpretation.

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Next task

Continue `g07.004` with Batch 4.2 by materializing runtime-owned complex
plugin-I/O, multi-output instrument, and bus-capable FX receipts across
discovery, execution, render, and stable host-edge surfaces without reopening
adapter-local pin ownership.
