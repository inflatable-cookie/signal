use super::*;

#[derive(Clone, Copy)]
pub(super) struct AtomEndpoints {
    pub layers: [[[Complex64; 2]; 2]; 2],
    pub fraction: f64,
}

impl AtomEndpoints {
    pub fn new(layers: [[[Complex64; 2]; 2]; 2], fraction: f64) -> Self {
        Self { layers, fraction }
    }

    pub fn magnitudes(&self, layer: usize) -> [f64; 2] {
        std::array::from_fn(|channel| {
            let values = self.layers[layer][channel];
            values[0].norm() + (values[1].norm() - values[0].norm()) * self.fraction
        })
    }

    pub fn base(&self, layer: usize, reference: usize) -> ([Complex64; 2], RelationCounts, f64) {
        let (relation, counts) = self.shared_relation(layer, reference);
        let (output, error) = self.base_with_relation(layer, reference, relation);
        (output, counts, error)
    }

    pub fn shared_relation(&self, layer: usize, reference: usize) -> (f64, RelationCounts) {
        let peer = 1 - reference;
        let endpoints = &self.layers[layer];
        let reference_value = polar_sample(endpoints[reference], self.fraction);
        let magnitudes = self.magnitudes(layer);
        if magnitudes[reference] == 0.0 && magnitudes[peer] == 0.0 {
            let mut counts = RelationCounts::default();
            counts.silent = 1;
            return (0.0, counts);
        }
        if magnitudes[peer] == 0.0 {
            let mut counts = RelationCounts::default();
            counts.zero_peer = 1;
            return (0.0, counts);
        }

        let relations = [0, 1].map(|endpoint| {
            let reference_value = endpoints[reference][endpoint];
            let peer_value = endpoints[peer][endpoint];
            (reference_value.norm() > 0.0 && peer_value.norm() > 0.0)
                .then(|| (peer_value * reference_value.conj()).arg())
        });
        let (relation, counts) = match relations {
            [Some(first), Some(second)] => {
                let mut counts = RelationCounts::default();
                counts.two_defined = 1;
                (first + wrap(second - first) * self.fraction, counts)
            }
            [Some(value), None] | [None, Some(value)] => {
                let mut counts = RelationCounts::default();
                counts.one_defined = 1;
                (value, counts)
            }
            [None, None] => {
                let mut counts = RelationCounts::default();
                counts.undefined = 1;
                let current_peer = polar_sample(endpoints[peer], self.fraction);
                ((current_peer * reference_value.conj()).arg(), counts)
            }
        };
        (relation, counts)
    }

    pub fn base_with_relation(
        &self,
        layer: usize,
        reference: usize,
        relation: f64,
    ) -> ([Complex64; 2], f64) {
        let peer = 1 - reference;
        let endpoints = &self.layers[layer];
        let reference_value = polar_sample(endpoints[reference], self.fraction);
        let magnitudes = self.magnitudes(layer);
        let mut output = [Complex64::default(); 2];
        output[reference] = reference_value;
        if magnitudes[peer] > 0.0 {
            output[peer] =
                Complex64::from_polar(magnitudes[peer], reference_value.arg() + relation);
        }
        let error = if output[reference].norm() > 0.0 && output[peer].norm() > 0.0 {
            wrap((output[peer] * output[reference].conj()).arg() - relation).abs()
        } else {
            0.0
        };
        (output, error)
    }
}

fn polar_sample(values: [Complex64; 2], fraction: f64) -> Complex64 {
    let magnitude = values[0].norm() + (values[1].norm() - values[0].norm()) * fraction;
    let phase = values[0].arg() + wrap(values[1].arg() - values[0].arg()) * fraction;
    Complex64::from_polar(magnitude, phase)
}

pub(super) fn mechanics_review() -> (RelationCounts, f64) {
    let one = Complex64::new(1.0, 0.0);
    let zero = Complex64::default();
    let controls = [
        AtomEndpoints::new([[[one, one], [one, one]]; 2], 0.5),
        AtomEndpoints::new([[[one, one], [one, zero]]; 2], 0.5),
        AtomEndpoints::new([[[one, zero], [zero, one]]; 2], 0.5),
        AtomEndpoints::new([[[one, one], [zero, zero]]; 2], 0.5),
        AtomEndpoints::new([[[zero, zero], [zero, zero]]; 2], 0.5),
    ];
    let mut counts = RelationCounts::default();
    let mut maximum_error = 0.0_f64;
    for control in controls {
        let (_, result, error) = control.base(0, 0);
        counts.add(result);
        maximum_error = maximum_error.max(error);
    }
    (counts, maximum_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoints(reference: [f64; 2], peer: [f64; 2]) -> AtomEndpoints {
        AtomEndpoints::new(
            [
                [
                    [
                        Complex64::from_polar(1.0, reference[0]),
                        Complex64::from_polar(1.0, reference[1]),
                    ],
                    [
                        Complex64::from_polar(1.0, peer[0]),
                        Complex64::from_polar(1.0, peer[1]),
                    ],
                ],
                [[Complex64::default(); 2]; 2],
            ],
            0.5,
        )
    }

    #[test]
    fn sliced_material_relation_interpolates_relation_not_peer_phase() {
        let degrees = |value: f64| value.to_radians();
        let source = endpoints([0.0, degrees(170.0)], [0.0, degrees(-170.0)]);
        let (output, counts, error) = source.base(0, 0);
        let relation = wrap((output[1] * output[0].conj()).arg());
        assert!((relation - degrees(10.0)).abs() <= 1.0e-12);
        assert_eq!(counts.two_defined, 1);
        assert!(error <= 1.0e-12);
    }

    #[test]
    fn sliced_material_relation_mechanics_cover_all_states() {
        let (counts, error) = mechanics_review();
        assert_eq!(counts.as_array(), [1; 5]);
        assert!(error <= 1.0e-12);
    }

    #[test]
    fn sliced_material_relation_identity_is_shared_across_layers() {
        let source = AtomEndpoints::new(
            [
                [
                    [Complex64::new(1.0, 0.0); 2],
                    [Complex64::from_polar(1.0, 0.2); 2],
                ],
                [
                    [Complex64::new(1.0, 0.0); 2],
                    [Complex64::from_polar(1.0, 1.4); 2],
                ],
            ],
            0.5,
        );
        let (relation, _) = source.shared_relation(0, 0);
        let (second_layer, error) = source.base_with_relation(1, 0, relation);
        assert!((wrap((second_layer[1] * second_layer[0].conj()).arg()) - 0.2).abs() <= 1.0e-12);
        assert!(error <= 1.0e-12);
    }
}
