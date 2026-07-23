# 032 - Cyclic Creative Stretch Research

Status: active; Batch 32.2 ready
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

Status: ready

Use ignored `target/` and disposable external build state only.

- [ ] build source-faithful probes for pinned Potenza and SickoCV schedules
- [ ] build the pinned Sonic specialist contrast
- [ ] capture ReaReaRea at `2x` for the five retained musical sources
- [ ] capture ReaReaRea synthetic impulse, tone, chord, noise, and stereo rows
- [ ] render `2x`, `4x`, and `8x` across short, medium, and long cycle regions
- [ ] record exact source revisions, commands, request semantics, and output
  hashes
- [ ] measure event replicas, cycle cadence, pitch delta, join width, energy,
  boundaries, and stereo relation
- [ ] keep every artifact ignored and out of public or hidden Signal surfaces

Stop after the forensic receipt. Do not select a Signal renderer in this
batch.

## Batch 32.3 - Behavioral Synthesis And Gate Correction

Status: blocked on Batch 32.2

- [ ] distinguish compressed-anchor and repeat/jump behavior
- [ ] decide whether fixed cycle length is sufficient
- [ ] classify automatic cycle selection as Cyclic assistance, `INTELL`, or
  unsupported
- [ ] freeze the minimum useful UI vocabulary
- [ ] replace arbitrary absolute quality thresholds with hard integrity and
  comparator-relative diagnostics
- [ ] record remaining uncertainty explicitly

This batch is docs only. If evidence cannot distinguish a complete schedule,
close or extend research. Do not choose by convenience.

## Batch 32.4 - Complete Renderer Brief

Status: blocked on Batch 32.3

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

Execute Batch 32.2 only. Produce the ignored executable forensic matrix and
receipt. Do not implement Signal Cyclic DSP.
