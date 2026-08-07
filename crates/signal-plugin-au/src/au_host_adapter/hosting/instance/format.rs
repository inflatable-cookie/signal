#[cfg(target_os = "macos")]
use super::super::ffi;
use super::layout::AuHostedPortLayout;

#[cfg(target_os = "macos")]
pub(crate) fn stereo_stream_format(sample_rate_hz: f64) -> ffi::AudioStreamBasicDescription {
    ffi::AudioStreamBasicDescription {
        mSampleRate: sample_rate_hz,
        mFormatID: ffi::kAudioFormatLinearPCM,
        mFormatFlags: ffi::kAudioFormatFlagIsFloat
            | ffi::kAudioFormatFlagIsPacked
            | ffi::kAudioFormatFlagIsNonInterleaved,
        // Non-interleaved: per-channel packets/frames of one f32 each.
        mBytesPerPacket: 4,
        mFramesPerPacket: 1,
        mBytesPerFrame: 4,
        mChannelsPerFrame: 2,
        mBitsPerChannel: 32,
        mReserved: 0,
    }
}

/// Read back the negotiated stream format on `scope` element 0 and check it
/// is the f32 non-interleaved stereo layout at the requested rate.
#[cfg(target_os = "macos")]
pub(crate) unsafe fn verify_stereo_format(
    unit: ffi::AudioUnit,
    scope: u32,
    sample_rate_hz: f64,
) -> bool {
    let mut format = ffi::AudioStreamBasicDescription::default();
    let mut size = std::mem::size_of::<ffi::AudioStreamBasicDescription>() as u32;
    if ffi::AudioUnitGetProperty(
        unit,
        ffi::kAudioUnitProperty_StreamFormat,
        scope,
        0,
        &mut format as *mut _ as *mut _,
        &mut size,
    ) != 0
    {
        return false;
    }
    format.mFormatID == ffi::kAudioFormatLinearPCM
        && format.mFormatFlags & ffi::kAudioFormatFlagIsFloat != 0
        && format.mFormatFlags & ffi::kAudioFormatFlagIsNonInterleaved != 0
        && format.mChannelsPerFrame == 2
        && format.mBitsPerChannel == 32
        && (format.mSampleRate - sample_rate_hz).abs() < 1.0
}

/// Channel counts of the default stream formats on input/output element 0
/// (instruments report zero input elements → 0 input channels).
#[cfg(target_os = "macos")]
pub(crate) unsafe fn main_element_layout(unit: ffi::AudioUnit) -> AuHostedPortLayout {
    let mut layout = AuHostedPortLayout {
        main_input_channels: 0,
        main_output_channels: 0,
    };
    for (scope, slot) in [
        (ffi::kAudioUnitScope_Input, &mut layout.main_input_channels),
        (
            ffi::kAudioUnitScope_Output,
            &mut layout.main_output_channels,
        ),
    ] {
        let mut format = ffi::AudioStreamBasicDescription::default();
        let mut size = std::mem::size_of::<ffi::AudioStreamBasicDescription>() as u32;
        if ffi::AudioUnitGetProperty(
            unit,
            ffi::kAudioUnitProperty_StreamFormat,
            scope,
            0,
            &mut format as *mut _ as *mut _,
            &mut size,
        ) == 0
        {
            *slot = format.mChannelsPerFrame.min(u16::MAX as u32) as u16;
        }
    }
    layout
}
