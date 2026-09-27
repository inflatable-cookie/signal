# Glossary: Plain-English Guide to Signal Docs

The Signal docs were built for a mix of human readers and automation, so they
use a lot of shorthand. This page translates the shorthand. When a term has a
canonical definition elsewhere, this page points you there instead of
rewriting it.

## The cast of characters

| Term | What it is |
| --- | --- |
| **Signal** | This repository. The shared audio library and runtime: DSP kernels, analysis, graph execution, plugin discovery. The reusable layer beneath other products. |
| **Loophole** | The DAW (digital audio workstation). Signal's primary consumer. |
| **Pulse** | Loophole's project/session state and editing layer. It owns what a song *is*, not how audio is *processed*. |
| **Aura / Spark** | Loophole's UI surfaces. |
| **Finch** | A companion app (review workflow, sidecar handling, library behavior). Consumes Signal crates. |
| **Chorus** | The cross-repo specs repository (upstream of this one). Signal's IPC/message contracts align to its specs. |
| **Inflatable Cookie** | The product owner of Loophole and the Loophole ecosystem. |
| **consumer** | Any downstream repository that depends on Signal crates (Loophole, Finch, etc.). See `docs/reference/consuming-signal.md`. |

## Documentation and process terms

| Term | What it is |
| --- | --- |
| **Northstar** | The repository keeps current knowledge in `docs/knowledge/`; Queue keeps the plan, leads, tasks, briefs, status, review, closeout, and outcomes. |
| **Vision** | The long-horizon purpose and constraints in `docs/knowledge/vision.md`. |
| **Architecture** | Current crates, boundaries, and invariants in `docs/knowledge/architecture/`. |
| **Contract** | A durable boundary decision in `docs/knowledge/contracts/`. See the contract index. |
| **Lane** | A stable plan area with a key that Queue can use for related tasks. |
| **Queue brief** | A bounded task held by Queue, with scope, acceptance, and stop conditions. |
| **Rule 5 / Rule 11** | Evidence rules in Contract `084`: listening admission and evidence-identity closure. |
| **Admission / admitted** | Gated acceptance of a candidate implementation. Private admission stays internal; public admission ships an API. |
| **Checkpoint** | A frozen candidate implementation plus evidence, referenced by hash. |
| **Receipt** | Evidence of which product gates ran and their results. |
| **Evidence-invalid** | A receipt claiming more than its proof shows. |
| **Promotion** | A research finding becoming binding knowledge in architecture or contracts. |

## Audio and DSP terms

| Term | What it is |
| --- | --- |
| **DSP** | Digital signal processing: any math applied to audio samples. |
| **Audio thread / realtime path** | The thread that must deliver samples to the device on time, every time. |
| **RT safety** | The invariant that the audio thread never allocates memory, blocks, or takes locks. Everything that does those things lives off the audio thread. |
| **Alloc-free** | Guarantees no heap allocation in a given code path. |
| **Render plane** | Signal's realtime output executor: it runs pre-compiled plans, declick envelopes, and polyphase resampling on the audio thread. |
| **Kernel** | A single reusable DSP routine (e.g. a biquad filter, a smoother, a delay). |
| **FFT / STFT** | Fast Fourier Transform / Short-Time Fourier Transform: moving audio between time domain and frequency domain in short windows. |
| **Phase vocoder** | A frequency-domain technique for changing speed without changing pitch (and vice versa). `OfflineHighQuality` is a phase-vocoder baseline. |
| **Polyphase resampling** | A high-quality sample-rate conversion technique used in the realtime path. |
| **Onset** | The moment a note or transient begins. |
| **Chroma** | The pitch class content of audio (which of the 12 notes are present). Used for key detection. |
| **Key detection** | Determining the musical key of a piece from chroma and profile correlation. |
| **LUFS / true peak / LRA / BS.1770** | Loudness measurement standards: LUFS is loudness units full scale; true peak is the inter-sample peak; LRA is loudness range; BS.1770 is the ITU standard that defines them. |
| **Warp** | Stretching audio to follow a tempo map (DAW-style elastic audio). |
| **Varispeed** | Changing speed and pitch together, like a tape machine. |
| **Repitch** | Signal's realtime-safe varispeed implementation. |
| **Corpus** | A fixed, frozen set of test audio used for comparison evidence. |
| **Comparator** | An external tool used as a benchmark for Signal's DSP (e.g. PaulXStretch, Rubber Band). Clean-room reference, never a dependency. |
| **Concealed mono listening** | A blind A/B listening test on mono audio where the listener does not know which side is which. |
| **ULP** | Unit in the last place: the smallest step a floating-point value can take. One-ULP differences are the last word in bit-exactness debates. |

## Stretch (time-stretch & pitch-shift) terms

| Term | What it is |
| --- | --- |
| **Time-stretch** | Changing the speed (length) of audio without changing its pitch. |
| **Pitch-shift** | Changing pitch without changing speed. |
| **Transparent** | The "faithful" stretch character: the renderer should sound like the source, just at a different length. Signal's transparent route is `OfflineHighQuality`. |
| **CreativeStretch** | The separate public creative stretch API. Exposes exact `4x`, `8x`, `16x` "neutral Dream" with `space` as its only control. |
| **Dream** | A creative stretch character: a diffuse, textured, "dreamy" sound. |
| **Cyclic** | A creative stretch character built on repeated cycles of source audio (exact `2x`, `4x`, `8x`). |
| **RenewalSpectral, LayeredCloud, LinkedStnNoiseMorph, …** | Rejected research candidates for creative stretch. They are archived in `docs/research/master-index.md` and the architecture briefs; none of their DSP is in `main`. |
| **Stretch candidate / renderer brief** | A complete written specification for one candidate stretch renderer, frozen before any implementation. |
| **Seam** | The boundary between chunks in a chunked render. Seam artifacts are a classic stretch quality problem. |

## Plugin and runtime terms

| Term | What it is |
| --- | --- |
| **CLAP / VST3 / AU / LV2** | The four plugin formats Signal can discover. |
| **Sandbox** | The out-of-process container that runs plugin code. Plugins are treated as untrusted; sandboxing is the default containment. |
| **Broker** | The process that owns sandboxed plugin instances and their shared-memory transport. |
| **Host** | The application embedding the runtime (e.g. Pulse inside Loophole). |
| **Control plane** | The non-realtime side of the runtime: lifecycle, graph planning, diagnostics. |
| **Graph** | The connected model of nodes and routes that the runtime plans and executes. |

## Reading numbers

| You see | Meaning |
| --- | --- |
| `g11.002` | Generation 11, task 002 |
| `Batch 31.66` | Historical batch 66 inside old milestone `g10.031` (Git history) |
| `Contract 084` | Contract number 084 in `docs/knowledge/contracts/` |
| `Y01`…`Y09`, `S01`…`S17` | Named evidence gates (synthetic pitch, structural proof, etc.) defined inside a contract or brief |
| `4x` / `8x` / `16x` | Stretch ratios: output is 4, 8, or 16 times the source length |
