use crate::{
    dynamic_ratio_output_frames, dynamic_ratio_segments, sanitize_ratio, StretchRatioPoint,
};

/// Default maximum decoded source payload per offline stretch chunk.
///
/// At 48 kHz this is thirty seconds of source audio before overlap context.
pub const DEFAULT_OFFLINE_STRETCH_CHUNK_SOURCE_FRAMES: usize = 48_000 * 30;

/// Default source-frame overlap context for neighboring offline stretch chunks.
///
/// This matches the current default OfflineHighQuality STFT window size, so a
/// future chunk renderer can carry one window of context into boundary
/// crossfades without changing chunk identity.
pub const DEFAULT_OFFLINE_STRETCH_CHUNK_OVERLAP_FRAMES: usize = 2_048;

/// Bounded-memory chunking policy for offline stretch artifact renders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StretchOfflineChunkConfig {
    /// Maximum non-overlap source payload frames per chunk.
    pub max_source_frames: usize,
    /// Source context frames requested on either side of a chunk payload.
    pub overlap_frames: usize,
}

impl StretchOfflineChunkConfig {
    /// Construct a chunking policy.
    pub const fn new(max_source_frames: usize, overlap_frames: usize) -> Self {
        Self {
            max_source_frames,
            overlap_frames,
        }
    }

    fn sanitized(self) -> Self {
        let max_source_frames = self.max_source_frames.max(1);
        let overlap_frames = self.overlap_frames.min(max_source_frames / 2);
        Self {
            max_source_frames,
            overlap_frames,
        }
    }
}

impl Default for StretchOfflineChunkConfig {
    fn default() -> Self {
        Self {
            max_source_frames: DEFAULT_OFFLINE_STRETCH_CHUNK_SOURCE_FRAMES,
            overlap_frames: DEFAULT_OFFLINE_STRETCH_CHUNK_OVERLAP_FRAMES,
        }
    }
}

/// One deterministic offline stretch chunk.
///
/// `source_*` is the payload range that contributes to the artifact output.
/// `render_*` includes clamped source context for boundary handling. A chunk
/// never crosses a dynamic-ratio segment, so its `ratio` is static.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchOfflineChunk {
    /// Payload source start frame, inclusive.
    pub source_start_frame: usize,
    /// Payload source end frame, exclusive.
    pub source_end_frame: usize,
    /// Render source start frame including context, inclusive.
    pub render_start_frame: usize,
    /// Render source end frame including context, exclusive.
    pub render_end_frame: usize,
    /// Artifact output start frame for this payload, inclusive.
    pub output_start_frame: usize,
    /// Artifact output end frame for this payload, exclusive.
    pub output_end_frame: usize,
    /// Static output/input stretch ratio for this chunk.
    pub ratio: f64,
}

impl StretchOfflineChunk {
    /// Payload source frame count.
    pub fn source_frames(&self) -> usize {
        self.source_end_frame - self.source_start_frame
    }

    /// Render source frame count, including overlap context.
    pub fn render_source_frames(&self) -> usize {
        self.render_end_frame - self.render_start_frame
    }

    /// Payload output frame count.
    pub fn output_frames(&self) -> usize {
        self.output_end_frame - self.output_start_frame
    }
}

/// Deterministic bounded-memory plan for one offline stretch artifact render.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchOfflineChunkPlan {
    /// Total source frames covered by the plan.
    pub total_source_frames: usize,
    /// Total artifact output frames promised by the ratio curve.
    pub total_output_frames: usize,
    /// Sanitized chunking policy used to build this plan.
    pub config: StretchOfflineChunkConfig,
    /// Ordered chunks covering the source without payload gaps.
    pub chunks: Vec<StretchOfflineChunk>,
}

impl StretchOfflineChunkPlan {
    /// Returns true when the artifact can render as one bounded chunk.
    pub fn is_single_chunk(&self) -> bool {
        self.chunks.len() <= 1
    }

    /// Largest render source frame span requested by any chunk.
    pub fn max_render_source_frames(&self) -> usize {
        self.chunks
            .iter()
            .map(StretchOfflineChunk::render_source_frames)
            .max()
            .unwrap_or(0)
    }
}

/// Build a deterministic chunk plan for a future offline stretch artifact.
///
/// The plan follows the same stepwise ratio semantics as
/// `OfflineHighQualityStretcher::stretch_dynamic_ratio_*`: invalid ratio
/// points are ignored, gaps before the first valid point use `fallback_ratio`,
/// and target frame rounding is preserved at the dynamic-ratio segment level.
pub fn plan_offline_stretch_chunks(
    input_frames: usize,
    ratio_curve: &[StretchRatioPoint],
    fallback_ratio: f64,
    config: StretchOfflineChunkConfig,
) -> StretchOfflineChunkPlan {
    let config = config.sanitized();
    let segments =
        dynamic_ratio_segments(input_frames, ratio_curve, sanitize_ratio(fallback_ratio));
    let total_output_frames =
        dynamic_ratio_output_frames(input_frames, ratio_curve, fallback_ratio);
    let mut chunks = Vec::new();
    let mut segment_output_start = 0usize;

    for segment in segments {
        let segment_source_frames = segment.end_frame - segment.start_frame;
        let mut source_start_frame = segment.start_frame;
        while source_start_frame < segment.end_frame {
            let source_end_frame =
                (source_start_frame + config.max_source_frames).min(segment.end_frame);
            let local_start = source_start_frame - segment.start_frame;
            let local_end = source_end_frame - segment.start_frame;
            let output_start_frame = segment_output_start
                + segment_local_output_frame(local_start, segment_source_frames, segment.ratio);
            let output_end_frame = if source_end_frame == segment.end_frame {
                segment_output_start + segment.target_frames
            } else {
                segment_output_start
                    + segment_local_output_frame(local_end, segment_source_frames, segment.ratio)
                        .min(segment.target_frames)
            };
            chunks.push(StretchOfflineChunk {
                source_start_frame,
                source_end_frame,
                render_start_frame: source_start_frame
                    .saturating_sub(config.overlap_frames)
                    .max(segment.start_frame),
                render_end_frame: source_end_frame
                    .saturating_add(config.overlap_frames)
                    .min(segment.end_frame),
                output_start_frame,
                output_end_frame,
                ratio: segment.ratio,
            });
            source_start_frame = source_end_frame;
        }
        segment_output_start += segment.target_frames;
    }

    StretchOfflineChunkPlan {
        total_source_frames: input_frames,
        total_output_frames,
        config,
        chunks,
    }
}

