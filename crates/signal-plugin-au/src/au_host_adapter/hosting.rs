//! In-child Audio Unit (AUv2) instance hosting: system-registry component
//! resolution, instance lifecycle (new/initialize/uninitialize/dispose),
//! parameter inventory via the AudioUnit property API, and a raw process
//! session for the sandbox audio thread — the AU mirror of
//! `signal-plugin-vst3`'s hosting module.
//!
//! # FFI design
//!
//! The AudioToolbox + CoreFoundation surface is handwritten C externs with
//! `#[link(name = "...", kind = "framework")]` on `cfg(target_os = "macos")`
//! blocks — no build script, no objc bridge, no SDK crates. Public types are
//! unconditional; only the FFI internals are platform-gated, and an
//! off-macOS `load` fails with the stable `unsupported_platform` token so
//! the sandbox/bridge/host layers stay cfg-free.
//!
//! # Pull-model rendering
//!
//! Unlike CLAP/VST3's push-model `process()`, an Audio Unit PULLS its input:
//! the host installs a render callback on the input scope and then asks the
//! unit to render via `AudioUnitRender`; mid-render the unit calls back into
//! the host to fetch the dry block. [`AuProcessSession`] stashes each
//! incoming block in preallocated planar buffers and serves it from an
//! `extern "C"` trampoline (boxed session state as `inRefCon`), handling
//! both callback buffer conventions: non-null `mData` (copy into the unit's
//! buffer) and null `mData` (point the unit at our stash). The render
//! timestamp advances monotonically (`mSampleTime` = session frame counter)
//! because time-based units (delays) misbehave on a static timestamp. Zero
//! allocation in the render path.

#![allow(unsafe_op_in_unsafe_fn)]

use std::path::Path;

use signal_plugin::{PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags};

use super::AuHostPlatform;

/// Sentinel `.component` path for registry-resolved AU entries: the file is
/// never opened — the load key alone rebuilds the `AudioComponentDescription`
/// and the system registry resolves the component. The `.component`
/// extension is what routes the sandbox broker to the AU hosting branch.
pub const AU_REGISTRY_COMPONENT_PATH: &str = "au-registry.component";

/// Error surface for AU hosting operations; carries a stable snake_case
/// token suitable for broker receipt details (mirrors `Vst3HostingError`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuHostingError {
    /// Stable snake_case failure token (e.g. `component_not_found`).
    pub token: String,
}

impl AuHostingError {
    fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl std::fmt::Display for AuHostingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.token)
    }
}

impl std::error::Error for AuHostingError {}

/// The build-target AU platform (macOS is the only one that can host).
pub const fn current_au_platform() -> AuHostPlatform {
    AuHostPlatform::MacOs
}

// ── FourCC codes ────────────────────────────────────────────────────────────

/// Encode a 4-character OSType code (`"aufx"`) as its big-endian `u32`.
/// `None` when the string is not exactly four ASCII bytes.
pub(crate) fn fourcc_from_str(code: &str) -> Option<u32> {
    let bytes = code.as_bytes();
    if bytes.len() != 4 || !bytes.iter().all(u8::is_ascii) {
        return None;
    }
    Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// Decode an OSType `u32` back into its 4-character code. Non-printable
/// bytes render as `?` (real registry OSTypes are printable ASCII).
pub(crate) fn fourcc_to_string(code: u32) -> String {
    code.to_be_bytes()
        .iter()
        .map(|byte| {
            if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '?'
            }
        })
        .collect()
}

