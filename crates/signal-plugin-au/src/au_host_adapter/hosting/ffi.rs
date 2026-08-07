//! AudioToolbox / CoreFoundation FFI (macOS only).

#![allow(non_snake_case, non_upper_case_globals, dead_code)]
#![allow(missing_docs)]

use std::ffi::c_void;

pub type OSStatus = i32;
pub type Boolean = u8;
pub type AudioComponent = *mut c_void;
pub type AudioUnit = *mut c_void;
pub type CFTypeRef = *const c_void;
pub type CFStringRef = *const c_void;
pub type CFDataRef = *const c_void;
pub type CFPropertyListRef = *const c_void;
pub type CFAllocatorRef = *const c_void;
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
/// `kAudioUnitParameterUnit_Boolean` (off/on toggle).
pub const kAudioUnitParameterUnit_Boolean: u32 = 2;
pub const kAudioUnitParameterUnit_Percent: u32 = 3;
pub const kAudioUnitParameterUnit_Seconds: u32 = 4;
pub const kAudioUnitParameterUnit_Phase: u32 = 6;
pub const kAudioUnitParameterUnit_Rate: u32 = 7;
pub const kAudioUnitParameterUnit_Hertz: u32 = 8;
pub const kAudioUnitParameterUnit_Cents: u32 = 9;
pub const kAudioUnitParameterUnit_RelativeSemiTones: u32 = 10;
pub const kAudioUnitParameterUnit_Decibels: u32 = 13;
pub const kAudioUnitParameterUnit_Degrees: u32 = 15;
/// 0..100 crossfade (e.g. AUDelay's wet/dry mix); displays as percent.
pub const kAudioUnitParameterUnit_EqualPowerCrossfade: u32 = 16;
pub const kAudioUnitParameterUnit_AbsoluteCents: u32 = 20;
pub const kAudioUnitParameterUnit_Octaves: u32 = 21;
pub const kAudioUnitParameterUnit_BPM: u32 = 22;
pub const kAudioUnitParameterUnit_Beats: u32 = 23;
pub const kAudioUnitParameterUnit_Milliseconds: u32 = 24;
pub const kAudioUnitParameterUnit_Ratio: u32 = 25;
/// `kAudioUnitParameterUnit_CustomUnit`: `unitName` carries the label.
pub const kAudioUnitParameterUnit_CustomUnit: u32 = 26;

pub const kCFStringEncodingUTF8: u32 = 0x0800_0100;
pub const kAudioUnitProperty_ClassInfo: u32 = 0;
pub const kCFPropertyListBinaryFormat_v1_0: isize = 200;

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
    pub fn AudioComponentGetVersion(inComponent: AudioComponent, outVersion: *mut u32) -> OSStatus;
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
    pub fn MusicDeviceMIDIEvent(
        inUnit: AudioUnit,
        inStatus: u32,
        inData1: u32,
        inData2: u32,
        inOffsetSampleFrame: u32,
    ) -> OSStatus;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    pub static kCFAllocatorDefault: CFAllocatorRef;
    pub fn CFRelease(cf: CFTypeRef);
    pub fn CFDataCreate(allocator: CFAllocatorRef, bytes: *const u8, length: CFIndex) -> CFDataRef;
    pub fn CFDataGetBytePtr(data: CFDataRef) -> *const u8;
    pub fn CFDataGetLength(data: CFDataRef) -> CFIndex;
    pub fn CFPropertyListCreateData(
        allocator: CFAllocatorRef,
        propertyList: CFPropertyListRef,
        format: isize,
        options: usize,
        error: *mut CFTypeRef,
    ) -> CFDataRef;
    pub fn CFPropertyListCreateWithData(
        allocator: CFAllocatorRef,
        data: CFDataRef,
        options: usize,
        format: *mut isize,
        error: *mut CFTypeRef,
    ) -> CFPropertyListRef;
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
