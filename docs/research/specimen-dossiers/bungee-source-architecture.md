# Bungee Source Architecture

Status: reviewed
Specimen: Bungee `2.4.24`
Owner: dsp
Last updated: 2026-07-17
Scope: exact source at `746833f68a574d997ec50443e7cfd2d37b026302`

## Why This Specimen Matters

Bungee is an independently implemented whole-kernel peak-region phase vocoder.
It is not a Rubber Band derivative. Its public Basic engine supports mono and
multichannel audio, nonuniform source movement, dynamic speed, reverse motion,
reset, and bounded grain processing.

The source is MPL-2.0. Signal uses it as architecture evidence only. No source
expression enters Signal. MIT AudioTSM remains the permissive implementation
control for the basic Laroche-Dolson identity-locking operator.

## Whole-Kernel Topology

Each Bungee grain:

1. derives an exact rounded analysis hop while carrying fractional position
   error
2. windows and transforms every channel on one fixed grid
3. sums channel coefficients to obtain one shared analysis phase and energy
   field
4. partitions the spectrum into peak-owned regions
5. advances one phase rotation per region from previous synthesis state
6. applies that same rotation to every coefficient and channel in the region
7. inverse-transforms each channel and overlap-adds on one output timeline

This is not peak state overlaid on another phase field. Peak regions own the
complete active phase operation. Applying one rotation to all channels keeps
their current complex relation intact without a post-render image repair.

The public engine also has explicit discontinuity reset. Its transient heuristic
reduces the number of competing partial regions when new peak energy rises. The
exact rule and constant are implementation choices, not Signal policy.

## Strong Evidence

- peak ownership is a complete synthesis operator, not a local patch
- one common region rotation preserves peer phase and magnitude relation
- exact position-error carry supports dynamic and nonintegral source movement
- the same phase topology works in realtime grain processing
- analysis, phase state, inverse transform, and overlap synthesis are one
  kernel boundary

## Weaknesses And Exclusions

- complex channel summation can cancel opposite-polarity or delayed stereo
  content before peak selection
- shared phase is derived from the channel sum rather than a cancellation-safe
  dominant-channel decision
- the transient partial suppression contains an explicitly labelled heuristic
  constant
- the Basic engine does not expose Rubber Band-style ordinary, unlocked,
  material-guided, or frequency-bounded linked states
- MPL source expression is excluded from Signal's clean-room implementation

Signal therefore adopts neither the channel sum nor the transient rule. Dorran,
Lawlor, and Coyle's greater-magnitude channel ownership supplies the safer
multichannel control. Laroche-Dolson and Ottosen-Dörfler supply independent
peak-region and predecessor-trajectory definitions.

## Signal Translation Boundary

The useful invariant is `common region rotation`:

- form a cancellation-safe joint energy map from per-channel energies
- select one deterministic owner channel at each joint peak
- advance the peak from complete predecessor synthesis state
- derive every channel and bin in the region by applying one common rotation
  to its current analysis coefficient
- reset the complete region when continuity is unavailable

This preserves current within-channel vertical structure and current
interchannel relation simultaneously. It differs from Batch 29.7Q because the
region-locked phase vocoder replaces the complete weighted-predictor phase
kernel rather than switching owners inside it.

## Source Inventory

| Source | Type | Revision | Use |
| --- | --- | --- | --- |
| [Bungee repository](https://github.com/bungee-audio-stretch/bungee/tree/746833f68a574d997ec50443e7cfd2d37b026302) | MPL-2.0 source | `2.4.24` | whole-kernel and realtime architecture |
| [grain analysis](https://github.com/bungee-audio-stretch/bungee/blob/746833f68a574d997ec50443e7cfd2d37b026302/src/Stretcher.cpp) | MPL-2.0 source | pinned | shared field, region construction, channel application |
| [phase synthesis](https://github.com/bungee-audio-stretch/bungee/blob/746833f68a574d997ec50443e7cfd2d37b026302/src/Synthesis.cpp) | MPL-2.0 source | pinned | temporal peak advance and common region rotation |
| [partial regions](https://github.com/bungee-audio-stretch/bungee/blob/746833f68a574d997ec50443e7cfd2d37b026302/src/Partials.cpp) | MPL-2.0 source | pinned | region and transient architecture evidence |
| [AudioTSM phase vocoder](https://github.com/Muges/audiotsm/blob/cf3875842bda44d81930c44b008937e72109ae9f/audiotsm/phasevocoder.py) | MIT source | `cf387584` | permissive identity-locking control |

## Next Task

Batch 29.7T proves the source-independent common-region-rotation invariant is
materially stronger on stereo but leaves tone-local failures. Batch 29.7U
localizes those failures to overlap of finite-support boundary frames, not the
interior common-region rotation. Batch 29.7V rejects the deterministic
finite-support reset because it trades boundary failures across rows. Bungee
does not justify another derived intervention here. Run Batch 29.7W as a
complete material-state architecture review, with Bungee expression, channel
summation, constants, and transient heuristic kept out of Signal.
