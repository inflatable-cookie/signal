//! Channel mix-matrix coefficient generators.
//!
//! Per chorus a14, spatial control in the mix graph is an N×M gain matrix at
//! a node boundary — never a baked end-of-chain knob. This module generates
//! the coefficients; applying them per frame is the consumer's (alloc-free)
//! loop. Matrices are row-major `source_channels × dest_channels`:
//! `coefficient[source_channel * dest_channels + dest_channel]`.
//!
//! The generators here are count-based: they take channel counts, not layout
//! types, so this crate stays free of any render-plane vocabulary. Layout
//! aware generators (ITU downmix for 5.1/7.1.4, azimuth/elevation panners)
//! layer on later as additional functions, not new primitives.

/// Equal-power stereo pan/balance matrix (2×2, row-major source×dest).
///
/// `pan` runs -1.0 (hard left) .. 0.0 (center) .. 1.0 (hard right) and is
/// clamped. The law is the constant-power quarter-circle: left passes at
/// `cos(theta)`, right at `sin(theta)` with `theta = (pan + 1) * π/4`, so
/// center sits at -3 dB on both sides and total power is constant across the
/// sweep. Layout: `[l→L, l→R, r→L, r→R]`.
pub fn equal_power_pan_matrix(pan: f32) -> [f32; 4] {
    let theta = (pan.clamp(-1.0, 1.0) + 1.0) * std::f32::consts::FRAC_PI_4;
    let left_gain = theta.cos();
    let right_gain = theta.sin();
    [left_gain, 0.0, 0.0, right_gain]
}

/// Default adapter matrix between two channel counts (row-major
/// `source_channels × dest_channels`).
///
/// These are the implicit conversions used when a graph edge connects two
/// differently-shaped nodes without an explicit matrix. Rules, in order:
///
/// - **equal counts**: identity (formats match, samples pass through);
/// - **mono → N**: every destination channel receives `1/sqrt(N)` — equal
///   distribution at -3 dB per channel-count doubling, preserving power;
/// - **N → mono**: equal-weight average (`1/N` per source channel);
/// - **narrow → wide** (1 < src < dst): identity into the first `src`
///   destination channels, silence in the rest. Spreading without layout
///   semantics would invent spatial policy; count-only generics truncate.
///   Layout-aware ITU-style upmix arrives with layout-typed generators;
/// - **wide → narrow** (src > dst > 1): interleaved fold — source channel
///   `c` lands on destination `c % dst`, weighted by the equal-weight
///   average of the sources folding into that destination. For 4→2 this is
///   `L = (s0 + s2) / 2`, `R = (s1 + s3) / 2`. ITU coefficients apply only
///   to named layouts (5.1 → stereo etc.) and arrive with layout-typed
///   generators; counts alone get this documented neutral fold.
///
/// Channel counts of zero are clamped to one.
pub fn default_adapter_matrix(source_channels: u16, dest_channels: u16) -> Vec<f32> {
    let source = source_channels.max(1) as usize;
    let dest = dest_channels.max(1) as usize;
    let mut matrix = vec![0.0f32; source * dest];
    if source == dest {
        for channel in 0..source {
            matrix[channel * dest + channel] = 1.0;
        }
    } else if source == 1 {
        // One row of `dest` coefficients: the whole matrix.
        matrix.fill(1.0 / (dest as f32).sqrt());
    } else if dest == 1 {
        // One column of `source` coefficients: the whole matrix.
        matrix.fill(1.0 / source as f32);
    } else if source < dest {
        for channel in 0..source {
            matrix[channel * dest + channel] = 1.0;
        }
    } else {
        // source > dest: interleaved fold, equal-weight per destination.
        for dest_channel in 0..dest {
            let fold_count = (source - dest_channel).div_ceil(dest);
            let gain = 1.0 / fold_count.max(1) as f32;
            let mut source_channel = dest_channel;
            while source_channel < source {
                matrix[source_channel * dest + dest_channel] = gain;
                source_channel += dest;
            }
        }
    }
    matrix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_power_pan_extremes_and_center() {
        let left = equal_power_pan_matrix(-1.0);
        assert!((left[0] - 1.0).abs() < 1e-6);
        assert!(left[3].abs() < 1e-6);
        let right = equal_power_pan_matrix(1.0);
        assert!(right[0].abs() < 1e-6);
        assert!((right[3] - 1.0).abs() < 1e-6);
        let center = equal_power_pan_matrix(0.0);
        let minus_3db = std::f32::consts::FRAC_1_SQRT_2;
        assert!((center[0] - minus_3db).abs() < 1e-6);
        assert!((center[3] - minus_3db).abs() < 1e-6);
        // Cross terms stay zero: balance law, no channel bleed.
        assert_eq!(center[1], 0.0);
        assert_eq!(center[2], 0.0);
    }

    #[test]
    fn adapter_identity_when_counts_match() {
        assert_eq!(default_adapter_matrix(2, 2), vec![1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn adapter_mono_up_and_down() {
        let up = default_adapter_matrix(1, 2);
        assert!((up[0] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        assert_eq!(up[0], up[1]);
        let down = default_adapter_matrix(2, 1);
        assert_eq!(down, vec![0.5, 0.5]);
    }

    #[test]
    fn adapter_folds_wide_onto_narrow() {
        // 4 → 2: L = (s0 + s2)/2, R = (s1 + s3)/2.
        let matrix = default_adapter_matrix(4, 2);
        assert_eq!(matrix, vec![0.5, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.5]);
    }

    #[test]
    fn adapter_truncates_narrow_onto_wide() {
        // 2 → 4: identity into the first two channels, silence beyond.
        let matrix = default_adapter_matrix(2, 4);
        assert_eq!(matrix, vec![1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }
}
