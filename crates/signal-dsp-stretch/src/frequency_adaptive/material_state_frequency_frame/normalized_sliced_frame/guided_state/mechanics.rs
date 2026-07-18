use super::{fixtures::*, *};
use crate::frequency_adaptive::material_state_frequency_frame::{
    absolute_bin,
    guided_frequency_partitioned_linked_phase::{Decision, Workspace},
};

mod projection;
use projection::*;

pub(super) fn run_length(geometry: &Geometry, length: usize) -> LengthReview {
    let slices = required_slice_count(length, geometry.outer_advance);
    let Some((first, last)) = render::schedule_range(length, geometry) else {
        return LengthReview::default();
    };
    let expected_updates = (last - first + 1) as usize;
    let frequencies_hz = positive_frequencies(geometry);
    let mut workspaces =
        std::array::from_fn::<_, 4, _>(|_| Workspace::new(geometry.sample_rate, geometry.hop));
    let decisions = [
        Decision::Reset,
        Decision::Attack,
        Decision::Ordinary,
        Decision::Unlocked,
        Decision::Locked,
    ];
    let mut review = LengthReview {
        length,
        slices,
        expected_updates,
        ..LengthReview::default()
    };
    let mut hash = HASH_OFFSET;

    for (update, time) in (first..=last).enumerate() {
        review.failures[0] += workspaces
            .iter()
            .filter(|workspace| workspace.updates != update)
            .count();
        let active = render::active_slices(time, slices);
        let active_count = active.iter().flatten().count();
        review.active_high_water = review.active_high_water.max(active_count);
        review.dual_layer_updates += usize::from(active_count == 2);
        review.failures[4] += usize::from(active_count > OUTPUT_SLICE_CAPACITY);

        let decision = decisions[update % decisions.len()];
        review.decision_updates[decision.index()] += 1;
        let context = match time.rem_euclid(16) {
            15 => 1,
            0 => 2,
            1 => 3,
            _ => 0,
        };
        review.boundary_decisions[context][decision.index()] += 1;

        let duplicate = scenario(
            [Signal::A, Signal::A],
            &frequencies_hz,
            geometry,
            time,
            active,
        );
        let mono = scenario(
            [Signal::A, Signal::Silence],
            &frequencies_hz,
            geometry,
            time,
            active,
        );
        let ordinary = scenario(
            [Signal::A, Signal::B],
            &frequencies_hz,
            geometry,
            time,
            active,
        );
        let swapped = scenario(
            [Signal::B, Signal::A],
            &frequencies_hz,
            geometry,
            time,
            active,
        );
        let projected = [
            project_layers(
                &mut workspaces[0],
                &duplicate.0,
                &duplicate.1,
                &frequencies_hz,
                decision,
            ),
            project_layers(
                &mut workspaces[1],
                &mono.0,
                &mono.1,
                &frequencies_hz,
                decision,
            ),
            project_layers(
                &mut workspaces[2],
                &ordinary.0,
                &ordinary.1,
                &frequencies_hz,
                decision,
            ),
            project_layers(
                &mut workspaces[3],
                &swapped.0,
                &swapped.1,
                &frequencies_hz,
                decision,
            ),
        ];
        let [Ok(duplicate_out), Ok(mono_out), Ok(ordinary_out), Ok(swapped_out)] = projected else {
            review.failures[1] += 1;
            continue;
        };

        for (input, output) in [
            (&duplicate, &duplicate_out),
            (&mono, &mono_out),
            (&ordinary, &ordinary_out),
            (&swapped, &swapped_out),
        ] {
            projection_errors(&input.0, &input.1, output, &mut review.maximum_errors);
        }
        for layer in 0..active_count {
            review.maximum_errors[0] = review.maximum_errors[0].max(frame_error(
                &duplicate_out.layers[layer][0],
                &duplicate_out.layers[layer][1],
            ));
            review.maximum_errors[1] = review.maximum_errors[1].max(frame_error(
                &duplicate_out.layers[layer][0],
                &mono_out.layers[layer][0],
            ));
            review.maximum_errors[2] =
                review.maximum_errors[2].max(maximum_norm(&mono_out.layers[layer][1]));
            review.maximum_errors[3] = review.maximum_errors[3]
                .max(frame_error(
                    &ordinary_out.layers[layer][0],
                    &swapped_out.layers[layer][1],
                ))
                .max(frame_error(
                    &ordinary_out.layers[layer][1],
                    &swapped_out.layers[layer][0],
                ));
        }
        for output in [&duplicate_out, &mono_out, &ordinary_out, &swapped_out] {
            review.failures[3] += non_finite_values(output);
            hash_projected(&mut hash, output);
        }
        review.updates += 1;
        review.failures[0] += workspaces
            .iter()
            .filter(|workspace| workspace.updates != review.updates)
            .count();
    }

    review.failures[2] += usize::from(review.updates != expected_updates);
    review.failures[2] += workspaces
        .iter()
        .filter(|workspace| workspace.updates != expected_updates)
        .count();
    review.state = workspaces[0].counts;
    review.region_high_water = workspaces[0].region_high_water;
    review.atom_visits = workspaces[0].updates * frequencies_hz.len();
    review.region_visits = workspaces[0].region_visits;
    review.hash = hash;
    review
}

pub(super) fn overflow_failures(geometry: &Geometry) -> usize {
    let frequencies = positive_frequencies(geometry);
    let state = std::array::from_fn(|_| {
        frame(
            Signal::A,
            &frequencies,
            geometry.sample_rate,
            geometry.hop,
            0,
        )
    });
    let mut layers = layers(&state, [Some(0), Some(1)]);
    layers.push(layers[0].clone());
    let mut workspace = Workspace::new(geometry.sample_rate, geometry.hop);
    let result = project_layers(
        &mut workspace,
        &state,
        &layers,
        &frequencies,
        Decision::Reset,
    );
    usize::from(!matches!(result, Err(BoundaryError::OutputLayers)))
        + usize::from(workspace.updates != 0)
}

fn positive_frequencies(geometry: &Geometry) -> Vec<f64> {
    geometry
        .representation
        .bands
        .iter()
        .filter(|band| band.center <= geometry.fft_frames / 2)
        .map(|band| {
            absolute_bin(band.center, geometry.fft_frames) as f64 * geometry.sample_rate as f64
                / geometry.fft_frames as f64
        })
        .collect()
}
