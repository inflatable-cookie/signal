# Signal Quick Start: Hear It, Analyze It, Stretch It

Signal is a realtime audio library: DSP kernels, analysis, graph execution,
plugin discovery, and a runtime. This page gets you from a clean checkout to
sound in under five minutes, using the examples that ship in the repo.

Every snippet here is real code from the repository — you can run them as
written. Terms in *italics* are in the [glossary](./glossary.md).

## Prerequisites

- A Rust toolchain (the workspace pins its version in `rust-toolchain.toml`;
  consumers need Rust `1.95`+, see [Consuming Signal](./consuming-signal.md)).
- No audio hardware is required except for the first two examples, which play
  to your default output device. If none is available, those examples skip or
  fail loudly at stream open — the analysis and stretch examples run anywhere.

```bash
git clone git@github.com:inflatable-cookie/signal.git
cd signal
cargo build --workspace        # first build; grab a coffee
```

## 1. First sound: a 440 Hz tone

The shortest path from zero to audio is the `tone` example in
`signal-hardware-cpal`. It opens the default output device at 48 kHz and plays
a two-second 440 Hz sine.

```bash
cargo run -p signal-hardware-cpal --example tone
```

What you should see:

```
playing 440 Hz for 2s on default device (48000 Hz, 2 ch, state Running)
stopped
```

The whole program (`crates/signal-hardware-cpal/examples/tone.rs`) is this:

```rust
use std::f32::consts::TAU;

use signal_hardware::{OutputStreamBackend, OutputStreamSpec};
use signal_hardware_cpal::CpalOutputBackend;

fn main() {
    let sample_rate_hz = 48_000u32;
    let channels = 2u16;
    let mut phase = 0.0f32;
    let phase_step = 440.0 * TAU / sample_rate_hz as f32;

    let backend = CpalOutputBackend::new();
    let stream = backend
        .open_output_stream(
            OutputStreamSpec {
                sample_rate_hz,
                channels,
                buffer_frames: None,
            },
            Box::new(move |frames| {
                for frame in frames.chunks_mut(channels as usize) {
                    let sample = (phase).sin() * 0.2;
                    phase = (phase + phase_step) % TAU;
                    for slot in frame {
                        *slot = sample;
                    }
                }
            }),
        )
        .expect("open output stream");

    println!(
        "playing 440 Hz for 2s on default device ({} Hz, {} ch, state {:?})",
        stream.sample_rate_hz(),
        stream.channels(),
        stream.state()
    );
    std::thread::sleep(std::time::Duration::from_secs(2));
    drop(stream);
    println!("stopped");
}
```

What's happening, in plain words:

- `CpalOutputBackend` is the concrete hardware backend (CPAL = the
  cross-platform audio library underneath). The *audio thread* is the thread
  that calls your closure — it must deliver samples on time, every time.
- Your closure receives a buffer of interleaved frames and must fill it. That
  closure is the whole "engine": this example synthesizes a sine with nothing
  but a running phase variable.
- The one hard rule of this codebase: the callback must never allocate, block,
  or take locks. This example is allocation-free by construction.

This is the realtime path — for offline, whole-buffer work, keep reading.

## 2. A graph plan on the render plane

The next example up is `render_soak` in `signal-render-plane`. It builds a
graph-shaped plan — two tone lanes panned hard left and right, plus an
instrument lane and a live-input lane, all mixed through a `Sum` stage to the
output — and runs it inside the real audio callback while a counting allocator
proves the callback never allocates.

```bash
cargo run -p signal-render-plane --example render_soak
```

The key shapes, from `crates/signal-render-plane/examples/render_soak.rs`:

```rust
let (mut controller, mut executor) = render_plane();

let summed_plan = RenderPlanSpec {
    sample_rate_hz,
    master_gain: 0.5,
    master_limiter: None,
    stages: vec![
        tone_lane(1, 0.4, 440.0),          // lane A: 440 Hz, hard left
        tone_lane(2, 0.25, 660.0),         // lane B: 660 Hz, hard right
        // ... an instrument lane and a live-input monitor lane ...
        RenderStageSpec {
            kind: RenderStageKind::Sum,    // the mix point
            inputs: vec![
                RenderEdgeSpec {
                    source_stage_id: 1,
                    gain: 1.0,
                    matrix: Some(equal_power_pan_matrix(-1.0).to_vec()),
                },
                RenderEdgeSpec {
                    source_stage_id: 2,
                    gain: 1.0,
                    matrix: Some(equal_power_pan_matrix(1.0).to_vec()),
                },
            ],
            // ...
        },
        RenderStageSpec { kind: RenderStageKind::Output, /* ... */ },
    ],
};

controller.install_plan(&summed_plan).expect("install plan");
controller.set_playing(true).expect("play");
```

