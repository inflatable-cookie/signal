use crate::ffi;

/// Prefix of every CoreMIDI `port_id`; the suffix is the source endpoint's
/// `kMIDIPropertyUniqueID` in decimal — CoreMIDI's persistent machine-scoped
/// identity, stable across unplug/replug and reboots on the same machine.
pub(crate) const PORT_ID_PREFIX: &str = "coremidi:";

/// Build an owned CFString from a Rust string. Caller releases.
pub(crate) unsafe fn cf_string(text: &str) -> ffi::CFStringRef {
    unsafe {
        ffi::CFStringCreateWithBytes(
            ffi::kCFAllocatorDefault,
            text.as_ptr(),
            text.len() as ffi::CFIndex,
            ffi::kCFStringEncodingUTF8,
            0,
        )
    }
}

/// Copy a CFString into a Rust string. Does NOT release `string`.
unsafe fn string_from_cf(string: ffi::CFStringRef) -> Option<String> {
    if string.is_null() {
        return None;
    }
    unsafe {
        let length = ffi::CFStringGetLength(string);
        let max_bytes =
            ffi::CFStringGetMaximumSizeForEncoding(length, ffi::kCFStringEncodingUTF8) + 1;
        let mut buffer = vec![0u8; max_bytes.max(1) as usize];
        if ffi::CFStringGetCString(
            string,
            buffer.as_mut_ptr(),
            buffer.len() as ffi::CFIndex,
            ffi::kCFStringEncodingUTF8,
        ) == 0
        {
            return None;
        }
        let end = buffer.iter().position(|&byte| byte == 0)?;
        buffer.truncate(end);
        String::from_utf8(buffer).ok()
    }
}

/// Read a string property off a MIDI object, when present.
pub(crate) fn string_property(
    object: ffi::MIDIObjectRef,
    property: ffi::CFStringRef,
) -> Option<String> {
    unsafe {
        let mut value: ffi::CFStringRef = std::ptr::null();
        if ffi::MIDIObjectGetStringProperty(object, property, &mut value) != 0 || value.is_null() {
            return None;
        }
        let text = string_from_cf(value);
        ffi::CFRelease(value);
        text
    }
}

/// Read an integer property off a MIDI object, when present.
pub(crate) fn integer_property(
    object: ffi::MIDIObjectRef,
    property: ffi::CFStringRef,
) -> Option<i32> {
    unsafe {
        let mut value: i32 = 0;
        if ffi::MIDIObjectGetIntegerProperty(object, property, &mut value) != 0 {
            return None;
        }
        Some(value)
    }
}

/// Parse a `coremidi:<unique-id>` port id back to the CoreMIDI unique id.
pub(crate) fn parse_port_id(port_id: &str) -> Option<i32> {
    port_id.strip_prefix(PORT_ID_PREFIX)?.parse().ok()
}
