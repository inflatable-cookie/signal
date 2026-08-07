use super::compile::RenderPlan;
use super::types::CompiledSource;

impl RenderPlan {
    /// Carry smoothed gains and tone phases over from the plan being
    /// replaced, so a recompile (gain tweak, clip edit) never steps audio.
    /// Matching is precomputed by the controller into `inherit_stage_map` /
    /// `inherit_clip_maps` at install time (by stage_id and clip_id), so this
    /// is O(stages + clips) index copies — no identity comparisons run on
    /// the audio thread, and inserting a clip mid-lane no longer cross-wires
    /// its neighbours' state.
    pub(crate) fn inherit_state(&mut self, previous: &mut RenderPlan) {
        // Limiter recovery gain carries over so a recompile mid-limiting
        // does not snap the gain back to unity.
        if let (Some(limiter), Some(previous_limiter)) =
            (self.limiter.as_mut(), previous.limiter.as_ref())
        {
            limiter.set_gain(previous_limiter.gain());
        }
        if self.inherit_stage_map.len() != self.stages.len() {
            // No map (first install or controller skipped): nothing carries.
            return;
        }
        for (index, stage) in self.stages.iter_mut().enumerate() {
            let Some(previous_index) = self.inherit_stage_map[index] else {
                continue;
            };
            let Some(previous_node) = previous.stages.get_mut(previous_index) else {
                continue;
            };
            stage.gain_current = previous_node.gain_current;
            if stage.delay_ring.len() == previous_node.delay_ring.len()
                && !stage.delay_ring.is_empty()
            {
                stage.delay_ring.copy_from_slice(&previous_node.delay_ring);
                stage.delay_cursor = previous_node.delay_cursor;
            }
            let clip_map = self
                .inherit_clip_maps
                .get(index)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            for (clip_index, clip) in stage.clips.iter_mut().enumerate() {
                let Some(previous_clip_index) = clip_map.get(clip_index).copied().flatten() else {
                    continue;
                };
                let Some(previous_clip) = previous_node.clips.get_mut(previous_clip_index) else {
                    continue;
                };
                match (&mut clip.source, &mut previous_clip.source) {
                    (
                        CompiledSource::Tone { phase, step },
                        CompiledSource::Tone {
                            phase: previous_phase,
                            step: previous_step,
                        },
                    ) if *step == *previous_step => {
                        *phase = *previous_phase;
                    }
                    // Streaming clips MOVE their held read-ahead chunks into
                    // the new plan (same handle, same rate ratio), so an
                    // identity recompile mid-stream never underruns. Chunks
                    // that do not transfer ride the retired plan back to the
                    // control side and drop there — never on this thread.
                    (
                        CompiledSource::Stream {
                            handle, held, step, ..
                        },
                        CompiledSource::Stream {
                            handle: previous_handle,
                            held: previous_held,
                            step: previous_step,
                            ..
                        },
                    ) if handle == previous_handle && *step == *previous_step => {
                        for (slot, previous_slot) in held.iter_mut().zip(previous_held.iter_mut()) {
                            *slot = previous_slot.take();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
