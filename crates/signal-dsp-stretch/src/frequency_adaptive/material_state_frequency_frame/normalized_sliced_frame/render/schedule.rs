use super::*;

pub(super) fn required_slice_count(length: usize, advance: usize) -> usize {
    if length == 0 {
        0
    } else {
        (length - 1) / advance + 2
    }
}

pub(in super::super) fn schedule_range(
    length: usize,
    geometry: &Geometry,
) -> Option<(isize, isize)> {
    let slices = required_slice_count(length, geometry.outer_advance);
    (slices > 0).then(|| (-16, 16 * (slices as isize - 2) + 31))
}

pub(in super::super) fn active_slices(time: isize, slice_count: usize) -> [Option<usize>; 2] {
    let newest = (time + 16).div_euclid(16);
    let mut active = [None; OUTPUT_SLICE_CAPACITY];
    for candidate in [newest - 1, newest] {
        if candidate < 0 || candidate >= slice_count as isize {
            continue;
        }
        let start = (candidate - 1) * 16;
        if (start..start + 32).contains(&time) {
            if let Some(slot) = active.iter_mut().find(|slot| slot.is_none()) {
                *slot = Some(candidate as usize);
            }
        }
    }
    active
}

pub(super) fn boundary_token_review(length: usize, geometry: &Geometry) -> TokenReview {
    let slices = required_slice_count(length, geometry.outer_advance);
    let expected_updates = if slices == 0 { 0 } else { 16 * slices + 16 };
    if slices == 0 {
        return TokenReview::default();
    }
    let first = -16_isize;
    let last = 16 * (slices as isize - 2) + 31;
    let mut active = [None; OUTPUT_SLICE_CAPACITY];
    let mut review = TokenReview {
        expected_updates,
        ..TokenReview::default()
    };
    let mut previous_time = None;
    for time in first..=last {
        review.duplicate_updates +=
            usize::from(previous_time.is_some_and(|previous| time != previous + 1));
        previous_time = Some(time);
        let previous_active = active;
        for slot in &mut active {
            if slot.is_some_and(|slice| time >= (slice as isize - 1) * 16 + 32) {
                *slot = None;
                review.slice_retirements += 1;
            }
        }
        if (time + 16).rem_euclid(16) == 0 {
            let slice = ((time + 16) / 16) as usize;
            if slice < slices {
                if let Some(slot) = active.iter_mut().find(|slot| slot.is_none()) {
                    *slot = Some(slice);
                    review.slice_creations += 1;
                } else {
                    review.capacity_failures += 1;
                }
            }
        }
        if active != previous_active {
            review.boundary_crossings += 1;
            review.reset_failures += usize::from(review.final_value != review.updates);
        }
        review.active_high_water = review
            .active_high_water
            .max(active.iter().filter(|slot| slot.is_some()).count());
        review.updates += 1;
        review.final_value += 1;
    }
    review
}

pub(super) fn coverage_failures(length: usize, slice_count: usize, advance: usize) -> usize {
    (0..length)
        .filter(|logical| {
            let logical = *logical;
            let block = logical / advance;
            let count = [block, block + 1]
                .into_iter()
                .filter(|slice| {
                    let start = (*slice as isize - 1) * advance as isize;
                    let logical = logical as isize;
                    (start..start + 2 * advance as isize).contains(&logical) && *slice < slice_count
                })
                .count();
            count != 2
        })
        .count()
}
