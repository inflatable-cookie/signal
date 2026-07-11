pub(super) fn solve(entropies: &[[f64; 4]]) -> (Vec<u8>, f64) {
    if entropies
        .iter()
        .all(|row| row.iter().all(|value| *value == 0.0))
    {
        return (vec![3; entropies.len()], 0.0);
    }
    let mut states: [Option<(f64, Vec<u8>)>; 4] =
        std::array::from_fn(|level| Some((entropies[0][level], vec![level as u8])));
    for row in &entropies[1..] {
        let previous = states;
        states = std::array::from_fn(|level| {
            let mut best: Option<(f64, Vec<u8>)> = None;
            for prior in 0_usize..4 {
                if prior.abs_diff(level) > 1 {
                    continue;
                }
                let Some((cost, path)) = &previous[prior] else {
                    continue;
                };
                let mut candidate_path = path.clone();
                candidate_path.push(level as u8);
                let candidate = (cost + row[level], candidate_path);
                if better(&candidate, best.as_ref()) {
                    best = Some(candidate);
                }
            }
            best
        });
    }
    let (cost, path) = states
        .into_iter()
        .flatten()
        .reduce(|best, candidate| {
            if better(&candidate, Some(&best)) {
                candidate
            } else {
                best
            }
        })
        .unwrap_or((0.0, Vec::new()));
    (path, cost)
}

fn better(candidate: &(f64, Vec<u8>), best: Option<&(f64, Vec<u8>)>) -> bool {
    let Some(best) = best else { return true };
    candidate.0 < best.0
        || (candidate.0 == best.0
            && candidate
                .1
                .iter()
                .zip(&best.1)
                .find_map(|(left, right)| (left != right).then_some(left > right))
                .unwrap_or(false))
}
