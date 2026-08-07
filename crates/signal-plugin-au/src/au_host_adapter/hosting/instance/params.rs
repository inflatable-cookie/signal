use signal_plugin::{PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags};

#[cfg(target_os = "macos")]
use super::super::ffi;

/// Enumerate the global-scope parameter inventory into Signal descriptors.
/// CFString names are copied out and released exactly per the
/// `kAudioUnitParameterFlag_CFNameRelease` contract.
#[cfg(target_os = "macos")]
pub(crate) unsafe fn parameter_inventory(unit: ffi::AudioUnit) -> Vec<PluginParameterDescriptor> {
    let mut list_bytes = 0u32;
    let mut writable: ffi::Boolean = 0;
    if ffi::AudioUnitGetPropertyInfo(
        unit,
        ffi::kAudioUnitProperty_ParameterList,
        ffi::kAudioUnitScope_Global,
        0,
        &mut list_bytes,
        &mut writable,
    ) != 0
        || list_bytes == 0
    {
        return Vec::new();
    }
    let mut ids = vec![0u32; list_bytes as usize / std::mem::size_of::<u32>()];
    let mut io_bytes = list_bytes;
    if ffi::AudioUnitGetProperty(
        unit,
        ffi::kAudioUnitProperty_ParameterList,
        ffi::kAudioUnitScope_Global,
        0,
        ids.as_mut_ptr() as *mut _,
        &mut io_bytes,
    ) != 0
    {
        return Vec::new();
    }
    ids.truncate(io_bytes as usize / std::mem::size_of::<u32>());

    let mut parameters = Vec::with_capacity(ids.len());
    for id in ids {
        let mut info = ffi::AudioUnitParameterInfo::zeroed();
        let mut info_bytes = std::mem::size_of::<ffi::AudioUnitParameterInfo>() as u32;
        if ffi::AudioUnitGetProperty(
            unit,
            ffi::kAudioUnitProperty_ParameterInfo,
            ffi::kAudioUnitScope_Global,
            id,
            &mut info as *mut _ as *mut _,
            &mut info_bytes,
        ) != 0
        {
            continue;
        }
        let cf_name = if info.flags & ffi::kAudioUnitParameterFlag_HasCFNameString != 0 {
            let name = ffi::cfstring_to_string(info.cfNameString);
            if info.flags & ffi::kAudioUnitParameterFlag_CFNameRelease != 0
                && !info.cfNameString.is_null()
            {
                ffi::CFRelease(info.cfNameString);
            }
            name
        } else {
            None
        };
        let name = cf_name
            .or_else(|| c_name_field_to_string(&info.name))
            .unwrap_or_else(|| format!("Param {id}"));
        let range = info.maxValue - info.minValue;
        let default_normalized = if range.abs() > f32::EPSILON {
            ((info.defaultValue - info.minValue) / range).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let writable = info.flags & ffi::kAudioUnitParameterFlag_IsWritable != 0;
        // Indexed and Boolean units take discrete integer values; the step
        // count is the integer span of the range (g12.013).
        let stepped = matches!(
            info.unit,
            ffi::kAudioUnitParameterUnit_Indexed | ffi::kAudioUnitParameterUnit_Boolean
        );
        let step_count = stepped.then(|| (range.round().abs() as u32).max(1));
        parameters.push(PluginParameterDescriptor {
            parameter_id: id,
            name,
            unit: parameter_unit_label(&info),
            domain: PluginParameterDomain::GenericNormalized,
            default_normalized,
            min_plain: info.minValue.min(info.maxValue),
            max_plain: info.maxValue.max(info.minValue),
            step_count,
            flags: PluginParameterFlags {
                automatable: writable,
                modulatable: false,
                supports_gesture: false,
                stepped,
                hidden: false,
                read_only: !writable,
            },
        });
    }
    parameters
}

/// Map `AudioUnitParameterInfo.unit` to a display label. Well-known
/// AudioToolbox unit enums map to conventional short strings; a
/// `CustomUnit` copies the plugin-provided `unitName` (read-only borrow —
/// AudioToolbox owns the CFString for the info's lifetime, mirroring how
/// other hosts consume it). Unknown or unitless enums report `None` —
/// never synthesized.
#[cfg(target_os = "macos")]
unsafe fn parameter_unit_label(info: &ffi::AudioUnitParameterInfo) -> Option<String> {
    let label = match info.unit {
        ffi::kAudioUnitParameterUnit_Percent | ffi::kAudioUnitParameterUnit_EqualPowerCrossfade => {
            "%"
        }
        ffi::kAudioUnitParameterUnit_Seconds => "s",
        ffi::kAudioUnitParameterUnit_Milliseconds => "ms",
        ffi::kAudioUnitParameterUnit_Hertz => "Hz",
        ffi::kAudioUnitParameterUnit_Decibels => "dB",
        ffi::kAudioUnitParameterUnit_Cents | ffi::kAudioUnitParameterUnit_AbsoluteCents => "cents",
        ffi::kAudioUnitParameterUnit_RelativeSemiTones => "semitones",
        ffi::kAudioUnitParameterUnit_Octaves => "oct",
        ffi::kAudioUnitParameterUnit_BPM => "BPM",
        ffi::kAudioUnitParameterUnit_Beats => "beats",
        ffi::kAudioUnitParameterUnit_Phase | ffi::kAudioUnitParameterUnit_Degrees => "deg",
        ffi::kAudioUnitParameterUnit_Rate | ffi::kAudioUnitParameterUnit_Ratio => "x",
        ffi::kAudioUnitParameterUnit_CustomUnit => {
            return ffi::cfstring_to_string(info.unitName)
                .map(|name| name.trim().to_string())
                .filter(|name| !name.is_empty());
        }
        _ => return None,
    };
    Some(label.to_string())
}

#[cfg(target_os = "macos")]
fn c_name_field_to_string(field: &[u8]) -> Option<String> {
    let end = field.iter().position(|byte| *byte == 0)?;
    let text = String::from_utf8_lossy(&field[..end]).trim().to_string();
    (!text.is_empty()).then_some(text)
}
