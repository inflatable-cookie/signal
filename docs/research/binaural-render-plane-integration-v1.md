# Binaural / HRTF Render-Plane Integration — v1

Status: **Draft — design decision needed (per-voice model)**
Date: 2026-07-18
Downstream driver: Jetstream (game engine, `../jetstream/crates/jetstream-audio`)
Landed alongside this memo: `signal_dsp::BinauralConvolver` (item 1 below).

## Context

Jetstream ships a working binaural chain (KEMAR HRIR set, per-voice dual-ear
convolution, crossfaded direction changes, distance attenuation) — but it runs
entirely CPU-side *outside* the render plane. Its own maturity register names
the target: "HRTF binaural + per-voice occlusion-filter as
`signal-render-plane` stages (per-voice → before the mix sum)". Today the
render plane only performs Source→Sum→Output gain/pan mixing; every real DSP
effect downstream (HRTF, reverb, occlusion) wraps the executor externally.

Signal already has most of the substrate:

- `signal-dsp` convolution kernels (`FirConvolver`, and the partitioned pair
  in `signal-dsp-spectral`), all RT-safe.
- The `PluginBlockProcessor` hook on `Sum` stages.
- The g13.018 live-event mailbox (bounded lock-free MPMC) — the RT-safe
  game-thread→audio-thread transport.

## What landed now

**`signal_dsp::BinauralConvolver`** — the HRTF rendering kernel: dual-ear FIR
over a shared mono input, double-buffered ear pairs (the idle pair keeps a
warm history), linear constant-sum crossfade on response swaps
(deliberately not equal-power: the branches carry the same input through
neighbouring HRIRs — highly correlated — so constant-sum is flat for
near-identical responses and never overshoots the ceiling), mid-fade
retargeting, snap-set for a voice's first direction. RT-safe after
construction. Dataset policy (which HRIR for which azimuth/elevation, grid
lookup, mirroring) deliberately stays with callers.

## The ladder (remaining, ordered)

2. **Per-voice model in the render plane** — the design decision this memo
   exists to force. `PluginBlockProcessor` is one-per-`Sum`-stage; binaural
   needs N per-voice convolvers *before* the mix sum, and voice spawn today
   implies a full-plan recompile — wrong shape for fire-and-forget game SFX.
   Options (from Jetstream's assessment, decision belongs here):
   - **A. Voice-pool wrapper stays downstream** — consumers keep their own
     voice mixers feeding a Source; signal stays voice-agnostic. Cheapest,
     but every consumer re-solves voices and none of signal's plan-level
     guarantees (automation, events, metering) reach individual voices.
   - **B. Hybrid** — signal grows a `VoiceBank` *processor* (one Sum-stage
     plugin hosting N slots, each slot = mono ring + `BinauralConvolver` +
     gain), addressed through live events. No plan-shape change, no
     recompile on spawn; voices are processor-internal.
   - **C. Evolve the plan** — first-class cheap per-voice stages (spawn
     without recompile). Most general, most invasive; touches compile,
     scheduling, and the stage-id vocabulary.
   Recommendation: **B** — it reuses the existing processor + live-event
   machinery, keeps plan compilation untouched, and A's downside (signal
   never learns about voices) is exactly what blocks the rest of this
   ladder. C can still come later; B does not foreclose it.
3. **Binaural stage processor** — ship the `VoiceBank`/binaural processor in
   signal-render-plane (or signal-dsp behind a feature) so downstreams stop
   hand-rolling mixers. Same move promotes reverb/occlusion from CPU
   post-processes into real stages.
4. **Direction/pose event vocabulary** — `RenderPluginEventKind` is
   MIDI-shaped (NoteOn/CC/expression); nothing carries per-voice
   azimuth/elevation/HRIR-select or listener pose. Extend the enum (or add a
   per-voice parameter ring). The transport exists; the payload type does
   not. Wire format must stay `Copy` + bounded for the mailbox.
5. **Fire-and-forget source ergonomics** — mono one-shot sources that fit
   game SFX without timeline clips or plan recompile (falls out of B's
   voice slots if that's the chosen shape).

## Division of labor (unchanged)

Signal owns: convolution kernels, the voice-hosting processor, event
vocabulary, plan/scheduling. Downstream (Jetstream) owns: HRIR datasets +
grid selection, listener/emitter math, voice budget/stealing policy,
physics-audio coupling, distance/attenuation game rules.

## Validation expectations

Kernel-level: unit tests landed with the convolver (flat same-response fade,
monotone smooth swaps, ITD preservation, block/sample parity, mid-fade
retarget). Stage-level (once 3 lands): offline render parity against the
downstream CPU mixer on identical inputs, then live-posture soak with event
floods (drop-count telemetry already exists on the mailbox).
