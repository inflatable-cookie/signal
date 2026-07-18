use super::super::RATIOS;
use super::{
    inputs::PreparedMonoRow,
    specimen::{ResourceSummary, Specimen},
    MONO_SAMPLE_RATE,
};
use crate::frequency_adaptive::{
    adaptive_single_frame_synthesis::development_measurement,
    source_studied::faithful_predictor::{coherent_representation, linked_stereo::mechanics},
};

pub(super) fn mechanics(
    specimen: &Specimen,
    root: &std::path::Path,
    resources: &mut ResourceSummary,
    hash: &mut u64,
) -> [f64; 6] {
    let primary = mechanics::primary_control(super::super::SAMPLE_RATE);
    let secondary = mechanics::secondary_control(super::super::SAMPLE_RATE);
    let silence = vec![0.0; primary.len()];
    let mut errors = [0.0_f64; 6];
    for ratio in RATIOS {
        let ordinary = specimen.render(
            root,
            &format!("mechanics-ordinary-{ratio:.2}"),
            &[&primary, &secondary],
            ratio,
        );
        resources.add(ordinary.stats);
        super::mix(hash, ordinary.output_hash);
        let duplicate = specimen.render(
            root,
            &format!("mechanics-duplicate-{ratio:.2}"),
            &[&primary, &primary],
            ratio,
        );
        resources.add(duplicate.stats);
        super::mix(hash, duplicate.output_hash);
        errors[0] = errors[0].max(maximum_error(
            &duplicate.channels[0],
            &duplicate.channels[1],
            1.0,
        ));
        let mono = specimen.render(
            root,
            &format!("mechanics-mono-{ratio:.2}"),
            &[&primary],
            ratio,
        );
        resources.add(mono.stats);
        super::mix(hash, mono.output_hash);
        errors[1] = errors[1].max(maximum_error(
            &duplicate.channels[0],
            &mono.channels[0],
            1.0,
        ));

        let hard_pan = specimen.render(
            root,
            &format!("mechanics-pan-{ratio:.2}"),
            &[&primary, &silence],
            ratio,
        );
        resources.add(hard_pan.stats);
        super::mix(hash, hard_pan.output_hash);
        errors[2] = errors[2].max(
            hard_pan.channels[1]
                .iter()
                .map(|sample| sample.abs())
                .fold(0.0, f64::max),
        );

        let swapped = specimen.render(
            root,
            &format!("mechanics-swap-{ratio:.2}"),
            &[&secondary, &primary],
            ratio,
        );
        resources.add(swapped.stats);
        super::mix(hash, swapped.output_hash);
        errors[3] = errors[3]
            .max(maximum_error(
                &swapped.channels[0],
                &ordinary.channels[1],
                1.0,
            ))
            .max(maximum_error(
                &swapped.channels[1],
                &ordinary.channels[0],
                1.0,
            ));

        let negative_primary = mechanics::scaled(&primary, -1.0);
        let negative_secondary = mechanics::scaled(&secondary, -1.0);
        let negative = specimen.render(
            root,
            &format!("mechanics-polarity-{ratio:.2}"),
            &[&negative_primary, &negative_secondary],
            ratio,
        );
        resources.add(negative.stats);
        super::mix(hash, negative.output_hash);
        errors[4] = errors[4]
            .max(maximum_error(
                &negative.channels[0],
                &ordinary.channels[0],
                -1.0,
            ))
            .max(maximum_error(
                &negative.channels[1],
                &ordinary.channels[1],
                -1.0,
            ));

        let gained_primary = mechanics::scaled(&primary, 0.25);
        let gained_secondary = mechanics::scaled(&secondary, 0.25);
        let gained = specimen.render(
            root,
            &format!("mechanics-gain-{ratio:.2}"),
            &[&gained_primary, &gained_secondary],
            ratio,
        );
        resources.add(gained.stats);
        super::mix(hash, gained.output_hash);
        errors[5] = errors[5]
            .max(maximum_error(
                &gained.channels[0],
                &ordinary.channels[0],
                0.25,
            ))
            .max(maximum_error(
                &gained.channels[1],
                &ordinary.channels[1],
                0.25,
            ));
    }
    errors
}

pub(super) fn mono(
    specimen: &Specimen,
    root: &std::path::Path,
    prepared: &[PreparedMonoRow],
    resources: &mut ResourceSummary,
    hash: &mut u64,
) -> (usize, usize, usize, String) {
    let mut evidence = Vec::new();
    let mut hard_failures = 0;
    let mut row_complete_regressions = 0;
    let mut worse_than_both = 0;
    for (index, item) in prepared.iter().enumerate() {
        let rendered = specimen.render(
            root,
            &format!("mono-{index:02}-{}", item.id),
            &[&item.source],
            item.ratio,
        );
        resources.add(rendered.stats);
        super::mix(hash, rendered.output_hash);
        let current = coherent_representation::render(&item.source, item.ratio, MONO_SAMPLE_RATE);
        let source_f32 = as_f32(&item.source);
        let current_evidence = development_measurement::measure(
            item.id,
            item.ratio,
            "coherent-control",
            &source_f32,
            &as_f32(&current.samples),
        );
        let candidate_evidence = development_measurement::measure(
            item.id,
            item.ratio,
            "sbsms-2.3.0",
            &source_f32,
            &as_f32(&rendered.channels[0]),
        );
        hard_failures += usize::from(!development_measurement::hard_pass(&candidate_evidence));
        let current_fields = quality_fields(&current_evidence);
        let candidate_fields = quality_fields(&candidate_evidence);
        row_complete_regressions += usize::from(
            candidate_fields
                .iter()
                .zip(current_fields)
                .all(|(candidate, current)| *candidate > current),
        );
        if let Some(rubber_band) = &item.rubber_band {
            let rubber_evidence = development_measurement::measure(
                item.id,
                item.ratio,
                "rubber-band-r3",
                &source_f32,
                &as_f32(rubber_band),
            );
            worse_than_both += candidate_fields
                .iter()
                .zip(current_fields)
                .zip(quality_fields(&rubber_evidence))
                .filter(|((candidate, current), rubber)| {
                    **candidate > *current && **candidate > *rubber
                })
                .count();
            evidence.push(rubber_evidence);
        }
        evidence.push(current_evidence);
        evidence.push(candidate_evidence);
    }
    (
        hard_failures,
        row_complete_regressions,
        worse_than_both,
        development_measurement::report(&evidence),
    )
}

fn quality_fields(evidence: &development_measurement::Evidence) -> [f64; 7] {
    [
        evidence.mean_event_offset,
        evidence.replica_ratio,
        evidence.static_residual,
        evidence.unsupported_mass,
        evidence.formant_residual,
        evidence.formant_shift_hz,
        evidence.boundary_growth_db,
    ]
}

fn maximum_error(actual: &[f64], expected: &[f64], gain: f64) -> f64 {
    actual
        .iter()
        .zip(expected)
        .map(|(actual, expected)| (actual - expected * gain).abs())
        .fold(0.0, f64::max)
}

fn as_f32(samples: &[f64]) -> Vec<f32> {
    samples.iter().map(|sample| *sample as f32).collect()
}
