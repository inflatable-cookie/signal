use rustfft::num_complex::Complex64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in super::super) enum CoefficientAblation {
    Initial,
    Fallback,
    Weak,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CoefficientLifecycle {
    Initial,
    Corrected,
    Fallback,
}

impl CoefficientLifecycle {
    fn index(self) -> usize {
        match self {
            Self::Initial => 0,
            Self::Corrected => 1,
            Self::Fallback => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::frequency_adaptive) struct CoefficientClassEvidence {
    pub(in crate::frequency_adaptive) count: usize,
    pub(in crate::frequency_adaptive) synthesized_energy: f64,
    pub(in crate::frequency_adaptive) measurable_relations: usize,
    pub(in crate::frequency_adaptive) maximum_relation_error: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::frequency_adaptive) struct CoefficientContributionTrace {
    pub(in super::super) lifecycle: [CoefficientClassEvidence; 3],
    pub(in super::super) energy: [CoefficientClassEvidence; 2],
}

#[derive(Clone, Copy)]
pub(super) struct CoefficientTraceSpec {
    pub(super) oracle_relation: Option<f64>,
    pub(super) ablation: Option<CoefficientAblation>,
}

pub(super) struct CoefficientTraceState {
    spec: CoefficientTraceSpec,
    trace: CoefficientContributionTrace,
}

pub(super) struct ContributionFrame {
    lifecycle: Option<Vec<CoefficientLifecycle>>,
}

impl ContributionFrame {
    pub(super) fn new(enabled: bool, bins: usize) -> Self {
        Self {
            lifecycle: enabled.then(|| vec![CoefficientLifecycle::Initial; bins]),
        }
    }

    pub(super) fn record_recurrence(&mut self, bin: usize, corrected: bool) {
        if let Some(lifecycle) = &mut self.lifecycle {
            lifecycle[bin] = if corrected {
                CoefficientLifecycle::Corrected
            } else {
                CoefficientLifecycle::Fallback
            };
        }
    }

    pub(super) fn finish(
        self,
        state: &mut Option<CoefficientTraceState>,
        current: &[Vec<Complex64>; 2],
        output: &mut [Vec<Complex64>; 2],
        significant_energy: f64,
    ) {
        if let (Some(state), Some(lifecycle)) = (state, self.lifecycle) {
            state.process_frame(current, output, &lifecycle, significant_energy);
        }
    }
}

impl CoefficientTraceState {
    pub(super) fn new(spec: CoefficientTraceSpec) -> Self {
        Self {
            spec,
            trace: CoefficientContributionTrace::default(),
        }
    }

    pub(super) fn process_frame(
        &mut self,
        current: &[Vec<Complex64>; 2],
        output: &mut [Vec<Complex64>; 2],
        lifecycle: &[CoefficientLifecycle],
        significant_energy: f64,
    ) {
        for bin in 0..output[0].len() {
            let significant = current[0][bin].norm_sqr() > significant_energy
                && current[1][bin].norm_sqr() > significant_energy;
            let input_relation = relation(current, bin);
            observe(
                &mut self.trace.lifecycle[lifecycle[bin].index()],
                output,
                bin,
                input_relation,
            );
            observe(
                &mut self.trace.energy[usize::from(!significant)],
                output,
                bin,
                input_relation,
            );
            let selected = match self.spec.ablation {
                Some(CoefficientAblation::Initial) => {
                    lifecycle[bin] == CoefficientLifecycle::Initial
                }
                Some(CoefficientAblation::Fallback) => {
                    lifecycle[bin] == CoefficientLifecycle::Fallback
                }
                Some(CoefficientAblation::Weak) => !significant,
                None => false,
            };
            if selected {
                force_relation(
                    output,
                    bin,
                    self.spec.oracle_relation.unwrap_or(input_relation),
                );
            }
        }
    }

    pub(super) fn finish(self) -> CoefficientContributionTrace {
        self.trace
    }
}

fn observe(
    evidence: &mut CoefficientClassEvidence,
    output: &[Vec<Complex64>; 2],
    bin: usize,
    expected_relation: f64,
) {
    let energy = [output[0][bin].norm_sqr(), output[1][bin].norm_sqr()];
    evidence.count += 1;
    evidence.synthesized_energy += energy[0] + energy[1];
    if energy[0] > 0.0 && energy[1] > 0.0 {
        evidence.measurable_relations += 1;
        evidence.maximum_relation_error = evidence
            .maximum_relation_error
            .max(wrap(relation(output, bin) - expected_relation).abs());
    }
}

fn force_relation(output: &mut [Vec<Complex64>; 2], bin: usize, relation: f64) {
    let target_energy = [output[0][bin].norm_sqr(), output[1][bin].norm_sqr()];
    let reference = usize::from(target_energy[1] > target_energy[0]);
    let peer = 1 - reference;
    if target_energy[reference] == 0.0 || target_energy[peer] == 0.0 {
        return;
    }
    let offset = if reference == 0 { relation } else { -relation };
    let projected = output[reference][bin] * Complex64::from_polar(1.0, offset);
    output[peer][bin] = projected * (target_energy[peer] / projected.norm_sqr()).sqrt();
}

fn relation(spectra: &[Vec<Complex64>; 2], bin: usize) -> f64 {
    (spectra[1][bin] * spectra[0][bin].conj()).arg()
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}
