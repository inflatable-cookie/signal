use super::{mechanics::*, *};

pub(super) fn guided_stage_review() -> GuidedStageReview {
    let representation_hash = super::super::report::stage_a_review().hash;
    let rates = PROOF_RATES
        .into_iter()
        .map(|sample_rate| {
            let geometry = prepare(sample_rate).expect("frozen Rule 31T geometry");
            let lengths = [
                geometry.outer_advance + 17,
                4 * geometry.outer_advance + 29,
                12 * geometry.outer_advance + 31,
            ]
            .map(|length| run_length(&geometry, length));
            GuidedRateReview {
                sample_rate,
                geometry: [
                    geometry.fft_frames,
                    geometry.outer_advance,
                    geometry.hop,
                    geometry.representation.bands.len(),
                    geometry.positive_atoms,
                ],
                lengths,
            }
        })
        .collect::<Vec<_>>();
    let maximum = prepare(48_000).expect("maximum proof geometry");
    let mut review = GuidedStageReview {
        representation_hash,
        rates,
        overflow_failures: overflow_failures(&maximum),
        hash: 0,
    };
    review.hash = review_hash(&review);
    review
}

fn review_hash(review: &GuidedStageReview) -> u64 {
    let mut hash = HASH_OFFSET;
    hash_u64(&mut hash, review.representation_hash);
    for rate in &review.rates {
        hash_usize(&mut hash, rate.sample_rate);
        for value in rate.geometry {
            hash_usize(&mut hash, value);
        }
        for length in rate.lengths {
            hash_length(&mut hash, length);
        }
    }
    hash_usize(&mut hash, review.overflow_failures);
    hash
}
