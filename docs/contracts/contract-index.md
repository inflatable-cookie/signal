# Contract Index

Status: active
Owner: core-product
Updated: 2026-07-16

## Purpose

Provide one searchable front door for the Signal contract set so roadmap work
can group dependencies by boundary family instead of relying on contract-number
memory alone.

## Current Lane

- Signal is baseline-routed with no active strict spec lane
- `g10.029` is the active correctness-first stretch roadmap
- `046` governs sample-domain stretch behavior and its promotion gates
- `082` governs the report-only successor policy, the competitive Rule 31
  coherent mono baseline, and the Rule 31H reference-relative linked-stereo
  proof
- `g10.028` source-fill work is paused until the actual DSP kernel and source
  consumption geometry pass the `g10.029` correctness gate

## Working Rule

- `001` repository working rules

## Core architecture and export baseline

- `001` shared DSP and host boundary
- `002` supervisor export schema and report boundary
- `003` crate maturity and public runtime boundary baseline

## Runtime scheduling, recovery, and diagnostics

- `004` runtime multicore scheduling and anticipative execution
- `005` runtime work orchestration and deferred service policy
- `012` runtime interruption taxonomy and resumability
- `013` recording continuity, MIDI capture, and checkpoint truth
- `014` plugin isolation policy, transport rebind, and shared sandbox continuity
- `015` offline render recovery and resumability
- `016` runtime fault cause attribution and diagnostic receipts
- `017` per-block execution timing and pressure snapshots
- `018` graph critical path, hot node, and worker-lane instrumentation
- `019` deferred-work scheduler priority, backpressure, and cancellation
- `025` device supervision restart state machine and fault boundary

## Plugin adapter and capability breadth

- `007` plugin backend and host-neutral delegation
- `008` backend-neutral plugin capability and adapter breadth
- `020` VST3 adapter baseline and runtime-owned lifecycle
- `021` AU adapter baseline and runtime-owned lifecycle
- `022` backend capability parity, Linux plugin support, and cross-adapter
  conformance
- `024` plugin preset state interchange, portable recall, and ARA context
- `038` LV2 adapter baseline and Linux-native plugin lifecycle
- `039` Linux cross-adapter plugin parity and sandbox policy
- `055` LV2 worker, URID, patch, and extension negotiation
- `056` complex plugin pin matrix and dynamic bus negotiation
- `083` VST3 bundle discovery diagnostics and helper outcome classification

## Hardware, backend, and endpoint portability

- `006` runtime hardware portability and clock-domain boundary
- `026` clock-domain drift, duplex mismatch, and endpoint topology
- `027` external I/O monitoring, tap point, and loopback measurement
- `040` Linux audio backend portability across ALSA, JACK, and PipeWire
- `041` Linux backend clocking, duplex, and endpoint topology parity
- `052` live Linux audio backend ownership and session lifecycle
- `053` JACK transport, graph, and backend-native coordination
- `054` PipeWire and ALSA session-role, device-claim, and stream-policy parity
- `065` live external MIDI device ownership and backend parity
- `066` cross-backend device protocol and live workflow acceptance
- `067` live Linux backend acceptance and failure injection

## MIDI, control-surface, and advanced hardware depth

- `023` generic MIDI note expression and plugin event model
- `042` external MIDI endpoint graph and device identity
- `043` MIDI 2.0, MPE, and richer controller expression
- `044` control-surface transport mapping and feedback
- `045` advanced hardware extensibility and scripting-safe device policy
- `060` advanced control-surface display, motor, and haptic transport
- `061` control-surface scene mapping, feedback pages, and safe action graphs
- `069` control-surface and preview workflow acceptance

## Routing, immersive, and graph topology

- `032` canonical multichannel layout and channel role
- `033` sidechain routing and secondary-input execution
- `034` multi-bus graph execution and auxiliary topology
- `035` plugin complex I/O topology and multi-output instrument
- `036` spatial adapter execution
- `037` surround bed, object, and mix policy expansion
- `057` immersive object rendering and room-policy substrate
- `058` speaker deployment, fold-down, and monitoring scene
- `059` renderer capability negotiation and immersive export
- `068` immersive render and monitoring acceptance

## Preview, transform, and media services

- `028` media indexing, waveform analysis, and preview service
- `029` analysis metadata extraction and library service
- `046` sample-domain time-stretch engine
- `047` warp marker, transient anchor, and tempo-assist analysis
- `048` post-warp render cache and transform artifact
- `049` low-latency audition, scrub, and preview transform service
- `062` preview output routing, audition sink, and low-latency device policy
- `063` preview browser queue, media audition, and transform scheduling
- `064` asset/session transform persistence, retention, and cache placement
  policy
- `082` offline time-stretch synthesis policy

## Packaging, conformance, and generation closure

- `009` shared host convenience API and consumer-edge contract
- `010` publication-grade packaging manifest and release receipt
- `011` shared downstream conformance and release acceptance automation
- `030` fault injection harness and multi-backend acceptance
- `031` long-session soak promotion gate and Loophole readiness
- `050` multichannel Linux time-stretch and control-surface acceptance
- `051` generation closeout and Loophole feature-readiness gate
- `070` integrated live ownership and workflow acceptance
- `071` generation closeout and downstream workflow readiness gate

## Post-g08 audit-remediation contracts

- `072` real plugin hosting, discovery, and sandbox execution
- `073` native backend device truth and CoreAudio implementation
- `074` shared host/runtime execution and recovery unification
- `075` runtime public interface decomposition and internal assembly boundary
- `076` low-level correctness, safety, and protocol hardening
- `077` DSP fidelity, semantic calibration, and analysis realism
- `078` rhythm continuity, failure containment, and policy normalization
- `079` interactive demo binary and crate-capability proof
- `080` production readiness grade and generation release gate
- `081` operator-visible interactive demo and low-dependency UI

## Working Rule

Roadmap milestones should cite the narrowest governing contract family they
depend on, then add a new contract only when the intended seam is not already
frozen elsewhere in this index.

## Next Task

Run `g10.029` Batch 29.7F under contract `082`, Rule 31H. Attribute the residual
between coefficient relation, edge constraint, overlap, and boundaries before
another stereo topology change. Keep independent listening, dynamic ratio,
routing, and promotion closed.
