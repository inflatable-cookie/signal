# g10.029 Phase-Gradient Reassessment

Date: 2026-07-10
Status: complete
Roadmap: `g10.029`
Contract: `082`

## Trigger

The additive H/R/P candidate passed separation and transient-crest checks but
failed timing, integrity, transient-replica, static-spectrum, and combined
gates. The next direction had to change synthesis family rather than tune the
component split.

## Screen

- WSOLA retains one dominant local waveform period and carries known
  polyphonic and transient failure modes. Transient-preserving variants also
  redistribute local time, which Signal already rejected.
- sinusoidal/residual synthesis again separates components and remains weak on
  broadband attacks and noise
- adaptive-resolution nonstationary transforms remain credible later, but the
  surveyed TSM path depends on onset detection and local unity-rate spans with
  compensation
- full phase-gradient integration uses one whole-band STFT and one global map
  while restoring the frequency-direction phase information omitted by the
  classical and identity-locked phase-vocoder paths

## Decision

Open a fixed-resolution full phase-gradient proof. Freeze the published
`4092`-sample Hann window, `8192` FFT, fixed `1024` synthesis hop,
nearest-integer ratio-derived analysis hop, centered phase derivatives,
trapezoidal propagation, magnitude-prioritized heap, and relative tolerance
`1e-6`.

Signal adds deterministic boundaries: nonredundant-spectrum integration with
explicit conjugate mirroring, stable heap tie breaks, analyzed phase for the
first frame, and analyzed phase below tolerance. No peak tracker, onset
detector, phase reset, component split, local timing compensation, adaptive
resolution, or parameter sweep enters the proof.

Batch 29.6F proves the kernel on synthetic controls. Batch 29.6G owns the full
mono corpus gate. Linked stereo and product work remain closed.

## Primary Sources

- [Prusa and Holighaus, Phase Vocoder Done Right](https://arxiv.org/abs/2202.07382)
- [Driedger and Muller, A Review of Time-Scale Modification of Music Signals](https://www.mdpi.com/2076-3417/6/2/57)
- [Roelands and Verhelst, Waveform Similarity Based Overlap-Add](https://www.isca-archive.org/eurospeech_1993/roelands93_eurospeech.html)
- [Balazs et al., Nonstationary Gabor Frames](https://arxiv.org/abs/1112.5262)
