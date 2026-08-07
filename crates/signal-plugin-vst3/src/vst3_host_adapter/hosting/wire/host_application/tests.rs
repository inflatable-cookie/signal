use std::ffi::CStr;
use std::path::Path;
use std::ptr;

use super::super::com::*;
use super::super::stream::vtable_of;
use super::application::host_create_instance;
use super::attribute_list::{host_attribute_list_release, HostAttributeListVTable};
use super::message::HostMessageVTable;
use super::*;

#[test]
fn skips_factory_context_for_application_private_bundle_components() {
    assert!(!should_set_factory_host_context(Path::new(
        "/Applications/Cubase.app/Contents/Components/Modulation FX.bundle"
    )));
    assert!(should_set_factory_host_context(Path::new(
        "/Library/Audio/Plug-Ins/VST3/Example.vst3"
    )));
}

#[test]
fn creates_messages_with_writable_attributes() {
    unsafe {
        let mut cid = IMESSAGE_IID;
        let mut iid = IMESSAGE_IID;
        let mut message = ptr::null_mut();
        assert_eq!(
            host_create_instance(
                host_context(),
                cid.as_mut_ptr(),
                iid.as_mut_ptr(),
                &mut message,
            ),
            K_RESULT_OK
        );
        assert!(!message.is_null());

        let message_vtable = vtable_of::<HostMessageVTable>(message);
        let message_id = c"slate-ui-message";
        ((*message_vtable).set_message_id)(message, message_id.as_ptr());
        assert_eq!(
            CStr::from_ptr(((*message_vtable).get_message_id)(message)),
            message_id
        );

        let attributes = ((*message_vtable).get_attributes)(message);
        assert!(!attributes.is_null());
        let attributes_vtable = vtable_of::<HostAttributeListVTable>(attributes);
        let key = c"parameter";
        assert_eq!(
            ((*attributes_vtable).set_int)(attributes, key.as_ptr(), 42),
            K_RESULT_OK
        );
        let mut value = 0;
        assert_eq!(
            ((*attributes_vtable).get_int)(attributes, key.as_ptr(), &mut value),
            K_RESULT_OK
        );
        assert_eq!(value, 42);

        assert_eq!(((*message_vtable).release)(message), 0);
    }
}

#[test]
fn creates_standalone_attribute_lists() {
    unsafe {
        let mut cid = IATTRIBUTE_LIST_IID;
        let mut iid = IATTRIBUTE_LIST_IID;
        let mut attributes = ptr::null_mut();
        assert_eq!(
            host_create_instance(
                host_context(),
                cid.as_mut_ptr(),
                iid.as_mut_ptr(),
                &mut attributes,
            ),
            K_RESULT_OK
        );
        assert!(!attributes.is_null());
        assert_eq!(host_attribute_list_release(attributes), 0);
    }
}
