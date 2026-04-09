//! Bus state management for graph execution.
//!
//! This module provides utilities for managing audio buses during graph
//! execution, including bus state initialization, buffer adaptation, and
//! mixing operations.

use crate::{
    classify_channel_adaptation, GraphBusState, GraphChannelAdaptationMode,
    GraphChannelAdaptationResult, GraphNodeRenderOverride, GraphNodeSilencePolicy, GraphNodeSpec,
    GraphPreparedDispatch,
};
use signal_primitives::{
    AudioBuffer, AudioBufferConstructionError, ChannelLayout, FrameCount, SampleRate,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphChannelAdaptationFailure {
    InvalidTargetLayout {
        layout: ChannelLayout,
        source: AudioBufferConstructionError,
    },
    Unsupported {
        input: ChannelLayout,
        output: ChannelLayout,
        mode: GraphChannelAdaptationMode,
    },
}

/// Initialize a bus state from an input buffer.
pub fn seeded_bus_state(input: &AudioBuffer) -> GraphBusState {
    let mut buses = BTreeMap::new();
    let mut latencies = BTreeMap::new();
    let mut tails = BTreeMap::new();
    buses.insert("main:in".into(), input.clone());
    latencies.insert("main:in".into(), 0);
    tails.insert("main:in".into(), 0);
    GraphBusState {
        buses,
        latencies,
        tails,
        silent_source_bus_count: 0,
        failed_channel_adaptation_count: 0,
    }
}

/// Initialize a bus state from prepared dispatch buses.
pub fn prepared_bus_state(prepared: &GraphPreparedDispatch) -> GraphBusState {
    let mut buses = BTreeMap::new();
    let mut latencies = BTreeMap::new();
    let mut tails = BTreeMap::new();
    for bus in &prepared.buses {
        buses.insert(bus.bus_id.clone(), bus.buffer.clone());
        latencies.insert(bus.bus_id.clone(), bus.latency_samples);
        tails.insert(bus.bus_id.clone(), bus.tail_samples);
    }
    GraphBusState {
        buses,
        latencies,
        tails,
        silent_source_bus_count: 0,
        failed_channel_adaptation_count: 0,
    }
}

/// Get the output buffer from the bus state.
pub fn graph_output_buffer(state: &GraphBusState, fallback: &AudioBuffer) -> AudioBuffer {
    state.buses.get("main:out").cloned().unwrap_or_else(|| {
        if state.buses.len() == 1 && state.buses.contains_key("main:in") {
            fallback.clone()
        } else {
            AudioBuffer::new(
                fallback.sample_rate(),
                fallback.channels(),
                fallback.frames(),
            )
        }
    })
}

/// Compute the peak absolute value across all buses.
pub fn peak_abs_across_buses(state: &GraphBusState) -> f32 {
    state
        .buses
        .values()
        .map(|buffer| peak_abs(buffer.samples()))
        .fold(0.0_f32, f32::max)
}

/// Create a map of node render overrides by node ID.
pub fn node_render_override_map(
    node_render_overrides: &[GraphNodeRenderOverride],
) -> BTreeMap<&str, &GraphNodeRenderOverride> {
    node_render_overrides
        .iter()
        .map(|node_render_override| (node_render_override.node_id.as_str(), node_render_override))
        .collect()
}

/// Get the source buffer for a node from the bus state.
pub fn source_buffer_for_node(
    state: &GraphBusState,
    node: &GraphNodeSpec,
) -> Result<AudioBuffer, GraphChannelAdaptationFailure> {
    let source = state
        .buses
        .get(&node.buffer_contract.input.bus_id)
        .cloned()
        .unwrap_or_else(|| {
            let fallback = state
                .buses
                .get("main:in")
                .or_else(|| state.buses.values().next());
            AudioBuffer::try_new(
                fallback
                    .map(|buffer| buffer.sample_rate())
                    .unwrap_or(SampleRate(48_000)),
                node.buffer_contract.input.channels,
                fallback
                    .map(|buffer| buffer.frames())
                    .unwrap_or(FrameCount(0)),
            )
            .expect("graph source fallback should use a valid node input layout")
        });
    try_adapt_buffer_to_layout(
        &source,
        node.buffer_contract.input.channels,
        node.buffer_contract.channel_adaptation,
    )
}

/// Adapt a buffer to a target channel layout.
pub fn try_adapt_buffer_to_layout(
    input: &AudioBuffer,
    target_layout: ChannelLayout,
    mode: GraphChannelAdaptationMode,
) -> Result<AudioBuffer, GraphChannelAdaptationFailure> {
    if input.channels() == target_layout {
        return Ok(input.clone());
    }

    match classify_channel_adaptation(input.channels(), target_layout, mode) {
        GraphChannelAdaptationResult::MonoToStereo => {
            let mono = input.samples();
            let mut samples = Vec::with_capacity(mono.len().saturating_mul(2));
            for sample in mono {
                samples.push(*sample);
                samples.push(*sample);
            }
            AudioBuffer::try_from_interleaved(input.sample_rate(), target_layout, samples).map_err(
                |source| GraphChannelAdaptationFailure::InvalidTargetLayout {
                    layout: target_layout,
                    source,
                },
            )
        }
        GraphChannelAdaptationResult::StereoToMono => {
            AudioBuffer::try_from_interleaved(input.sample_rate(), target_layout, input.to_mono())
                .map_err(
                    |source| GraphChannelAdaptationFailure::InvalidTargetLayout {
                        layout: target_layout,
                        source,
                    },
                )
        }
        GraphChannelAdaptationResult::Exact => Ok(input.clone()),
        GraphChannelAdaptationResult::Unsupported => {
            Err(GraphChannelAdaptationFailure::Unsupported {
                input: input.channels(),
                output: target_layout,
                mode,
            })
        }
    }
}

/// Mix a buffer into a bus, adapting channels if necessary.
pub fn mix_buffer_into_bus(
    state: &mut GraphBusState,
    bus_id: &str,
    mut buffer: AudioBuffer,
    latency: u32,
    tail: u32,
) -> Result<(), GraphChannelAdaptationFailure> {
    if let Some(existing) = state.buses.get_mut(bus_id) {
        if existing.channels() != buffer.channels() {
            buffer = try_adapt_buffer_to_layout(
                &buffer,
                existing.channels(),
                GraphChannelAdaptationMode::AdaptiveMonoStereo,
            )?;
        }
        for (dst, src) in existing.samples_mut().iter_mut().zip(buffer.samples()) {
            *dst += *src;
        }
        if let Some(existing_latency) = state.latencies.get_mut(bus_id) {
            *existing_latency = (*existing_latency).max(latency);
        }
        if let Some(existing_tail) = state.tails.get_mut(bus_id) {
            *existing_tail = (*existing_tail).max(tail);
        }
        return Ok(());
    }

    state.buses.insert(bus_id.to_string(), buffer);
    state.latencies.insert(bus_id.to_string(), latency);
    state.tails.insert(bus_id.to_string(), tail);
    Ok(())
}

/// Apply a node's silence contract to a buffer.
pub fn apply_node_contract(buffer: &mut AudioBuffer, node: &GraphNodeSpec) -> bool {
    let input_silent = peak_abs(buffer.samples()) == 0.0;
    if !input_silent {
        return true;
    }

    match node.buffer_contract.silence_policy {
        GraphNodeSilencePolicy::Process => true,
        GraphNodeSilencePolicy::Bypass => false,
        GraphNodeSilencePolicy::ClearOutput => {
            buffer.clear();
            false
        }
    }
}

/// Compute the peak absolute value of a sample slice.
pub fn peak_abs(samples: &[f32]) -> f32 {
    samples
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()))
}

/// Compute the RMS value of a sample slice.
pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum = samples.iter().map(|sample| sample * sample).sum::<f32>();
    (sum / samples.len() as f32).sqrt()
}
