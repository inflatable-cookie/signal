//! Minimal ARA document binding used by the isolated plugin inspector.
//!
//! This deliberately does not model audio sources, regions, transport, or
//! persistence. It gives an ARA editor-view instance an empty document so an
//! inspector can create its native UI. Full ARA hosting belongs in a richer
//! host layer, not in this inspection shim.

#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_void};
use std::mem;
use std::ptr;

use super::hosting::{
    com_query_interface, com_release, tuid_from_uid, vtable_of, Tresult, Tuid, Vst3HostingError,
};

// IIDs from Celemony's Apache-2.0 ARA API `ARAVST3.h`.
const ARA_PLUGIN_ENTRY_POINT_IID: Tuid =
    tuid_from_uid(0x12814E54, 0xA1CE4076, 0x82B96813, 0x16950BD6);
const ARA_PLUGIN_ENTRY_POINT_2_IID: Tuid =
    tuid_from_uid(0xCD9A5913, 0xC9EB46D7, 0x96CA53AD, 0xD1DB89F5);

const ARA_EDITOR_VIEW_ROLE: i32 = 1 << 2;
const ARA_KNOWN_ROLES: i32 = (1 << 0) | (1 << 1) | ARA_EDITOR_VIEW_ROLE;
const ARA_HIGHEST_SUPPORTED_GENERATION: i32 = 6; // ARA 2.3 Final

type AraRef = *mut c_void;
type AraAssertFunction = Option<
    unsafe extern "C" fn(
        category: i32,
        problematic_argument: *const c_void,
        diagnosis: *const c_char,
    ),
>;

static ARA_ASSERT_FUNCTION: AraAssertFunction = None;
static INSPECTION_DOCUMENT_NAME: &[u8] = b"Soundcheck plugin inspection\0";

#[repr(C)]
struct AraPluginEntryPointVTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    get_factory: unsafe extern "C" fn(*mut c_void) -> *const AraFactory,
    bind_to_document_controller:
        unsafe extern "C" fn(*mut c_void, AraRef) -> *const AraPluginExtensionInstance,
}

#[repr(C)]
struct AraPluginEntryPoint2VTable {
    query_interface: unsafe extern "C" fn(*mut c_void, *const Tuid, *mut *mut c_void) -> Tresult,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
    bind_to_document_controller_with_roles:
        unsafe extern "C" fn(*mut c_void, AraRef, i32, i32) -> *const AraPluginExtensionInstance,
}

#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), repr(C, packed))]
#[cfg_attr(not(any(target_arch = "x86", target_arch = "x86_64")), repr(C))]
struct AraInterfaceConfiguration {
    struct_size: usize,
    desired_api_generation: i32,
    assert_function_address: *const AraAssertFunction,
}

#[repr(C)]
struct AraFactory {
    struct_size: usize,
    lowest_supported_api_generation: i32,
    highest_supported_api_generation: i32,
    factory_id: *const c_char,
    initialize_ara_with_configuration:
        Option<unsafe extern "C" fn(*const AraInterfaceConfiguration)>,
    uninitialize_ara: Option<unsafe extern "C" fn()>,
    plugin_name: *const c_char,
    manufacturer_name: *const c_char,
    information_url: *const c_char,
    version: *const c_char,
    create_document_controller_with_document: Option<
        unsafe extern "C" fn(
            *const AraDocumentControllerHostInstance,
            *const AraDocumentProperties,
        ) -> *const AraDocumentControllerInstance,
    >,
}

#[repr(C)]
struct AraDocumentProperties {
    struct_size: usize,
    name: *const c_char,
}

#[repr(C)]
struct AraAudioAccessControllerInterface {
    struct_size: usize,
    create_audio_reader_for_source: unsafe extern "C" fn(AraRef, AraRef, i32) -> AraRef,
    read_audio_samples: unsafe extern "C" fn(AraRef, AraRef, i64, i64, *mut *mut c_void) -> i32,
    destroy_audio_reader: unsafe extern "C" fn(AraRef, AraRef),
}