/// Parse a catalog load key — the colon-separated fourcc triple
/// `{type}:{subtype}:{manufacturer}` (e.g. `aufx:dely:appl`) — into the
/// three OSType codes.
pub(crate) fn parse_load_key(load_key: &str) -> Option<(u32, u32, u32)> {
    let mut parts = load_key.split(':');
    let component_type = fourcc_from_str(parts.next()?)?;
    let component_subtype = fourcc_from_str(parts.next()?)?;
    let manufacturer = fourcc_from_str(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some((component_type, component_subtype, manufacturer))
}

// ── AudioToolbox / CoreFoundation FFI (macOS only) ─────────────────────────

#[cfg(target_os = "macos")]
pub(crate) mod ffi {
    #![allow(non_snake_case, non_upper_case_globals, dead_code)]
    #![allow(missing_docs)]

    use std::ffi::c_void;

    pub type OSStatus = i32;
    pub type Boolean = u8;
    pub type AudioComponent = *mut c_void;
    pub type AudioUnit = *mut c_void;
    pub type CFTypeRef = *const c_void;
    pub type CFStringRef = *const c_void;
    pub type CFIndex = isize;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct AudioComponentDescription {
        pub componentType: u32,
        pub componentSubType: u32,
        pub componentManufacturer: u32,
        pub componentFlags: u32,
        pub componentFlagsMask: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct AudioBuffer {
        pub mNumberChannels: u32,
        pub mDataByteSize: u32,
        pub mData: *mut c_void,
    }

    /// Fixed-stereo `AudioBufferList`: the C type ends in a variable-length
    /// `mBuffers[]` array; this is the two-buffer (planar stereo) shape the
    /// session preallocates and pointer-swizzles per block.
    #[repr(C)]
    pub struct StereoAudioBufferList {
        pub mNumberBuffers: u32,
        pub mBuffers: [AudioBuffer; 2],
    }

    /// Read-side view of a callback-provided `AudioBufferList` header; the
    /// trailing buffers are indexed by pointer arithmetic off `mBuffers`.
    #[repr(C)]
    pub struct RawAudioBufferList {
        pub mNumberBuffers: u32,
        pub mBuffers: [AudioBuffer; 1],
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct SMPTETime {
        pub mSubframes: i16,
        pub mSubframeDivisor: i16,
        pub mCounter: u32,
        pub mType: u32,
        pub mFlags: u32,
        pub mHours: i16,
        pub mMinutes: i16,
        pub mSeconds: i16,
        pub mFrames: i16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct AudioTimeStamp {
        pub mSampleTime: f64,
        pub mHostTime: u64,
        pub mRateScalar: f64,
        pub mWordClockTime: u64,
        pub mSMPTETime: SMPTETime,
        pub mFlags: u32,
        pub mReserved: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct AudioStreamBasicDescription {
        pub mSampleRate: f64,
        pub mFormatID: u32,
        pub mFormatFlags: u32,
        pub mBytesPerPacket: u32,
        pub mFramesPerPacket: u32,
        pub mBytesPerFrame: u32,
        pub mChannelsPerFrame: u32,
        pub mBitsPerChannel: u32,
        pub mReserved: u32,
    }

    pub type AURenderCallback = unsafe extern "C" fn(
        inRefCon: *mut c_void,
        ioActionFlags: *mut u32,
        inTimeStamp: *const AudioTimeStamp,
        inBusNumber: u32,
        inNumberFrames: u32,
        ioData: *mut RawAudioBufferList,
    ) -> OSStatus;

    #[repr(C)]
    pub struct AURenderCallbackStruct {
        pub inputProc: Option<AURenderCallback>,
        pub inputProcRefCon: *mut c_void,
    }

    /// `AudioUnitParameterInfo` (AUComponent.h layout).
    #[repr(C)]
    pub struct AudioUnitParameterInfo {
        pub name: [u8; 52],
        pub unitName: CFStringRef,
        pub clumpID: u32,
        pub cfNameString: CFStringRef,
        pub unit: u32,
        pub minValue: f32,
        pub maxValue: f32,
        pub defaultValue: f32,
        pub flags: u32,
    }

    impl AudioUnitParameterInfo {
        pub fn zeroed() -> Self {
            // Safety: all-zero bytes are a valid value for every field
            // (null CFStringRefs, zero numerics).
            unsafe { std::mem::zeroed() }
        }
    }

    // AudioUnit property/scope/format constants (AudioToolbox headers).
    pub const kAudioUnitProperty_ParameterList: u32 = 3;
    pub const kAudioUnitProperty_ParameterInfo: u32 = 4;
    pub const kAudioUnitProperty_StreamFormat: u32 = 8;
    pub const kAudioUnitProperty_MaximumFramesPerSlice: u32 = 14;
    pub const kAudioUnitProperty_SetRenderCallback: u32 = 23;

    pub const kAudioUnitScope_Global: u32 = 0;
    pub const kAudioUnitScope_Input: u32 = 1;
    pub const kAudioUnitScope_Output: u32 = 2;

    /// `'lpcm'`.
    pub const kAudioFormatLinearPCM: u32 = 0x6C70_636D;
    pub const kAudioFormatFlagIsFloat: u32 = 1 << 0;
    pub const kAudioFormatFlagIsPacked: u32 = 1 << 3;
    pub const kAudioFormatFlagIsNonInterleaved: u32 = 1 << 5;

    pub const kAudioTimeStampSampleTimeValid: u32 = 1 << 0;

    /// AUv3 components cannot be instantiated through
    /// `AudioComponentInstanceNew`; discovery filters them out.
    pub const kAudioComponentFlag_IsV3AudioUnit: u32 = 1 << 2;

    pub const kAudioUnitParameterFlag_CFNameRelease: u32 = 1 << 4;
    pub const kAudioUnitParameterFlag_HasCFNameString: u32 = 1 << 27;
    pub const kAudioUnitParameterFlag_IsReadable: u32 = 1 << 30;
    pub const kAudioUnitParameterFlag_IsWritable: u32 = 1 << 31;

    /// `kAudioUnitParameterUnit_Indexed` (discrete stepped values).
    pub const kAudioUnitParameterUnit_Indexed: u32 = 1;

    pub const kCFStringEncodingUTF8: u32 = 0x0800_0100;

    #[link(name = "AudioToolbox", kind = "framework")]
    extern "C" {
        pub fn AudioComponentCount(inDesc: *const AudioComponentDescription) -> u32;
        pub fn AudioComponentFindNext(
            inComponent: AudioComponent,
            inDesc: *const AudioComponentDescription,
        ) -> AudioComponent;
        pub fn AudioComponentGetDescription(
            inComponent: AudioComponent,
            outDesc: *mut AudioComponentDescription,
        ) -> OSStatus;
        pub fn AudioComponentCopyName(
            inComponent: AudioComponent,
            outName: *mut CFStringRef,
        ) -> OSStatus;
        pub fn AudioComponentGetVersion(
            inComponent: AudioComponent,
            outVersion: *mut u32,
        ) -> OSStatus;
        pub fn AudioComponentInstanceNew(
            inComponent: AudioComponent,
            outInstance: *mut AudioUnit,
        ) -> OSStatus;
        pub fn AudioComponentInstanceDispose(inInstance: AudioUnit) -> OSStatus;
        pub fn AudioUnitInitialize(inUnit: AudioUnit) -> OSStatus;
        pub fn AudioUnitUninitialize(inUnit: AudioUnit) -> OSStatus;
        pub fn AudioUnitSetProperty(
            inUnit: AudioUnit,
            inID: u32,
            inScope: u32,
            inElement: u32,
            inData: *const c_void,
            inDataSize: u32,
        ) -> OSStatus;
        pub fn AudioUnitGetProperty(
            inUnit: AudioUnit,
            inID: u32,
            inScope: u32,
            inElement: u32,
            outData: *mut c_void,
            ioDataSize: *mut u32,
        ) -> OSStatus;
        pub fn AudioUnitGetPropertyInfo(
            inUnit: AudioUnit,
            inID: u32,
            inScope: u32,
            inElement: u32,
            outDataSize: *mut u32,
            outWritable: *mut Boolean,
        ) -> OSStatus;
        pub fn AudioUnitRender(
            inUnit: AudioUnit,
            ioActionFlags: *mut u32,
            inTimeStamp: *const AudioTimeStamp,
            inOutputBusNumber: u32,
            inNumberFrames: u32,
            ioData: *mut StereoAudioBufferList,
        ) -> OSStatus;
        pub fn AudioUnitSetParameter(
            inUnit: AudioUnit,
            inID: u32,
            inScope: u32,
            inElement: u32,
            inValue: f32,
            inBufferOffsetInFrames: u32,
        ) -> OSStatus;
        pub fn AudioUnitReset(inUnit: AudioUnit, inScope: u32, inElement: u32) -> OSStatus;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        pub fn CFRelease(cf: CFTypeRef);
        pub fn CFStringGetLength(theString: CFStringRef) -> CFIndex;
        pub fn CFStringGetMaximumSizeForEncoding(length: CFIndex, encoding: u32) -> CFIndex;
        pub fn CFStringGetCString(
            theString: CFStringRef,
            buffer: *mut u8,
            bufferSize: CFIndex,
            encoding: u32,
        ) -> Boolean;
    }

    /// Copy a `CFStringRef` into a Rust `String` (UTF-8). Borrows the
    /// CFString — the caller keeps ownership (and any release duty).
    ///
    /// # Safety
    /// `string` must be a live `CFStringRef` or null (null returns `None`).
    pub unsafe fn cfstring_to_string(string: CFStringRef) -> Option<String> {
        if string.is_null() {
            return None;
        }
        let length = CFStringGetLength(string);
        let capacity = CFStringGetMaximumSizeForEncoding(length, kCFStringEncodingUTF8) + 1;
        let mut buffer = vec![0u8; capacity.max(1) as usize];
        if CFStringGetCString(
            string,
            buffer.as_mut_ptr(),
            buffer.len() as CFIndex,
            kCFStringEncodingUTF8,
        ) == 0
        {
            return None;
        }
        let end = buffer.iter().position(|byte| *byte == 0)?;
        buffer.truncate(end);
        String::from_utf8(buffer).ok()
    }

    /// Copy a `CFStringRef` into a Rust `String` and release it (consumes
    /// the caller's +1 reference, e.g. from a `Copy`-rule API).
    ///
    /// # Safety
    /// `string` must be an owned (+1) `CFStringRef` or null.
    pub unsafe fn cfstring_into_string(string: CFStringRef) -> Option<String> {
        let result = cfstring_to_string(string);
        if !string.is_null() {
            CFRelease(string);
        }
        result
    }
}

// ── Hosted instance ─────────────────────────────────────────────────────────

/// Main-element stereo port layout summary for a hosted AU instance
/// (mirrors `Vst3HostedPortLayout`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuHostedPortLayout {
    /// Channel count of input element 0 (0 = none, e.g. instruments).
    pub main_input_channels: u16,
    /// Channel count of output element 0 (0 = none).
    pub main_output_channels: u16,
}

impl AuHostedPortLayout {
    /// Phase 1 supports exactly a stereo main in + stereo main out effect.
    pub fn is_stereo_effect(&self) -> bool {
        self.main_input_channels == 2 && self.main_output_channels == 2
    }
}

/// Lifecycle state of a hosted instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostedInstanceState {
    Created,
    Active,
}

/// One live Audio Unit (AUv2) instance hosted in this process: owns the
/// `AudioComponentInstance` resolved from the system registry by fourcc
/// load key (`{type}:{subtype}:{manufacturer}`, e.g. `aufx:dely:appl`).
///
/// Threading: create/activate/deactivate/destroy run on the owning (main)
/// thread; audio processing runs through [`AuProcessSession`], which the
/// sandbox moves onto its audio thread. While a process session is live the
/// owner must not run lifecycle transitions until the session stops.
pub struct AuHostedInstance {
    #[cfg(target_os = "macos")]
    unit: ffi::AudioUnit,
    parameters: Vec<PluginParameterDescriptor>,
    port_layout: AuHostedPortLayout,
    state: HostedInstanceState,
    activated_max_frames: u32,
}

// Safety: the raw AudioUnit handle is only used through this type's public
// surface, which serializes lifecycle access per the type contract (main
// thread owns lifecycle; the session owns rendering).
unsafe impl Send for AuHostedInstance {}

impl AuHostedInstance {
    /// Resolve `load_key` (the fourcc triple `{type}:{subtype}:{manu}`)
    /// through the system AudioComponent registry and create the instance.
    ///
    /// `_bundle_root` keeps the (path, key) call shape of the CLAP/VST3
    /// hosting FFIs but is never opened: registry entries carry the
    /// [`AU_REGISTRY_COMPONENT_PATH`] sentinel and the component is resolved
    /// from the load key alone. Off macOS this fails with the stable
    /// `unsupported_platform` token.
    pub fn load(_bundle_root: &Path, load_key: &str) -> Result<Self, AuHostingError> {
        let (component_type, component_subtype, manufacturer) =
            parse_load_key(load_key).ok_or_else(|| AuHostingError::new("load_key_invalid"))?;
        Self::load_from_description(component_type, component_subtype, manufacturer)
    }

    #[cfg(target_os = "macos")]
    fn load_from_description(
        component_type: u32,
        component_subtype: u32,
        manufacturer: u32,
    ) -> Result<Self, AuHostingError> {
        let description = ffi::AudioComponentDescription {
            componentType: component_type,
            componentSubType: component_subtype,
            componentManufacturer: manufacturer,
            componentFlags: 0,
            componentFlagsMask: 0,
        };
        unsafe {
            let component = ffi::AudioComponentFindNext(std::ptr::null_mut(), &description);
            if component.is_null() {
                return Err(AuHostingError::new("component_not_found"));
            }
            let mut resolved = ffi::AudioComponentDescription::default();
            if ffi::AudioComponentGetDescription(component, &mut resolved) == 0
                && resolved.componentFlags & ffi::kAudioComponentFlag_IsV3AudioUnit != 0
            {
                return Err(AuHostingError::new("auv3_unsupported"));
            }
            let mut unit: ffi::AudioUnit = std::ptr::null_mut();
            if ffi::AudioComponentInstanceNew(component, &mut unit) != 0 || unit.is_null() {
                return Err(AuHostingError::new("instance_new_failed"));
            }
            let parameters = parameter_inventory(unit);
            let port_layout = main_element_layout(unit);
            Ok(Self {
                unit,
                parameters,
                port_layout,
                state: HostedInstanceState::Created,
                activated_max_frames: 0,
            })
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn load_from_description(
        _component_type: u32,
        _component_subtype: u32,
        _manufacturer: u32,
    ) -> Result<Self, AuHostingError> {
        Err(AuHostingError::new("unsupported_platform"))
    }

    /// Parameter inventory enumerated at load via
    /// `kAudioUnitProperty_ParameterList`/`ParameterInfo` (read-only
    /// phase 1).
    pub fn parameters(&self) -> &[PluginParameterDescriptor] {
        &self.parameters
    }

    /// Main-element port layout enumerated at load (default stream formats
    /// of input/output element 0).
    pub fn port_layout(&self) -> AuHostedPortLayout {
        self.port_layout
    }

    /// Set one parameter's plain value on the global scope
    /// (`AudioUnitSetParameter`). Phase 1 uses this from tests only; it is
    /// the natural phase-2 param-set entry point.
    pub fn set_parameter(&mut self, parameter_id: u32, value: f32) -> Result<(), AuHostingError> {
        #[cfg(target_os = "macos")]
        {
            let status = unsafe {
                ffi::AudioUnitSetParameter(
                    self.unit,
                    parameter_id,
                    ffi::kAudioUnitScope_Global,
                    0,
                    value,
                    0,
                )
            };
            if status != 0 {
                return Err(AuHostingError::new("set_parameter_failed"));
            }
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (parameter_id, value);
            Err(AuHostingError::new("unsupported_platform"))
        }
    }

    /// Activate for processing: f32 NON-INTERLEAVED stereo stream format on
    /// both scopes plus `MaximumFramesPerSlice`, all set BEFORE
    /// `AudioUnitInitialize`; the format is read back and verified — a unit
    /// that silently kept another layout is rejected with the stable
    /// `layout_unsupported` token, same as the CLAP/VST3 paths.
    pub fn activate(
        &mut self,
        sample_rate_hz: f64,
        _min_frames: u32,
        max_frames: u32,
    ) -> Result<(), AuHostingError> {
        if self.state == HostedInstanceState::Active {
            return Err(AuHostingError::new("already_active"));
        }
        #[cfg(target_os = "macos")]
        {
            let format = stereo_stream_format(sample_rate_hz);
            unsafe {
                // A unit may reject the set yet still satisfy the format, so
                // failures here fall through to the read-back verification.
                let _ = ffi::AudioUnitSetProperty(
                    self.unit,
                    ffi::kAudioUnitProperty_StreamFormat,
                    ffi::kAudioUnitScope_Input,
                    0,
                    &format as *const _ as *const _,
                    std::mem::size_of::<ffi::AudioStreamBasicDescription>() as u32,
                );
                let _ = ffi::AudioUnitSetProperty(
                    self.unit,
                    ffi::kAudioUnitProperty_StreamFormat,
                    ffi::kAudioUnitScope_Output,
                    0,
                    &format as *const _ as *const _,
                    std::mem::size_of::<ffi::AudioStreamBasicDescription>() as u32,
                );
                let max_frames_value: u32 = max_frames;
                if ffi::AudioUnitSetProperty(
                    self.unit,
                    ffi::kAudioUnitProperty_MaximumFramesPerSlice,
                    ffi::kAudioUnitScope_Global,
                    0,
                    &max_frames_value as *const _ as *const _,
                    std::mem::size_of::<u32>() as u32,
                ) != 0
                {
                    return Err(AuHostingError::new("max_frames_rejected"));
                }
                for scope in [ffi::kAudioUnitScope_Input, ffi::kAudioUnitScope_Output] {
                    if !verify_stereo_format(self.unit, scope, sample_rate_hz) {
                        return Err(AuHostingError::new("layout_unsupported"));
                    }
                }
                if ffi::AudioUnitInitialize(self.unit) != 0 {
                    return Err(AuHostingError::new("initialize_failed"));
                }
            }
            self.state = HostedInstanceState::Active;
            self.activated_max_frames = max_frames;
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (sample_rate_hz, max_frames);
            Err(AuHostingError::new("unsupported_platform"))
        }
    }

    /// Deactivate an active instance (no-op tokened error when inactive).
    pub fn deactivate(&mut self) -> Result<(), AuHostingError> {
        if self.state != HostedInstanceState::Active {
            return Err(AuHostingError::new("not_active"));
        }
        #[cfg(target_os = "macos")]
        unsafe {
            let _ = ffi::AudioUnitUninitialize(self.unit);
        }
        self.state = HostedInstanceState::Created;
        Ok(())
    }

    /// Build the raw process session for the sandbox audio thread. Only
    /// valid while active; the session preallocates its planar buffers at
    /// the activated max block size and installs the pull-model render
    /// callback, so processing never allocates.
    pub fn process_session(&self) -> Result<AuProcessSession, AuHostingError> {
        if self.state != HostedInstanceState::Active {
            return Err(AuHostingError::new("not_active"));
        }
        #[cfg(target_os = "macos")]
        {
            AuProcessSession::new(self.unit, self.activated_max_frames as usize)
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(AuHostingError::new("unsupported_platform"))
        }
    }
}

impl Drop for AuHostedInstance {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        unsafe {
            if self.state == HostedInstanceState::Active {
                let _ = ffi::AudioUnitUninitialize(self.unit);
            }
            let _ = ffi::AudioComponentInstanceDispose(self.unit);
        }
    }
}

#[cfg(target_os = "macos")]
fn stereo_stream_format(sample_rate_hz: f64) -> ffi::AudioStreamBasicDescription {
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
unsafe fn verify_stereo_format(unit: ffi::AudioUnit, scope: u32, sample_rate_hz: f64) -> bool {
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
unsafe fn main_element_layout(unit: ffi::AudioUnit) -> AuHostedPortLayout {
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

/// Enumerate the global-scope parameter inventory into Signal descriptors.
/// CFString names are copied out and released exactly per the
/// `kAudioUnitParameterFlag_CFNameRelease` contract.
#[cfg(target_os = "macos")]
unsafe fn parameter_inventory(unit: ffi::AudioUnit) -> Vec<PluginParameterDescriptor> {
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
        parameters.push(PluginParameterDescriptor {
            parameter_id: id,
            name,
            unit: None,
            domain: PluginParameterDomain::GenericNormalized,
            default_normalized,
            min_plain: info.minValue.min(info.maxValue),
            max_plain: info.maxValue.max(info.minValue),
            flags: PluginParameterFlags {
                automatable: writable,
                modulatable: false,
                supports_gesture: false,
                stepped: info.unit == ffi::kAudioUnitParameterUnit_Indexed,
                hidden: false,
                read_only: !writable,
            },
        });
    }
    parameters
}

#[cfg(target_os = "macos")]
fn c_name_field_to_string(field: &[u8]) -> Option<String> {
    let end = field.iter().position(|byte| *byte == 0)?;
    let text = String::from_utf8_lossy(&field[..end]).trim().to_string();
    (!text.is_empty()).then_some(text)
}

// ── Raw process session (audio thread) ──────────────────────────────────────

/// The dry-input stash served by the pull-model render callback. Boxed so
/// its address stays stable while [`AuProcessSession`] moves between
/// threads (the callback's `inRefCon` points here).
#[cfg(target_os = "macos")]
struct RenderInputState {
    left: Vec<f32>,
    right: Vec<f32>,
    /// Frames stashed for the in-flight `AudioUnitRender` call.
    frames: usize,
}

/// The input-scope render callback trampoline: serves the stashed dry block
/// into `ioData`. Two callback buffer conventions exist and both are
/// handled alloc-free: a non-null `mData` is the unit's own buffer (copy
/// into it); a null `mData` asks the host to point the unit at host-owned
/// memory (swizzle to the stash). Frames beyond the stash are zero-filled.
#[cfg(target_os = "macos")]
unsafe extern "C" fn render_input_trampoline(
    in_ref_con: *mut std::ffi::c_void,
    _io_action_flags: *mut u32,
    _in_time_stamp: *const ffi::AudioTimeStamp,
    _in_bus_number: u32,
    in_number_frames: u32,
    io_data: *mut ffi::RawAudioBufferList,
) -> ffi::OSStatus {
    if in_ref_con.is_null() || io_data.is_null() {
        return -50; // paramErr
    }
    let state = &mut *(in_ref_con as *mut RenderInputState);
    let buffer_count = ((*io_data).mNumberBuffers as usize).min(2);
    let buffers = (*io_data).mBuffers.as_mut_ptr();
    for index in 0..buffer_count {
        let source = if index == 0 {
            &mut state.left
        } else {
            &mut state.right
        };
        let requested = (in_number_frames as usize).min(source.len());
        let served = requested.min(state.frames);
        // Zero the unstashed tail so a long pull never reads stale samples.
        for slot in source[served..requested].iter_mut() {
            *slot = 0.0;
        }
        let buffer = &mut *buffers.add(index);
        if buffer.mData.is_null() {
            buffer.mData = source.as_mut_ptr() as *mut _;
            buffer.mDataByteSize = (requested * std::mem::size_of::<f32>()) as u32;
        } else {
            let capacity = (buffer.mDataByteSize as usize) / std::mem::size_of::<f32>();
            let count = requested.min(capacity);
            std::ptr::copy_nonoverlapping(source.as_ptr(), buffer.mData as *mut f32, count);
        }
        buffer.mNumberChannels = 1;
    }
    0
}

/// Raw, movable process handle for one activated AU instance: the
/// `AudioUnit` handle, the boxed dry-input stash (callback `inRefCon`),
/// planar output buffers, and a preallocated fixed-stereo
/// `AudioBufferList` — per block only the buffer pointers/sizes are
/// swizzled. The render timestamp's `mSampleTime` advances monotonically by
/// the processed frame count (time-based units misbehave on static
/// timestamps). The sandbox moves this onto its audio thread; the owning
/// [`AuHostedInstance`] must outlive it and must not run lifecycle
/// transitions while the session is live. Zero allocation in the render
/// path.
pub struct AuProcessSession {
    #[cfg(target_os = "macos")]
    unit: ffi::AudioUnit,
    #[cfg(target_os = "macos")]
    input: Box<RenderInputState>,
    #[cfg(target_os = "macos")]
    output_left: Vec<f32>,
    #[cfg(target_os = "macos")]
    output_right: Vec<f32>,
    #[cfg(target_os = "macos")]
    render_list: ffi::StereoAudioBufferList,
    #[cfg(target_os = "macos")]
    sample_time: f64,
    processing: bool,
}

// Safety: the session is handed to exactly one audio thread;
// `AudioUnitRender` is the AU render-thread entry point, the boxed input
// stash is only touched by the unit's synchronous mid-render pull on that
// same thread, and the owner serializes lifecycle against the session per
// the type contract (the vst3 session precedent).
unsafe impl Send for AuProcessSession {}

impl AuProcessSession {
    #[cfg(target_os = "macos")]
    fn new(unit: ffi::AudioUnit, max_frames: usize) -> Result<Self, AuHostingError> {
        let mut input = Box::new(RenderInputState {
            left: vec![0.0; max_frames],
            right: vec![0.0; max_frames],
            frames: 0,
        });
        let callback = ffi::AURenderCallbackStruct {
            inputProc: Some(render_input_trampoline),
            inputProcRefCon: &mut *input as *mut RenderInputState as *mut _,
        };
        let status = unsafe {
            ffi::AudioUnitSetProperty(
                unit,
                ffi::kAudioUnitProperty_SetRenderCallback,
                ffi::kAudioUnitScope_Input,
                0,
                &callback as *const _ as *const _,
                std::mem::size_of::<ffi::AURenderCallbackStruct>() as u32,
            )
        };
        if status != 0 {
            return Err(AuHostingError::new("render_callback_install_failed"));
        }
        Ok(Self {
            unit,
            input,
            output_left: vec![0.0; max_frames],
            output_right: vec![0.0; max_frames],
            render_list: ffi::StereoAudioBufferList {
                mNumberBuffers: 2,
                mBuffers: [
                    ffi::AudioBuffer {
                        mNumberChannels: 1,
                        mDataByteSize: 0,
                        mData: std::ptr::null_mut(),
                    },
                    ffi::AudioBuffer {
                        mNumberChannels: 1,
                        mDataByteSize: 0,
                        mData: std::ptr::null_mut(),
                    },
                ],
            },
            sample_time: 0.0,
            processing: false,
        })
    }

    /// Mark the session processing on the audio thread; must precede
    /// `process_*`. (AUv2 has no `setProcessing` equivalent — this is
    /// bookkeeping kept for surface parity with the CLAP/VST3 sessions.)
    pub fn start(&mut self) -> Result<(), AuHostingError> {
        self.processing = true;
        Ok(())
    }

    /// Mark the session stopped on the audio thread.
    pub fn stop(&mut self) {
        self.processing = false;
    }

    /// Whether `start()` has run and `stop()` has not yet.
    pub fn is_processing(&self) -> bool {
        self.processing
    }

    /// Pull one block from the unit via `AudioUnitRender` using the
    /// preallocated buffers. Returns `false` on unit error.
    ///
    /// # Safety
    /// `frames` must be within the preallocated buffer bounds (callers
    /// clamp).
    #[cfg(target_os = "macos")]
    unsafe fn render_block(&mut self, frames: usize) -> bool {
        self.input.frames = frames;
        // Per-block pointer swizzle only — the list itself is preallocated.
        let byte_size = (frames * std::mem::size_of::<f32>()) as u32;
        self.render_list.mBuffers[0].mData = self.output_left.as_mut_ptr() as *mut _;
        self.render_list.mBuffers[0].mDataByteSize = byte_size;
        self.render_list.mBuffers[0].mNumberChannels = 1;
        self.render_list.mBuffers[1].mData = self.output_right.as_mut_ptr() as *mut _;
        self.render_list.mBuffers[1].mDataByteSize = byte_size;
        self.render_list.mBuffers[1].mNumberChannels = 1;
        let timestamp = ffi::AudioTimeStamp {
            // Monotonically advancing sample time: delays keep their read
            // heads honest only when the timeline moves.
            mSampleTime: self.sample_time,
            mFlags: ffi::kAudioTimeStampSampleTimeValid,
            ..Default::default()
        };
        let mut action_flags: u32 = 0;
        let status = ffi::AudioUnitRender(
            self.unit,
            &mut action_flags,
            &timestamp,
            0,
            frames as u32,
            &mut self.render_list,
        );
        if status != 0 {
            return false;
        }
        self.sample_time += frames as f64;
        true
    }

    /// Read one rendered output sample; `AudioUnitRender` may point
    /// `mData` at the unit's own buffers instead of writing into ours, so
    /// output is always read back through the (possibly replaced) list
    /// pointers.
    ///
    /// # Safety
    /// Only valid after a successful `render_block(frames)` with
    /// `frame < frames`.
    #[cfg(target_os = "macos")]
    unsafe fn rendered_sample(&self, channel: usize, frame: usize) -> f32 {
        let data = self.render_list.mBuffers[channel].mData as *const f32;
        if data.is_null() {
            return 0.0;
        }
        *data.add(frame)
    }

    /// Process one block: interleaved stereo in, interleaved stereo out.
    /// Alloc-free (buffers preallocated at activate). On unit error the
    /// input passes through unchanged. Returns `false` on error.
    pub fn process_interleaved_stereo(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        frame_count: usize,
    ) -> bool {
        #[cfg(target_os = "macos")]
        {
            let frames = frame_count
                .min(self.input.left.len())
                .min(input.len() / 2)
                .min(output.len() / 2);
            for frame in 0..frames {
                self.input.left[frame] = input[frame * 2];
                self.input.right[frame] = input[frame * 2 + 1];
            }
            if !unsafe { self.render_block(frames) } {
                output[..frames * 2].copy_from_slice(&input[..frames * 2]);
                return false;
            }
            for frame in 0..frames {
                output[frame * 2] = unsafe { self.rendered_sample(0, frame) };
                output[frame * 2 + 1] = unsafe { self.rendered_sample(1, frame) };
            }
            true
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = frame_count;
            let samples = input.len().min(output.len());
            output[..samples].copy_from_slice(&input[..samples]);
            false
        }
    }

    /// In-place variant for the in-process isolation tier: processes the
    /// interleaved stereo buffer and writes the result back over it ONLY on
    /// success; on unit error the buffer is left untouched (bypass
    /// semantics). Alloc-free. `true` = buffer transformed.
    pub fn process_in_place(&mut self, io: &mut [f32], frame_count: usize) -> bool {
        #[cfg(target_os = "macos")]
        {
            let frames = frame_count.min(self.input.left.len()).min(io.len() / 2);
            for frame in 0..frames {
                self.input.left[frame] = io[frame * 2];
                self.input.right[frame] = io[frame * 2 + 1];
            }
            if !unsafe { self.render_block(frames) } {
                return false;
            }
            for frame in 0..frames {
                io[frame * 2] = unsafe { self.rendered_sample(0, frame) };
                io[frame * 2 + 1] = unsafe { self.rendered_sample(1, frame) };
            }
            true
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (io, frame_count);
            false
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for AuProcessSession {
    fn drop(&mut self) {
        // Uninstall the render callback so the unit can never pull from the
        // freed input stash after the session is gone.
        let callback = ffi::AURenderCallbackStruct {
            inputProc: None,
            inputProcRefCon: std::ptr::null_mut(),
        };
        unsafe {
            let _ = ffi::AudioUnitSetProperty(
                self.unit,
                ffi::kAudioUnitProperty_SetRenderCallback,
                ffi::kAudioUnitScope_Input,
                0,
                &callback as *const _ as *const _,
                std::mem::size_of::<ffi::AURenderCallbackStruct>() as u32,
            );
        }
    }
}

#[cfg(test)]
mod fourcc_tests {
    use super::*;

    #[test]
    fn fourcc_round_trips_printable_codes() {
        let code = fourcc_from_str("aufx").expect("aufx encodes");
        assert_eq!(code, u32::from_be_bytes(*b"aufx"));
        assert_eq!(fourcc_to_string(code), "aufx");
        assert!(fourcc_from_str("toolong").is_none());
        assert!(fourcc_from_str("ab").is_none());
    }

    #[test]
    fn load_key_parses_the_fourcc_triple() {
        let (component_type, subtype, manufacturer) =
            parse_load_key("aufx:dely:appl").expect("triple parses");
        assert_eq!(fourcc_to_string(component_type), "aufx");
        assert_eq!(fourcc_to_string(subtype), "dely");
        assert_eq!(fourcc_to_string(manufacturer), "appl");
        assert!(parse_load_key("aufx:dely").is_none());
        assert!(parse_load_key("aufx:dely:appl:extra").is_none());
        assert!(parse_load_key("nonsense").is_none());
    }

    fn error_token(result: Result<AuHostedInstance, AuHostingError>) -> String {
        match result {
            Ok(_) => panic!("load should have failed"),
            Err(error) => error.token,
        }
    }

    #[test]
    fn load_rejects_malformed_keys_with_a_stable_token() {
        let result = AuHostedInstance::load(Path::new(AU_REGISTRY_COMPONENT_PATH), "bogus");
        assert_eq!(error_token(result), "load_key_invalid");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn load_rejects_unknown_components_with_a_stable_token() {
        let result =
            AuHostedInstance::load(Path::new(AU_REGISTRY_COMPONENT_PATH), "zzzz:zzzz:zzzz");
        assert_eq!(error_token(result), "component_not_found");
    }
}
