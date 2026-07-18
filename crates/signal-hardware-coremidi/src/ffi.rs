//! Handwritten CoreMIDI (plus the sliver of CoreFoundation and Mach it
//! needs) FFI — the same posture as the AU/VST3 host adapters:
//! `#[link(name = "...", kind = "framework")]` on `cfg(target_os = "macos")`,
//! no binding crate, only the surface this backend actually calls.
//!
//! Legacy `MIDIReadProc`/`MIDIPacketList` path deliberately: it is the
//! MIDI 1.0 byte-stream API available on every supported macOS, and the
//! parser upstairs speaks MIDI 1.0 bytes.

#![allow(non_snake_case, non_upper_case_globals, missing_docs)]

use std::os::raw::c_void;

pub type OSStatus = i32;
pub type Boolean = u8;
pub type ItemCount = usize;
pub type CFIndex = isize;
pub type CFTypeRef = *const c_void;
pub type CFStringRef = *const c_void;
pub type CFAllocatorRef = *const c_void;
pub type CFRunLoopRef = *mut c_void;
pub type CFTimeInterval = f64;

pub type MIDIObjectRef = u32;
pub type MIDIClientRef = MIDIObjectRef;
pub type MIDIPortRef = MIDIObjectRef;
pub type MIDIEndpointRef = MIDIObjectRef;
pub type MIDITimeStamp = u64;

/// `MIDIObjectType` values (the ones we check).
pub const kMIDIObjectType_Source: i32 = 2;

/// `MIDINotificationMessageID` values (the ones we react to).
/// `kMIDIMsgObjectAdded` (= 2) is the device-return seam for the host's
/// auto-reopen pattern; the backend only reacts to removals today.
pub const kMIDIMsgObjectRemoved: i32 = 3;

pub const kCFStringEncodingUTF8: u32 = 0x0800_0100;

/// Header of every CoreMIDI notification; add/remove notifications extend it
/// as [`MIDIObjectAddRemoveNotification`].
#[repr(C)]
pub struct MIDINotification {
    pub messageID: i32,
    pub messageSize: u32,
}

/// Body of `kMIDIMsgObjectAdded` / `kMIDIMsgObjectRemoved` notifications.
#[repr(C)]
pub struct MIDIObjectAddRemoveNotification {
    pub messageID: i32,
    pub messageSize: u32,
    pub parent: MIDIObjectRef,
    pub parentType: i32,
    pub child: MIDIObjectRef,
    pub childType: i32,
}

pub type MIDINotifyProc = extern "C" fn(message: *const MIDINotification, refCon: *mut c_void);
pub type MIDIReadProc = extern "C" fn(
    pktlist: *const c_void, // *const MIDIPacketList — walked manually, see below
    readProcRefCon: *mut c_void,
    srcConnRefCon: *mut c_void,
);

// MIDIPacketList layout notes: the list is `numPackets: u32` followed by
// variable-length packets; each packet is `timeStamp: u64` at offset 0,
// `length: u16` at offset 8, `data: [u8; length]` at offset 10. On x86_64
// the structs are `#pragma pack(4)`; on arm64 they are naturally aligned and
// `MIDIPacketNext` rounds the next packet up to 4-byte alignment. We walk
// the list with raw offsets and unaligned reads instead of declaring a Rust
// struct whose layout would be wrong on one of the two architectures.

/// Offset of the first packet inside a `MIDIPacketList`.
pub const MIDI_PACKET_LIST_HEADER_BYTES: usize = 4;
/// Offset of `length` inside a `MIDIPacket`.
pub const MIDI_PACKET_LENGTH_OFFSET: usize = 8;
/// Offset of `data` inside a `MIDIPacket`.
pub const MIDI_PACKET_DATA_OFFSET: usize = 10;

/// Advance from one packet's data end to the next packet, mirroring the
/// architecture-specific `MIDIPacketNext` macro.
#[inline]
pub fn midi_packet_next(data_end: usize) -> usize {
    if cfg!(target_arch = "x86_64") {
        data_end
    } else {
        (data_end + 3) & !3
    }
}

#[link(name = "CoreMIDI", kind = "framework")]
extern "C" {
    pub static kMIDIPropertyName: CFStringRef;
    pub static kMIDIPropertyDisplayName: CFStringRef;
    pub static kMIDIPropertyManufacturer: CFStringRef;
    pub static kMIDIPropertyUniqueID: CFStringRef;

    pub fn MIDIClientCreate(
        name: CFStringRef,
        notifyProc: Option<MIDINotifyProc>,
        notifyRefCon: *mut c_void,
        outClient: *mut MIDIClientRef,
    ) -> OSStatus;
    pub fn MIDIClientDispose(client: MIDIClientRef) -> OSStatus;
    pub fn MIDIInputPortCreate(
        client: MIDIClientRef,
        portName: CFStringRef,
        readProc: MIDIReadProc,
        refCon: *mut c_void,
        outPort: *mut MIDIPortRef,
    ) -> OSStatus;
    pub fn MIDIPortConnectSource(
        port: MIDIPortRef,
        source: MIDIEndpointRef,
        connRefCon: *mut c_void,
    ) -> OSStatus;
    pub fn MIDIPortDisconnectSource(port: MIDIPortRef, source: MIDIEndpointRef) -> OSStatus;
    pub fn MIDIGetNumberOfSources() -> ItemCount;
    pub fn MIDIGetSource(sourceIndex0: ItemCount) -> MIDIEndpointRef;
    pub fn MIDIObjectGetStringProperty(
        obj: MIDIObjectRef,
        propertyID: CFStringRef,
        str_: *mut CFStringRef,
    ) -> OSStatus;
    pub fn MIDIObjectGetIntegerProperty(
        obj: MIDIObjectRef,
        propertyID: CFStringRef,
        outValue: *mut i32,
    ) -> OSStatus;
    pub fn MIDIObjectFindByUniqueID(
        inUniqueID: i32,
        outObject: *mut MIDIObjectRef,
        outObjectType: *mut i32,
    ) -> OSStatus;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    pub static kCFAllocatorDefault: CFAllocatorRef;
    pub static kCFRunLoopDefaultMode: CFStringRef;

    pub fn CFRelease(cf: CFTypeRef);
    pub fn CFStringCreateWithBytes(
        alloc: CFAllocatorRef,
        bytes: *const u8,
        numBytes: CFIndex,
        encoding: u32,
        isExternalRepresentation: Boolean,
    ) -> CFStringRef;
    pub fn CFStringGetLength(theString: CFStringRef) -> CFIndex;
    pub fn CFStringGetMaximumSizeForEncoding(length: CFIndex, encoding: u32) -> CFIndex;
    pub fn CFStringGetCString(
        theString: CFStringRef,
        buffer: *mut u8,
        bufferSize: CFIndex,
        encoding: u32,
    ) -> Boolean;
    pub fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    pub fn CFRunLoopStop(rl: CFRunLoopRef);
    pub fn CFRunLoopRunInMode(
        mode: CFStringRef,
        seconds: CFTimeInterval,
        returnAfterSourceHandled: Boolean,
    ) -> i32;
}

#[repr(C)]
pub struct MachTimebaseInfo {
    pub numer: u32,
    pub denom: u32,
}

// libSystem is linked by default; no #[link] needed for Mach time.
extern "C" {
    pub fn mach_absolute_time() -> u64;
    pub fn mach_timebase_info(info: *mut MachTimebaseInfo) -> i32;
}