Plain words:

- A *plan* describes what should play: sources (clips, notes, test tones),
  stages, and the edges that route audio between them. It is data, built on
  the control side.
- The `controller` installs plans and drives transport (play, stop, seek,
  plan swap). The `executor` runs inside the audio callback — it is the part
  that must stay alloc-free, which is what the soak's counting allocator
  proves at the end:

```
callback allocations: 0, deallocations: 0
soak passed: audible, summed, plugin-inserted, transport-gated, zero-alloc callback
```

## 3. Analysis: measure a file

Analysis crates share one pattern: construct an analyzer, call
`analyze(&audio)`, read the result. Input uses Signal's `AudioBuffer` type
from `signal-primitives`.

### Loudness (LUFS, true peak, range)

```bash
cargo run -p signal-analysis-loudness --example offline_loudness_demo
```

This synthesizes a 1 kHz sine and measures it (the source is 60 lines —
`crates/signal-analysis-loudness/examples/offline_loudness_demo.rs`). The
pattern:

```rust
use signal_analysis::AnalysisStage;
use signal_analysis_loudness::{LoudnessMeter, LoudnessMeterConfig};
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

let audio: AudioBuffer = /* your mono or stereo samples */;

let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
let result = meter.analyze(&audio);

println!("integrated_lufs={:.3}", result.integrated_lufs);
println!("lra_lu={:.3}", result.loudness_range_lu);
println!("true_peak_dbtp={:.3}", result.true_peak_dbtp);
```

### Tempo and beats

The rhythm crate can read a WAV file directly:

```bash
cargo run -p signal-analysis-rhythm --example file_rhythm_probe -- path/to/your.wav
```

It prints BPM, confidence, tempo candidates, and beat positions. The same
`analyze` shape applies:

```rust
use signal_analysis::AnalysisStage;
use signal_analysis_rhythm::{BeatTracker, BeatTrackerConfig};

let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
let result = tracker.analyze(&audio);

println!("bpm={:.5}", result.bpm);
println!("beats={}", result.beat_positions_seconds.len());
```

Other analyzers follow the same pattern: `signal-analysis-tonal` (key
detection, chroma) and `signal-analysis-embed` (descriptor embedding) both
ship `offline_*` examples. See the
[DSP and analysis feature reference](../architecture/dsp-analysis-feature-reference.md)
for the full surface.

## 4. Creative stretch: `Dream` and `Cyclic`

The stretch crate ships a public offline API for two creative characters:
`Dream` (smooth, musical smear; exact output lengths from `4x` to `16x`) and
`Cyclic` (commanded repetition; exact output lengths from `2x` to `8x`).
The request specifies an exact target frame count — the output is
byte-deterministic for one request, and there is no fallback: unsupported
targets return an error.

```rust
use signal_dsp_stretch::{
    render_creative_stretch, CreativeStretchCharacter, CreativeStretchRequest,
};
use signal_primitives::{Sample, SampleRate};

fn dream_16x(input: &[Sample], sample_rate: SampleRate) -> Vec<Sample> {
    let target_frames = 16 * input.len(); // exact: Dream accepts 4N..=16N
    let request = CreativeStretchRequest::new(
        input,
        1,                                   // mono
        sample_rate,
        target_frames,
        CreativeStretchCharacter::Dream,
    )
    .with_space(0.5);                       // 0..=1; default 0.5

    render_creative_stretch(request).expect("4x..16x Dream targets are valid")
}
```

The rules in one breath:

- `Dream`: `space` in `0..=1` is its only control. `Cyclic`: `cycle` in
  `5..=90 ms` is its only control (and `space` must stay at the default).
- Wrong-character controls, unsupported targets, and invalid values return
  typed errors — nothing is clamped, rounded, or silently rerouted.
- This is whole-buffer offline work: it allocates, so never call it from the
  audio thread.

## Next steps

- Use Signal from another repository: [Consuming Signal](./consuming-signal.md)
- Full crate inventory: [Package Map](../architecture/package-map.md)
- What the DSP/analysis crates expose today:
  [DSP and Analysis Feature Reference](../architecture/dsp-analysis-feature-reference.md)
- What the runtime/graph crates expose today:
  [Graph and Runtime Feature Reference](../architecture/graph-runtime-feature-reference.md)
- Plugin discovery (CLAP/VST3/AU/LV2): host-local example
  `cargo run -p signal-host-local --example signal_host_local_plugin_capability_scan`
- The transparent (faithful) stretch renderer: `OfflineHighQualityStretcher`,
  documented in the
  [time-stretch synthesis doc](../architecture/offline-time-stretch-synthesis.md)