#[repr(C)]
struct AraArchivingControllerInterface {
    struct_size: usize,
    get_archive_size: unsafe extern "C" fn(AraRef, AraRef) -> usize,
    read_bytes_from_archive: unsafe extern "C" fn(AraRef, AraRef, usize, usize, *mut u8) -> i32,
    write_bytes_to_archive: unsafe extern "C" fn(AraRef, AraRef, usize, usize, *const u8) -> i32,
    notify_document_archiving_progress: unsafe extern "C" fn(AraRef, f32),
    notify_document_unarchiving_progress: unsafe extern "C" fn(AraRef, f32),
    get_document_archive_id: unsafe extern "C" fn(AraRef, AraRef) -> *const c_char,
}

#[repr(C)]
struct AraDocumentControllerHostInstance {
    struct_size: usize,
    audio_access_controller_host_ref: AraRef,
    audio_access_controller_interface: *const AraAudioAccessControllerInterface,
    archiving_controller_host_ref: AraRef,
    archiving_controller_interface: *const AraArchivingControllerInterface,
    content_access_controller_host_ref: AraRef,
    content_access_controller_interface: *const c_void,
    model_update_controller_host_ref: AraRef,
    model_update_controller_interface: *const c_void,
    playback_controller_host_ref: AraRef,
    playback_controller_interface: *const c_void,
}

#[repr(C)]
struct AraDocumentControllerInstance {
    struct_size: usize,
    document_controller_ref: AraRef,
    document_controller_interface: *const AraDocumentControllerInterface,
}

#[repr(C)]
struct AraDocumentControllerInterface {
    struct_size: usize,
    destroy_document_controller: unsafe extern "C" fn(AraRef),
}

#[repr(C)]
struct AraPluginExtensionInstance {
    struct_size: usize,
}

struct AraHostStorage {
    audio_access: AraAudioAccessControllerInterface,
    archiving: AraArchivingControllerInterface,
    host_instance: AraDocumentControllerHostInstance,
}

impl AraHostStorage {
    fn new() -> Box<Self> {
        let mut storage = Box::new(Self {
            audio_access: AraAudioAccessControllerInterface {
                struct_size: mem::size_of::<AraAudioAccessControllerInterface>(),
                create_audio_reader_for_source: ara_create_audio_reader,
                read_audio_samples: ara_read_audio_samples,
                destroy_audio_reader: ara_destroy_audio_reader,
            },
            archiving: AraArchivingControllerInterface {
                struct_size: mem::size_of::<AraArchivingControllerInterface>(),
                get_archive_size: ara_get_archive_size,
                read_bytes_from_archive: ara_read_archive,
                write_bytes_to_archive: ara_write_archive,
                notify_document_archiving_progress: ara_archive_progress,
                notify_document_unarchiving_progress: ara_archive_progress,
                get_document_archive_id: ara_get_document_archive_id,
            },
            host_instance: AraDocumentControllerHostInstance {
                struct_size: mem::size_of::<AraDocumentControllerHostInstance>(),
                audio_access_controller_host_ref: ptr::null_mut(),
                audio_access_controller_interface: ptr::null(),
                archiving_controller_host_ref: ptr::null_mut(),
                archiving_controller_interface: ptr::null(),
                content_access_controller_host_ref: ptr::null_mut(),
                content_access_controller_interface: ptr::null(),
                model_update_controller_host_ref: ptr::null_mut(),
                model_update_controller_interface: ptr::null(),
                playback_controller_host_ref: ptr::null_mut(),
                playback_controller_interface: ptr::null(),
            },
        });
        let host_ref = (&mut *storage as *mut Self).cast();
        storage.host_instance.audio_access_controller_host_ref = host_ref;
        storage.host_instance.audio_access_controller_interface = &storage.audio_access;
        storage.host_instance.archiving_controller_host_ref = host_ref;
        storage.host_instance.archiving_controller_interface = &storage.archiving;
        storage
    }
}

