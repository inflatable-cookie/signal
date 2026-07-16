use super::super::super::coherent_representation;
use super::{
    contribution::{CoefficientAblation, CoefficientTraceSpec},
    linked_inner,
    overlap::SynthesisMode,
    StereoRender, SynthesisTraceSpec,
};

pub(in super::super) fn linked(
    inputs: [&[f64]; 2],
    ratio: f64,
    sample_rate: usize,
) -> StereoRender {
    linked_inner(
        inputs,
        ratio,
        sample_rate,
        None,
        None,
        None,
        SynthesisMode::Real,
    )
}

pub(in super::super) fn linked_with_relation_oracle(
    inputs: [&[f64]; 2],
    ratio: f64,
    sample_rate: usize,
    channel_one_phase_offset: f64,
) -> StereoRender {
    linked_inner(
        inputs,
        ratio,
        sample_rate,
        Some(channel_one_phase_offset),
        None,
        None,
        SynthesisMode::Real,
    )
}

pub(in super::super) fn linked_analytic(
    inputs: [&[f64]; 2],
    ratio: f64,
    sample_rate: usize,
) -> StereoRender {
    linked_inner(
        inputs,
        ratio,
        sample_rate,
        None,
        None,
        None,
        SynthesisMode::Analytic,
    )
}

pub(in super::super) fn linked_analytic_with_relation_oracle(
    inputs: [&[f64]; 2],
    ratio: f64,
    sample_rate: usize,
    channel_one_phase_offset: f64,
) -> StereoRender {
    linked_inner(
        inputs,
        ratio,
        sample_rate,
        Some(channel_one_phase_offset),
        None,
        None,
        SynthesisMode::Analytic,
    )
}

pub(in super::super) fn linked_with_synthesis_trace(
    inputs: [&[f64]; 2],
    ratio: f64,
    sample_rate: usize,
    frequency: f64,
    inverse_expected_ipd: f64,
    output_expected_ipd: [f64; 2],
    oracle_phase_offset: Option<f64>,
) -> StereoRender {
    let interior_trim = coherent_representation::source_geometry(sample_rate)[0];
    linked_inner(
        inputs,
        ratio,
        sample_rate,
        oracle_phase_offset,
        Some(SynthesisTraceSpec {
            frequency,
            inverse_expected_ipd,
            output_expected_ipd,
            sample_rate,
            interior_trim,
        }),
        None,
        SynthesisMode::Real,
    )
}

pub(in super::super) fn linked_with_coefficient_trace(
    inputs: [&[f64]; 2],
    ratio: f64,
    sample_rate: usize,
    oracle_relation: Option<f64>,
    ablation: Option<CoefficientAblation>,
) -> StereoRender {
    linked_inner(
        inputs,
        ratio,
        sample_rate,
        None,
        None,
        Some(CoefficientTraceSpec {
            oracle_relation,
            ablation,
        }),
        SynthesisMode::Real,
    )
}
