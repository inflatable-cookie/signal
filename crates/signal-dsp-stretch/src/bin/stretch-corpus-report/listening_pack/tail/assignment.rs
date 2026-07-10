use crate::ExternalBenchmarkQualityRender;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum TailCandidate {
    Current,
    AdditiveZeroAnchor,
    MultiplicativeZeroFade,
}

impl TailCandidate {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::AdditiveZeroAnchor => "additive-zero-anchor",
            Self::MultiplicativeZeroFade => "multiplicative-zero-fade",
        }
    }
}

pub(super) fn candidate_slot(candidate: TailCandidate) -> usize {
    match candidate {
        TailCandidate::Current => 0,
        TailCandidate::AdditiveZeroAnchor => 1,
        TailCandidate::MultiplicativeZeroFade => 2,
    }
}

pub(super) fn stable_tail_assignment(
    render: &ExternalBenchmarkQualityRender,
    source_path: &str,
) -> [TailCandidate; 3] {
    const PERMUTATIONS: [[TailCandidate; 3]; 6] = [
        [
            TailCandidate::Current,
            TailCandidate::AdditiveZeroAnchor,
            TailCandidate::MultiplicativeZeroFade,
        ],
        [
            TailCandidate::Current,
            TailCandidate::MultiplicativeZeroFade,
            TailCandidate::AdditiveZeroAnchor,
        ],
        [
            TailCandidate::AdditiveZeroAnchor,
            TailCandidate::Current,
            TailCandidate::MultiplicativeZeroFade,
        ],
        [
            TailCandidate::AdditiveZeroAnchor,
            TailCandidate::MultiplicativeZeroFade,
            TailCandidate::Current,
        ],
        [
            TailCandidate::MultiplicativeZeroFade,
            TailCandidate::Current,
            TailCandidate::AdditiveZeroAnchor,
        ],
        [
            TailCandidate::MultiplicativeZeroFade,
            TailCandidate::AdditiveZeroAnchor,
            TailCandidate::Current,
        ],
    ];
    let assignment = format!("{}|{:.9}|{source_path}", render.case_id, render.ratio);
    let hash = assignment
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    PERMUTATIONS[hash as usize % PERMUTATIONS.len()]
}