/// Owns the minimal ARA document for as long as the companion VST3 instance
/// is used. It is destroyed before the module is unloaded.
pub(super) struct AraInspectionSession {
    factory: *const AraFactory,
    document_controller: *const AraDocumentControllerInstance,
    _host: Box<AraHostStorage>,
}

impl AraInspectionSession {
    /// Bind `component` to an empty ARA document when it exposes the ARA VST3
    /// entry points. `Ok(None)` means this is an ordinary VST3 component.
    pub(super) unsafe fn try_bind(
        component: *mut c_void,
    ) -> Result<Option<Self>, Vst3HostingError> {
        let Some(entry) = com_query_interface(component, &ARA_PLUGIN_ENTRY_POINT_IID) else {
            return Ok(None);
        };
        let entry2 = com_query_interface(component, &ARA_PLUGIN_ENTRY_POINT_2_IID);
        let entry_vtable = vtable_of::<AraPluginEntryPointVTable>(entry);
        let factory = ((*entry_vtable).get_factory)(entry);
        if factory.is_null() {
            if let Some(entry2) = entry2 {
                com_release(entry2);
            }
            com_release(entry);
            return Ok(None);
        }

        let lowest = ptr::addr_of!((*factory).lowest_supported_api_generation).read_unaligned();
        let highest = ptr::addr_of!((*factory).highest_supported_api_generation).read_unaligned();
        let generation = highest.min(ARA_HIGHEST_SUPPORTED_GENERATION);
        if generation < lowest {
            if let Some(entry2) = entry2 {
                com_release(entry2);
            }
            com_release(entry);
            return Err(Vst3HostingError::new("ara_generation_unsupported"));
        }
        let initialize = ptr::addr_of!((*factory).initialize_ara_with_configuration)
            .read_unaligned()
            .ok_or_else(|| Vst3HostingError::new("ara_initialize_missing"))?;
        let uninitialize = ptr::addr_of!((*factory).uninitialize_ara)
            .read_unaligned()
            .ok_or_else(|| Vst3HostingError::new("ara_uninitialize_missing"))?;
        let create_document = ptr::addr_of!((*factory).create_document_controller_with_document)
            .read_unaligned()
            .ok_or_else(|| Vst3HostingError::new("ara_document_factory_missing"))?;

        let config = AraInterfaceConfiguration {
            struct_size: mem::size_of::<AraInterfaceConfiguration>(),
            desired_api_generation: generation,
            assert_function_address: &ARA_ASSERT_FUNCTION,
        };
        initialize(&config);

        let host = AraHostStorage::new();
        let properties = AraDocumentProperties {
            struct_size: mem::size_of::<AraDocumentProperties>(),
            name: INSPECTION_DOCUMENT_NAME.as_ptr().cast(),
        };
        let document_controller = create_document(&host.host_instance, &properties);
        if document_controller.is_null()
            || (*document_controller).document_controller_ref.is_null()
            || (*document_controller)
                .document_controller_interface
                .is_null()
        {
            uninitialize();
            if let Some(entry2) = entry2 {
                com_release(entry2);
            }
            com_release(entry);
            return Err(Vst3HostingError::new("ara_document_create_failed"));
        }

        let document_ref = (*document_controller).document_controller_ref;
        let extension = if let Some(entry2) = entry2 {
            let vtable = vtable_of::<AraPluginEntryPoint2VTable>(entry2);
            let result = ((*vtable).bind_to_document_controller_with_roles)(
                entry2,
                document_ref,
                ARA_KNOWN_ROLES,
                ARA_EDITOR_VIEW_ROLE,
            );
            com_release(entry2);
            result
        } else {
            ((*entry_vtable).bind_to_document_controller)(entry, document_ref)
        };
        com_release(entry);

        if extension.is_null() {
            let interface = (*document_controller).document_controller_interface;
            ((*interface).destroy_document_controller)(document_ref);
            uninitialize();
            return Err(Vst3HostingError::new("ara_bind_failed"));
        }

        Ok(Some(Self {
            factory,
            document_controller,
            _host: host,
        }))
    }
}