fn segment_local_output_frame(
    local_source_frame: usize,
    segment_frames: usize,
    ratio: f64,
) -> usize {
    if segment_frames == 0 {
        return 0;
    }
    (local_source_frame as f64 * ratio).round() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_chunk_plan_splits_fixed_ratio_without_length_drift() {
        let plan =
            plan_offline_stretch_chunks(1_000, &[], 1.25, StretchOfflineChunkConfig::new(300, 64));

        assert_eq!(plan.total_source_frames, 1_000);
        assert_eq!(plan.total_output_frames, 1_250);
        assert_eq!(plan.chunks.len(), 4);
        assert_eq!(plan.chunks.first().unwrap().source_start_frame, 0);
        assert_eq!(plan.chunks.last().unwrap().source_end_frame, 1_000);
        assert_eq!(plan.chunks.last().unwrap().output_end_frame, 1_250);
        assert!(plan.chunks.iter().all(|chunk| chunk.source_frames() <= 300));
        assert!(plan
            .chunks
            .iter()
            .all(|chunk| chunk.render_source_frames() <= 300 + 64 * 2));
        assert_contiguous_output(&plan);
    }

    #[test]
    fn offline_chunk_plan_preserves_dynamic_ratio_boundaries() {
        let ratio_curve = [
            StretchRatioPoint::new(0, 0.75),
            StretchRatioPoint::new(400, 1.0),
            StretchRatioPoint::new(700, 1.5),
        ];
        let plan = plan_offline_stretch_chunks(
            1_000,
            &ratio_curve,
            1.25,
            StretchOfflineChunkConfig::new(250, 50),
        );

        assert_eq!(
            plan.total_output_frames,
            dynamic_ratio_output_frames(1_000, &ratio_curve, 1.25)
        );
        assert_eq!(plan.total_output_frames, 1_050);
        assert!(plan.chunks.iter().any(|chunk| {
            chunk.source_start_frame == 400 && (chunk.ratio - 1.0).abs() < 1.0e-9
        }));
        assert!(plan.chunks.iter().any(|chunk| {
            chunk.source_start_frame == 700 && (chunk.ratio - 1.5).abs() < 1.0e-9
        }));
        assert!(plan.chunks.iter().all(|chunk| {
            !((chunk.source_start_frame < 400 && chunk.source_end_frame > 400)
                || (chunk.source_start_frame < 700 && chunk.source_end_frame > 700))
        }));
        assert_contiguous_output(&plan);
    }

    #[test]
    fn offline_chunk_plan_clamps_overlap_to_ratio_segment() {
        let ratio_curve = [StretchRatioPoint::new(500, 1.5)];
        let plan = plan_offline_stretch_chunks(
            1_000,
            &ratio_curve,
            0.75,
            StretchOfflineChunkConfig::new(300, 100),
        );

        let before_boundary = plan
            .chunks
            .iter()
            .find(|chunk| chunk.source_end_frame == 500)
            .expect("chunk before boundary");
        let after_boundary = plan
            .chunks
            .iter()
            .find(|chunk| chunk.source_start_frame == 500)
            .expect("chunk after boundary");

        assert_eq!(before_boundary.render_end_frame, 500);
        assert_eq!(after_boundary.render_start_frame, 500);
        assert_contiguous_output(&plan);
    }

    #[test]
    fn offline_chunk_plan_sanitizes_empty_config() {
        let plan = plan_offline_stretch_chunks(
            3,
            &[],
            f64::NAN,
            StretchOfflineChunkConfig::new(0, usize::MAX),
        );

        assert_eq!(plan.config, StretchOfflineChunkConfig::new(1, 0));
        assert_eq!(plan.total_output_frames, 3);
        assert_eq!(plan.chunks.len(), 3);
        assert!(plan.chunks.iter().all(|chunk| chunk.output_frames() == 1));
        assert_contiguous_output(&plan);
    }

    fn assert_contiguous_output(plan: &StretchOfflineChunkPlan) {
        let mut output_cursor = 0usize;
        for chunk in &plan.chunks {
            assert_eq!(chunk.output_start_frame, output_cursor);
            output_cursor = chunk.output_end_frame;
        }
        assert_eq!(output_cursor, plan.total_output_frames);
    }
}
