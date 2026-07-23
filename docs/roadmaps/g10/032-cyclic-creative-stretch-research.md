# 032 - Cyclic Creative Stretch Research

Status: active; Batch 32.4 ready
Owner: dsp
Updated: 2026-07-23
Contracts: `046`, `085`

## Problem

Signal has no Akai-style `Cyclic` renderer. The earlier program treated fixed
cyclic overlap and similarity-aligned overlap as one repair sequence. Original
Akai manuals and current systems show two distinct modes: fixed `CYCLIC` and
material-adaptive `INTELL`.

The first Signal candidate stopped on an absolute pitch threshold before
comparator listening. The second entered the `INTELL` family and failed its
frozen search reachability. Neither result establishes which source schedule
creates the desired ReaReaRea-like effect.

## Goal

Resolve the complete cyclic mechanism before another renderer is specified:

- source-progress clock
- cycle selection
- ratio accumulation
- local read and join law
- event placement
- linked-channel ownership
- exact boundaries
- honest user controls
- comparator-relative diagnostics and listening

Target expansion remains continuous above `1x` through `8x`, with mandatory
`2x`, `4x`, and `8x`.

## Non-Goals

- candidate DSP on `main`
- repair of `CyclicGrain` or `SimilarityAlignedCyclic`
- automatic routing
- Dream, Cloud, transparent, dynamic-ratio, cache, or product integration
- copied external source expression or constants
- a parameter sweep used to choose Signal implementation constants

## Batch 32.1 - Source Architecture Survey

Status: complete

- [x] re-read both rejected Signal cyclic briefs and receipts
- [x] separate original Akai `CYCLIC` from `INTELL`
- [x] pin Potenza slow-anchor two-grain source
- [x] pin SickoCV repeat/jump cycle source
- [x] pin Sonic period insertion as specialist automatic-cycle evidence
- [x] retain ReaReaRea as the primary behavioral target
- [x] classify Akaizer and TAL-Sampler as optional proprietary behavior
- [x] reinterpret the old absolute pitch stop as unresolved target-relative
  evidence
- [x] publish one canonical source dossier

Authority:

- [Cyclic Time-Stretch Source Architecture](../../research/specimen-dossiers/cyclic-time-stretch-source-architecture.md)

## Batch 32.2 - Source-Faithful Executable Forensics

Status: complete

Use ignored `target/` and disposable external build state only.

- [x] build source-faithful probes for pinned Potenza and SickoCV schedules
- [x] build the pinned Sonic specialist contrast
- [x] capture ReaReaRea at `2x` for the five retained musical sources
- [x] capture ReaReaRea synthetic impulse, tone, chord, noise, and stereo rows
- [x] render `2x`, `4x`, and `8x` across short, medium, and long cycle regions
- [x] record exact source revisions, commands, request semantics, and output
  hashes
- [x] measure event replicas, cycle cadence, pitch delta, join width, energy,
  boundaries, and stereo relation
- [x] keep every artifact ignored and out of public or hidden Signal surfaces

Stop after the forensic receipt. Do not select a Signal renderer in this
batch.

## Batch 32.3 - Behavioral Synthesis And Gate Correction

Status: complete

- [x] distinguish compressed-anchor and repeat/jump behavior
- [x] decide whether fixed cycle length is sufficient
- [x] classify automatic cycle selection as Cyclic assistance, `INTELL`, or
  unsupported
- [x] freeze the minimum useful UI vocabulary
- [x] replace arbitrary absolute quality thresholds with hard integrity and
  comparator-relative diagnostics
- [x] record remaining uncertainty explicitly

This batch is docs only. If evidence cannot distinguish a complete schedule,
close or extend research. Do not choose by convenience.

Decision:

- centred compressed-anchor Cyclic behavior selected
- raw whole-cycle repeat/jump rejected as the primary target
- fixed manual cycle sufficient for the first character
- optional automatic cycle classified as later specialist assistance
- `duration`, `character=Cyclic`, and `cycle` frozen as the minimum UI
- integrity remains hard; finite character metrics diagnose; listening
  promotes

Authority:

- [Offline Creative Cyclic Behavioral Synthesis](../../architecture/offline-creative-cyclic-behavioral-synthesis.md)

## Batch 32.4 - Complete Renderer Brief

Status: ready

Freeze one buildable renderer only if the source and behavioral evidence
select it. The brief must jointly own:

- exact source/output map and cycle grammar
- continuous fixed ratios and exact target length
- cycle control and any automatic guidance
- local read, interpolation, and join law
- event placement and commanded-replica definition
- linked stereo
- boundaries, memory, determinism, and cost
- structural, synthetic, comparator, and listening order
- rejection, cleanup, and minimal admission

No implementation belongs in this batch.

## Later Batches

Only a complete approved brief may open one isolated candidate. Candidate
implementation, admission, public exposure, routing, and product integration
remain separate stop points.

## Completion Gate

- source families are pinned and clean-room boundaries recorded
- behavioral forensics distinguish or explicitly fail to distinguish schedules
- operator listening owns musical character
- integrity metrics reject clicks, invalid duration, broken stereo, and
  uncommanded replicas
- one complete renderer brief exists before implementation
- rejected candidate code and evidence scaffolding stay out of `main`

## Next Task

Execute Batch 32.4 only. Freeze one complete centred compressed-anchor Cyclic
renderer and its Contract `085` Rule 11 evidence authority. Do not implement
Signal Cyclic DSP.
