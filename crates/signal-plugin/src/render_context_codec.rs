use crate::{LoopRange, PluginRenderContext};

pub fn write_render_context_to_slice(
    context: &PluginRenderContext,
    bytes: &mut [u8],
) -> Result<(), &'static str> {
    if bytes.len() < PluginRenderContext::ENCODED_BYTES {
        return Err("render-context region is too small for encoded context");
    }

    bytes[0..4].copy_from_slice(&context.sample_rate_hz.to_le_bytes());
    bytes[4..8].copy_from_slice(&context.deadline_frames.to_le_bytes());
    bytes[8..16].copy_from_slice(&context.tempo_bpm.to_le_bytes());
    bytes[16..24].copy_from_slice(&context.timeline_position_samples.to_le_bytes());
    bytes[24] = u8::from(context.playing);
    bytes[25] = u8::from(context.bypassed);
    bytes[26] = u8::from(context.loop_range.is_some());
    bytes[27] = 0;

    let (loop_start, loop_end) = context
        .loop_range
        .map(|range| (range.start_samples, range.end_samples))
        .unwrap_or_default();
    bytes[28..36].copy_from_slice(&loop_start.to_le_bytes());
    bytes[36..44].copy_from_slice(&loop_end.to_le_bytes());
    bytes[44..48].fill(0);
    Ok(())
}

pub fn read_render_context_from_slice(bytes: &[u8]) -> Result<PluginRenderContext, &'static str> {
    if bytes.len() < PluginRenderContext::ENCODED_BYTES {
        return Err("render-context region is too small for encoded context");
    }

    let sample_rate_hz =
        u32::from_le_bytes(bytes[0..4].try_into().map_err(|_| "sample_rate decode")?);
    let deadline_frames =
        u32::from_le_bytes(bytes[4..8].try_into().map_err(|_| "deadline decode")?);
    let tempo_bpm = f64::from_le_bytes(bytes[8..16].try_into().map_err(|_| "tempo decode")?);
    let timeline_position_samples =
        i64::from_le_bytes(bytes[16..24].try_into().map_err(|_| "timeline decode")?);
    let has_loop = bytes[26] != 0;
    let loop_start = i64::from_le_bytes(bytes[28..36].try_into().map_err(|_| "loop start decode")?);
    let loop_end = i64::from_le_bytes(bytes[36..44].try_into().map_err(|_| "loop end decode")?);

    Ok(PluginRenderContext {
        sample_rate_hz,
        tempo_bpm,
        timeline_position_samples,
        playing: bytes[24] != 0,
        bypassed: bytes[25] != 0,
        loop_range: has_loop.then_some(LoopRange {
            start_samples: loop_start,
            end_samples: loop_end,
        }),
        deadline_frames,
    })
}
