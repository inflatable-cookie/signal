use rustfft::num_complex::Complex64;

use super::{active_state_hash, nearest_owner, ordered_matches};
use crate::frequency_adaptive::adaptive_single_frame_synthesis::render::{Frame, FFT_FRAMES};

use super::super::{wrap, ActiveOwner, PhaseState, Result, Trace};

pub(super) fn transport(
    spectrum: &mut [Complex64],
    tracking: &[Complex64],
    frame: &Frame,
    state: &mut PhaseState,
    events: &[usize],
    peaks: &[usize],
    trace_bin: usize,
) -> Result {
    let source_hop = state
        .source
        .map(|previous| (frame.source - previous) as f64)
        .unwrap_or(0.0);
    let output_hop = state
        .output
        .map(|previous| (frame.output - previous) as f64)
        .unwrap_or(0.0);
    let tracking_analysis = tracking
        .iter()
        .take(FFT_FRAMES / 2 + 1)
        .map(|value| value.arg())
        .collect::<Vec<_>>();
    let native_analysis = spectrum
        .iter()
        .take(spectrum.len() / 2 + 1)
        .map(|value| value.arg())
        .collect::<Vec<_>>();
    let assignments = ordered_matches(&state.active, peaks);
    let matched = assignments.iter().filter(|owner| owner.is_some()).count();
    let births = peaks.len() - matched;
    let retirements = state.active.len().saturating_sub(matched);
    let identity_hop = (output_hop - source_hop).abs() <= f64::EPSILON;
    let mut owners = Vec::with_capacity(peaks.len());
    for (peak, prior) in peaks.iter().copied().zip(&assignments) {
        let native_bin = native_bin(peak, spectrum.len());
        let owner = if let Some(prior) = prior.map(|index| state.active[index]) {
            let phase_delta = wrap(tracking_analysis[peak] - prior.analysis);
            let turns = if source_hop > 0.0 {
                ((prior.frequency * source_hop - phase_delta) / std::f64::consts::TAU).round()
            } else {
                0.0
            };
            let frequency = if source_hop > 0.0 {
                (phase_delta + turns * std::f64::consts::TAU) / source_hop
            } else {
                prior.frequency
            };
            ActiveOwner {
                bin: peak,
                analysis: tracking_analysis[peak],
                synthesis: if identity_hop {
                    native_analysis[native_bin]
                } else {
                    prior.synthesis + frequency * output_hop
                },
                frequency,
            }
        } else {
            let frequency = std::f64::consts::TAU * peak as f64 / FFT_FRAMES as f64;
            ActiveOwner {
                bin: peak,
                analysis: tracking_analysis[peak],
                synthesis: native_analysis[native_bin],
                frequency,
            }
        };
        owners.push(owner);
    }
    let event = frame.source >= 0 && events.contains(&(frame.source as usize));
    let mut event_changes = 0;
    if event {
        for owner in &mut owners {
            let native_bin = native_bin(owner.bin, spectrum.len());
            let reset = native_analysis[native_bin];
            event_changes += usize::from(wrap(owner.synthesis - reset).abs() > 1.0e-12);
            owner.synthesis = reset;
        }
    }
    let native_owners = owners
        .iter()
        .copied()
        .map(|owner| NativeOwner {
            owner,
            bin: native_bin(owner.bin, spectrum.len()),
            synthesis: owner.synthesis,
        })
        .collect::<Vec<_>>();
    let mut region_assignments = 0;
    if !native_owners.is_empty() {
        for bin in 1..spectrum.len() / 2 {
            let owner = nearest_native_owner(bin, spectrum.len(), &native_owners);
            let phase = owner.synthesis + wrap(native_analysis[bin] - native_analysis[owner.bin]);
            spectrum[bin] = Complex64::from_polar(spectrum[bin].norm(), phase);
            region_assignments += 1;
        }
    }
    let trace_owner = (!owners.is_empty()).then(|| nearest_owner(trace_bin, &owners));
    let trace_match = trace_owner.and_then(|owner| {
        owners
            .iter()
            .position(|candidate| candidate.bin == owner.bin)
            .and_then(|index| assignments[index].map(|prior| (owner, state.active[prior])))
    });
    let trace_frequency = trace_owner.map(|owner| owner.frequency).unwrap_or(0.0);
    let (prior_bin, analysis_advance, transported_advance) = trace_match
        .map(|(owner, prior)| {
            (
                prior.bin,
                wrap(owner.analysis - prior.analysis),
                wrap(owner.synthesis - prior.synthesis),
            )
        })
        .unwrap_or((trace_bin, 0.0, 0.0));
    let active_state_hash = active_state_hash(&owners);
    state.source = Some(frame.source);
    state.output = Some(frame.output);
    state.dominant = Some(trace_bin);
    state.active = owners;
    Result {
        event_changes,
        vertical_changes: 0,
        initialization: births,
        trace: Trace {
            source_hop,
            output_hop,
            bin: trace_bin,
            prior_bin,
            peak_owner: trace_owner.map(|owner| owner.bin).unwrap_or(trace_bin),
            analysis_advance,
            estimated_frequency: trace_frequency,
            transported_advance,
            final_advance: transported_advance,
            event_assignment: event,
            vertical_assignment: false,
            owner_births: births,
            owner_matches: matched,
            owner_retirements: retirements,
            region_assignments,
            active_state_hash,
            trace_owner_matched: trace_match.is_some(),
        },
    }
}

#[derive(Clone, Copy)]
struct NativeOwner {
    owner: ActiveOwner,
    bin: usize,
    synthesis: f64,
}

fn native_bin(tracking_bin: usize, fft_frames: usize) -> usize {
    ((tracking_bin * fft_frames + FFT_FRAMES / 2) / FFT_FRAMES).clamp(1, fft_frames / 2 - 1)
}

fn nearest_native_owner(bin: usize, fft_frames: usize, owners: &[NativeOwner]) -> NativeOwner {
    owners
        .iter()
        .copied()
        .min_by_key(|owner| (bin * FFT_FRAMES).abs_diff(owner.owner.bin * fft_frames))
        .expect("native active spectral owner")
}
