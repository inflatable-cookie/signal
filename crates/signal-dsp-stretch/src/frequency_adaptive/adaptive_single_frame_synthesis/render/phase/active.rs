use rustfft::num_complex::Complex64;

use super::{wrap, ActiveOwner, PhaseState, Result, Trace};
use crate::frequency_adaptive::adaptive_single_frame_synthesis::render::{Frame, FFT_FRAMES};

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
    let maximum = tracking
        .iter()
        .take(FFT_FRAMES / 2 + 1)
        .map(Complex64::norm_sqr)
        .fold(0.0_f64, f64::max);
    let mut active_peaks = peaks.to_vec();
    if active_peaks.is_empty() {
        if let Some(prior) = state
            .active
            .iter()
            .min_by_key(|owner| owner.bin.abs_diff(trace_bin))
        {
            if tracking[prior.bin].norm_sqr() >= maximum * 1.0e-2 {
                active_peaks.push(prior.bin);
            }
        }
    }
    active_peaks.sort_unstable();
    let assignments = ordered_matches(&state.active, &active_peaks);
    let matched = assignments.iter().filter(|owner| owner.is_some()).count();
    let births = active_peaks.len() - matched;
    let retirements = state.active.len().saturating_sub(matched);
    let analysis = spectrum
        .iter()
        .take(FFT_FRAMES / 2 + 1)
        .map(|value| value.arg())
        .collect::<Vec<_>>();
    let tracking_analysis = tracking
        .iter()
        .take(FFT_FRAMES / 2 + 1)
        .map(|value| value.arg())
        .collect::<Vec<_>>();
    let mut owners = Vec::with_capacity(active_peaks.len());
    for (peak, prior) in active_peaks.iter().copied().zip(&assignments) {
        let owner = if let Some(prior) = prior.map(|index| state.active[index]) {
            let phase_delta = wrap(tracking_analysis[prior.bin] - prior.analysis);
            let turns =
                ((prior.frequency * source_hop - phase_delta) / std::f64::consts::TAU).round();
            let frequency = (phase_delta + turns * std::f64::consts::TAU) / source_hop;
            let transported = prior.synthesis + frequency * output_hop;
            ActiveOwner {
                bin: peak,
                analysis: tracking_analysis[peak],
                synthesis: if (output_hop - source_hop).abs() <= f64::EPSILON {
                    analysis[peak]
                } else {
                    transported + wrap(analysis[peak] - analysis[prior.bin])
                },
                frequency,
            }
        } else {
            ActiveOwner {
                bin: peak,
                analysis: tracking_analysis[peak],
                synthesis: analysis[peak],
                frequency: std::f64::consts::TAU * peak as f64 / FFT_FRAMES as f64,
            }
        };
        owners.push(owner);
    }
    let event = frame.source >= 0 && events.contains(&(frame.source as usize));
    let mut event_changes = 0;
    if event {
        for owner in &mut owners {
            event_changes +=
                usize::from(wrap(owner.synthesis - analysis[owner.bin]).abs() > 1.0e-12);
            owner.synthesis = analysis[owner.bin];
        }
    }
    let mut region_assignments = 0;
    if !owners.is_empty() {
        for bin in 0..=FFT_FRAMES / 2 {
            let owner = nearest_owner(bin, &owners);
            let phase = owner.synthesis + wrap(analysis[bin] - analysis[owner.bin]);
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
    let final_advance = transported_advance;
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
            final_advance,
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

fn ordered_matches(previous: &[ActiveOwner], peaks: &[usize]) -> Vec<Option<usize>> {
    const MAX_DISTANCE: usize = 8;
    let mut result = vec![None; peaks.len()];
    let mut used = vec![false; previous.len()];
    for (current_index, current) in peaks.iter().copied().enumerate() {
        if let Ok(candidate) = previous.binary_search_by_key(&current, |owner| owner.bin) {
            result[current_index] = Some(candidate);
            used[candidate] = true;
        }
    }
    for current_index in 0..peaks.len() {
        if result[current_index].is_some() {
            continue;
        }
        let lower = result[..current_index]
            .iter()
            .rev()
            .flatten()
            .next()
            .map(|index| index + 1)
            .unwrap_or(0);
        let upper = result[current_index + 1..]
            .iter()
            .flatten()
            .next()
            .copied()
            .unwrap_or(previous.len());
        let candidate = (lower..upper)
            .filter(|index| !used[*index])
            .filter(|index| previous[*index].bin.abs_diff(peaks[current_index]) <= MAX_DISTANCE)
            .min_by_key(|index| previous[*index].bin.abs_diff(peaks[current_index]));
        if let Some(candidate) = candidate {
            result[current_index] = Some(candidate);
            used[candidate] = true;
        }
    }
    result
}

fn nearest_owner(bin: usize, owners: &[ActiveOwner]) -> ActiveOwner {
    owners
        .iter()
        .copied()
        .min_by_key(|owner| owner.bin.abs_diff(bin))
        .expect("active spectral owner")
}

fn active_state_hash(owners: &[ActiveOwner]) -> u64 {
    let mut state = 0xcbf2_9ce4_8422_2325_u64;
    for owner in owners {
        for value in [
            owner.bin as u64,
            owner.analysis.to_bits(),
            owner.synthesis.to_bits(),
            owner.frequency.to_bits(),
        ] {
            state = (state ^ value).wrapping_mul(0x100_0000_01b3);
        }
    }
    state
}
