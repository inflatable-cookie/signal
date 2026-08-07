use crate::RenderBlockPluginEvent;

/// Stable in-place insertion sort of a per-block event slice by
/// `offset_frames`. Used after live events append to a (sorted) compiled
/// prefix: near-linear on the mostly-sorted input, allocation-free, and
/// stability preserves compiled-before-live plus push order on equal
/// offsets — audio-thread safe.
#[inline]
pub(crate) fn insertion_sort_events_by_offset(events: &mut [RenderBlockPluginEvent]) {
    for index in 1..events.len() {
        let mut position = index;
        while position > 0 && events[position - 1].offset_frames > events[position].offset_frames {
            events.swap(position - 1, position);
            position -= 1;
        }
    }
}