impl Drop for AraInspectionSession {
    fn drop(&mut self) {
        unsafe {
            if !self.document_controller.is_null() {
                let document_ref = (*self.document_controller).document_controller_ref;
                let interface = (*self.document_controller).document_controller_interface;
                if !document_ref.is_null() && !interface.is_null() {
                    ((*interface).destroy_document_controller)(document_ref);
                }
            }
            if !self.factory.is_null() {
                if let Some(uninitialize) =
                    ptr::addr_of!((*self.factory).uninitialize_ara).read_unaligned()
                {
                    uninitialize();
                }
            }
        }
    }
}

unsafe extern "C" fn ara_create_audio_reader(_host: AraRef, _source: AraRef, _f64: i32) -> AraRef {
    ptr::null_mut()
}

unsafe extern "C" fn ara_read_audio_samples(
    _host: AraRef,
    _reader: AraRef,
    _position: i64,
    _samples: i64,
    _buffers: *mut *mut c_void,
) -> i32 {
    0
}

unsafe extern "C" fn ara_destroy_audio_reader(_host: AraRef, _reader: AraRef) {}

unsafe extern "C" fn ara_get_archive_size(_host: AraRef, _reader: AraRef) -> usize {
    0
}

unsafe extern "C" fn ara_read_archive(
    _host: AraRef,
    _reader: AraRef,
    _position: usize,
    _length: usize,
    _buffer: *mut u8,
) -> i32 {
    0
}

unsafe extern "C" fn ara_write_archive(
    _host: AraRef,
    _writer: AraRef,
    _position: usize,
    _length: usize,
    _buffer: *const u8,
) -> i32 {
    0
}

unsafe extern "C" fn ara_archive_progress(_host: AraRef, _progress: f32) {}

unsafe extern "C" fn ara_get_document_archive_id(_host: AraRef, _reader: AraRef) -> *const c_char {
    ptr::null()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    struct FakeAraEntry {
        vtable: *const AraPluginEntryPointVTable,
    }

    unsafe extern "C" fn query_interface(
        object: *mut c_void,
        iid: *const Tuid,
        result: *mut *mut c_void,
    ) -> Tresult {
        if !iid.is_null() && *iid == ARA_PLUGIN_ENTRY_POINT_IID {
            *result = object;
            0
        } else {
            *result = ptr::null_mut();
            1
        }
    }

    unsafe extern "C" fn add_ref(_object: *mut c_void) -> u32 {
        1
    }

    unsafe extern "C" fn release(_object: *mut c_void) -> u32 {
        1
    }

    unsafe extern "C" fn missing_factory(_object: *mut c_void) -> *const AraFactory {
        ptr::null()
    }

    unsafe extern "C" fn bind(
        _object: *mut c_void,
        _document: AraRef,
    ) -> *const AraPluginExtensionInstance {
        ptr::null()
    }

    static FAKE_ENTRY_VTABLE: AraPluginEntryPointVTable = AraPluginEntryPointVTable {
        query_interface,
        add_ref,
        release,
        get_factory: missing_factory,
        bind_to_document_controller: bind,
    };

    #[test]
    fn ara_entry_without_factory_falls_back_to_ordinary_vst3_loading() {
        let mut entry = FakeAraEntry {
            vtable: &FAKE_ENTRY_VTABLE,
        };

        let session =
            unsafe { AraInspectionSession::try_bind((&mut entry as *mut FakeAraEntry).cast()) }
                .expect("optional ARA probe");

        assert!(session.is_none());
    }
}
