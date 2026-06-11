use super::*;

impl RuntimeTempoMapStateModel {
    pub(crate) fn apply_projection(&mut self, mut projection: RuntimeTempoMapProjection) {
        projection.segment_count = projection.segments.len();
        projection
            .segments
            .sort_by_key(|segment| (segment.start_samples, segment.segment_id.clone()));
        self.projection = Some(projection);
    }

    pub(crate) fn resolve(
        &self,
        timeline_position_samples: Option<i64>,
        projected_transport: Option<TransportProjection>,
        observed_tempo_bpm: Option<f64>,
    ) -> RuntimeResolvedTempo {
        let projected_tempo_bpm = projected_transport
            .map(|transport| transport.tempo_bpm)
            .or(observed_tempo_bpm)
            .filter(|tempo| tempo.is_finite() && *tempo > 0.0);
        let mut resolved = RuntimeResolvedTempo {
            tempo_bpm: projected_tempo_bpm.unwrap_or(120.0),
            source: if projected_tempo_bpm.is_some() {
                RuntimeTempoSource::TransportProjection
            } else {
                RuntimeTempoSource::DefaultFallback
            },
            active_segment_id: None,
            active_segment_index: None,
            next_segment_start_samples: None,
            timeline_position_samples,
        };

        let Some(position) = timeline_position_samples else {
            return resolved;
        };
        let Some(projection) = self.projection.as_ref() else {
            return resolved;
        };

        for (index, segment) in projection.segments.iter().enumerate() {
            if segment.start_samples > position {
                resolved.next_segment_start_samples = Some(segment.start_samples);
                break;
            }
            let segment_end = segment.end_samples.unwrap_or(i64::MAX);
            if position < segment_end {
                resolved.tempo_bpm = resolved_tempo_for_segment(segment, position);
                resolved.source = RuntimeTempoSource::TempoMapSegment;
                resolved.active_segment_id = Some(segment.segment_id.clone());
                resolved.active_segment_index = Some(index);
                resolved.next_segment_start_samples = projection
                    .segments
                    .get(index + 1)
                    .map(|next| next.start_samples);
                break;
            }
        }

        resolved
    }

    pub(crate) fn snapshot(&self, resolved: &RuntimeResolvedTempo) -> RuntimeTempoMapSnapshot {
        let segments = self
            .projection
            .as_ref()
            .map(|projection| {
                projection
                    .segments
                    .iter()
                    .enumerate()
                    .map(|(index, segment)| RuntimeTempoMapSegmentSnapshot {
                        segment_id: segment.segment_id.clone(),
                        start_samples: segment.start_samples,
                        end_samples: segment.end_samples,
                        start_tempo_bpm: segment.start_tempo_bpm,
                        end_tempo_bpm: segment.end_tempo_bpm,
                        interpolation: segment.interpolation,
                        covers_timeline_position: resolved.active_segment_index == Some(index),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        RuntimeTempoMapSnapshot {
            segment_count: segments.len(),
            active_segment_id: resolved.active_segment_id.clone(),
            active_segment_index: resolved.active_segment_index,
            next_segment_start_samples: resolved.next_segment_start_samples,
            resolved_tempo_bpm: resolved.tempo_bpm,
            tempo_source: resolved.source,
            timeline_position_samples: resolved.timeline_position_samples,
            segments,
        }
    }
}

fn resolved_tempo_for_segment(segment: &RuntimeTempoMapSegmentProjection, position: i64) -> f64 {
    match (
        segment.interpolation,
        segment.end_samples,
        segment.end_tempo_bpm,
    ) {
        (RuntimeTempoMapInterpolation::Linear, Some(end_samples), Some(end_tempo_bpm))
            if end_samples > segment.start_samples =>
        {
            let span = (end_samples - segment.start_samples) as f64;
            let offset = (position - segment.start_samples)
                .clamp(0, end_samples - segment.start_samples) as f64;
            segment.start_tempo_bpm + ((end_tempo_bpm - segment.start_tempo_bpm) * (offset / span))
        }
        _ => segment.start_tempo_bpm,
    }
}
