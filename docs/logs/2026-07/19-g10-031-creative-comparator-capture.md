# g10.031 Creative Comparator Capture

Date: 2026-07-19
Status: evidence captured; listening open
Scope: ignored comparator artifacts and documentation only

## Result

Captured the complete no-purchase Batch 31.2 matrix for the five retained
musical sources at `4x`, `8x`, and `16x`:

- PaulXStretch 1.6.0 default, FFT `16384`
- REAPER 7.69 `Rrreeeaaa`
- CDP 8.0 `SPECTSTR`, `d-ratio=1`, `d-rand=0.5`
- REAPER 7.69 `ReaReaRea` as a secondary cyclic control

CDP came from official tag `CDP8.0` at
`456ffe0687c8d8206f8bc4e22273587db4c0ee0a` and was built locally under
ignored `target/`. Its input was reduced `18 dB` to prevent the legacy
synthesis path clipping spectral overshoot. This gain is removed by common
listening-pack normalization.

The concealed A/B/C/D pack lives at
`target/creative-stretch-comparator-31-2/listening-pack/`. PaulXStretch and CDP
tails are cropped from the end to exact target length. All candidates are
converted to stereo float WAV at `44.1 kHz`; mono CDP output is duplicated only
for file-shape parity. Each source/ratio group uses one RMS target under a
`0.95` peak ceiling.

Validation found 15 cases and 60 candidate files. Every file has exact target
length and finite samples. Maximum inter-candidate RMS span is below `1e-9`;
maximum peak is below `0.95`.

## Availability Decision

Sloom, SoundHack `++spiralstretch`, and Ableton Texture remain supplementary,
not mandatory. Full Sloom and SoundHack require purchase; Ableton is
unavailable to the operator. Their absence cannot honestly block a no-purchase
study when PaulXStretch, Rrreeeaaa, and SPECTSTR already cover static spectral
dream, polyphase large stretch, and randomized spectral decoherence.

## Boundary

No Signal DSP, harness, fixture, report mode, public API, Loophole, or Chorus
surface changed. Batch 31.2 is not complete: target character and rejection
thresholds remain open until concealed operator listening is recorded. Stereo
remains explicitly unassessed pending an independent eligible listener.

## Next Task

Complete every non-stereo field in the 15-case `listening-notes.tsv` without
opening `listening-key.tsv`. Decode only after the character review, then
freeze one target and explicit rejection thresholds. Stop before candidate DSP.
