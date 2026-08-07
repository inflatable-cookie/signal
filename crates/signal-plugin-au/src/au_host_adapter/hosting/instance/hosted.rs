use std::path::Path;

use signal_plugin::PluginParameterDescriptor;

#[cfg(target_os = "macos")]
use super::super::ffi;
use super::super::process::AuProcessSession;
use super::super::types::{parse_load_key, AuHostingError};
use super::format::{main_element_layout, stereo_stream_format, verify_stereo_format};
use super::layout::{AuHostedPortLayout, HostedInstanceState};
use super::params::parameter_inventory;
use crate::au_host_adapter::gui::{AuCocoaViewInfo, AuGuiSession};

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
    /// The unit's custom Cocoa editor description, probed at load via
    /// `kAudioUnitProperty_CocoaUI` (g12.024). `None` = no editor UI (the
    /// generic view is deliberately not built — the parameter editor
    /// covers those units).
    cocoa_view: Option<AuCocoaViewInfo>,
    /// The live editor view, when open. Torn down (removeFromSuperview +
    /// release) BEFORE the unit is disposed in `Drop`.
    gui_session: Option<AuGuiSession>,
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
    /// hosting FFIs but is never opened: the component is resolved from the
    /// load key alone, whether discovery retained its bundle path or used the
    /// [`AU_REGISTRY_COMPONENT_PATH`] fallback. Off macOS this fails with the
    /// stable `unsupported_platform` token.
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
            let cocoa_view = crate::au_host_adapter::gui::cocoa_view_info(unit);
            Ok(Self {
                unit,
                parameters,
                port_layout,
                state: HostedInstanceState::Created,
                activated_max_frames: 0,
                cocoa_view,
                gui_session: None,
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

    /// Capture the Audio Unit's opaque class-info dictionary as a binary
    /// property list. The bytes are host-owned and may only be restored to
    /// the same Audio Unit identity.
    pub fn save_state(&self) -> Result<Vec<u8>, AuHostingError> {
        #[cfg(target_os = "macos")]
        unsafe {
            let mut class_info: ffi::CFPropertyListRef = std::ptr::null();
            let mut size = std::mem::size_of::<ffi::CFPropertyListRef>() as u32;
            if ffi::AudioUnitGetProperty(
                self.unit,
                ffi::kAudioUnitProperty_ClassInfo,
                ffi::kAudioUnitScope_Global,
                0,
                &mut class_info as *mut _ as *mut _,
                &mut size,
            ) != 0
                || class_info.is_null()
            {
                return Err(AuHostingError::new("state_capture_failed"));
            }
            let data = ffi::CFPropertyListCreateData(
                ffi::kCFAllocatorDefault,
                class_info,
                ffi::kCFPropertyListBinaryFormat_v1_0,
                0,
                std::ptr::null_mut(),
            );
            ffi::CFRelease(class_info);
            if data.is_null() {
                return Err(AuHostingError::new("state_serialize_failed"));
            }
            let length = ffi::CFDataGetLength(data).max(0) as usize;
            let pointer = ffi::CFDataGetBytePtr(data);
            let bytes = if length == 0 {
                Vec::new()
            } else if pointer.is_null() {
                ffi::CFRelease(data);
                return Err(AuHostingError::new("state_serialize_failed"));
            } else {
                std::slice::from_raw_parts(pointer, length).to_vec()
            };
            ffi::CFRelease(data);
            Ok(bytes)
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(AuHostingError::new("unsupported_platform"))
        }
    }

    /// Restore a binary class-info property list previously captured from
    /// this Audio Unit identity.
    pub fn load_state(&mut self, bytes: &[u8]) -> Result<(), AuHostingError> {
        #[cfg(target_os = "macos")]
        unsafe {
            let data = ffi::CFDataCreate(
                ffi::kCFAllocatorDefault,
                bytes.as_ptr(),
                bytes.len() as ffi::CFIndex,
            );
            if data.is_null() {
                return Err(AuHostingError::new("state_deserialize_failed"));
            }
            let property_list = ffi::CFPropertyListCreateWithData(
                ffi::kCFAllocatorDefault,
                data,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            ffi::CFRelease(data);
            if property_list.is_null() {
                return Err(AuHostingError::new("state_deserialize_failed"));
            }
            let status = ffi::AudioUnitSetProperty(
                self.unit,
                ffi::kAudioUnitProperty_ClassInfo,
                ffi::kAudioUnitScope_Global,
                0,
                &property_list as *const _ as *const _,
                std::mem::size_of::<ffi::CFPropertyListRef>() as u32,
            );
            ffi::CFRelease(property_list);
            if status != 0 {
                return Err(AuHostingError::new("state_restore_failed"));
            }
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = bytes;
            Err(AuHostingError::new("unsupported_platform"))
        }
    }

    /// Queue-free parameter write (g12.023): AU's set domain is the PLAIN
    /// value and `AudioUnitSetParameter` is safe from a non-render thread
    /// (AudioToolbox serializes against the render), so the host's
    /// normalized 0..1 value maps linearly onto the descriptor range and
    /// applies immediately — the unit picks it up on its next render pull
    /// (block-boundary in practice).
    pub fn set_parameter_normalized(
        &mut self,
        parameter_id: u32,
        normalized: f32,
    ) -> Result<(), AuHostingError> {
        let descriptor = self
            .parameters
            .iter()
            .find(|parameter| parameter.parameter_id == parameter_id)
            .ok_or_else(|| AuHostingError::new("unknown_parameter"))?;
        let normalized = normalized.clamp(0.0, 1.0);
        let plain =
            descriptor.min_plain + normalized * (descriptor.max_plain - descriptor.min_plain);
        self.set_parameter(parameter_id, plain)
    }

    /// Set one parameter's plain value on the global scope
    /// (`AudioUnitSetParameter`).
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
                if self.port_layout.main_input_channels > 0 {
                    let _ = ffi::AudioUnitSetProperty(
                        self.unit,
                        ffi::kAudioUnitProperty_StreamFormat,
                        ffi::kAudioUnitScope_Input,
                        0,
                        &format as *const _ as *const _,
                        std::mem::size_of::<ffi::AudioStreamBasicDescription>() as u32,
                    );
                }
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
                let scopes = if self.port_layout.main_input_channels > 0 {
                    &[ffi::kAudioUnitScope_Input, ffi::kAudioUnitScope_Output][..]
                } else {
                    &[ffi::kAudioUnitScope_Output][..]
                };
                for &scope in scopes {
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

    // ── Cocoa editor view hosting (g12.024, GUI phase 2) ───────────────
    //
    // MAIN-THREAD CONTRACT: `gui_open_embedded` and `gui_destroy` touch
    // AppKit and must run on the application main thread (Tauri
    // `run_on_main_thread`); this type only serializes access.

    /// Whether the unit provides a custom Cocoa editor
    /// (`kAudioUnitProperty_CocoaUI` probed and cached at load).
    pub fn gui_supported(&self) -> bool {
        self.cocoa_view.is_some()
    }

    /// The probed Cocoa editor description (bundle path + factory class),
    /// when the unit provides one.
    pub fn cocoa_view_info(&self) -> Option<&AuCocoaViewInfo> {
        self.cocoa_view.as_ref()
    }

    /// Whether an editor view is currently attached.
    pub fn gui_is_open(&self) -> bool {
        self.gui_session.is_some()
    }

    /// Open the unit's Cocoa editor child-attached into `parent` (a live
    /// `NSView*`): load the view bundle → instantiate the factory class →
    /// `uiViewForAudioUnit:withSize:` → `addSubview`. Returns the view's
    /// reported frame size (logical units). MAIN THREAD ONLY. Errors with
    /// stable tokens (`gui_unsupported`, `gui_already_open`,
    /// `gui_view_create_failed`, …).
    ///
    /// # Safety
    ///
    /// `parent` must be a live, valid `NSView*` owned by the caller, and must
    /// outlive the returned editor session. It is handed straight to the
    /// plugin, which attaches its own view to it. Must be called on the
    /// application main thread.
    pub unsafe fn gui_open_embedded(
        &mut self,
        parent: *mut std::ffi::c_void,
        _scale: Option<f64>,
    ) -> Result<(u32, u32), AuHostingError> {
        if self.gui_session.is_some() {
            return Err(AuHostingError::new("gui_already_open"));
        }
        #[cfg(target_os = "macos")]
        {
            let info = self
                .cocoa_view
                .as_ref()
                .ok_or_else(|| AuHostingError::new("gui_unsupported"))?;
            let session =
                unsafe { crate::au_host_adapter::gui::open_embedded(self.unit, info, parent) }?;
            let size = session.size();
            self.gui_session = Some(session);
            Ok(size)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = parent;
            Err(AuHostingError::new("unsupported_platform"))
        }
    }

    /// The open editor view, read-only.
    pub fn gui_session(&self) -> Option<&AuGuiSession> {
        self.gui_session.as_ref()
    }

    /// Destroy the open editor view (idempotent; removeFromSuperview +
    /// release — the unit stays live). MAIN THREAD ONLY.
    pub fn gui_destroy(&mut self) {
        self.gui_session = None;
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
            AuProcessSession::new(
                self.unit,
                self.activated_max_frames as usize,
                self.port_layout.main_input_channels > 0,
            )
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(AuHostingError::new("unsupported_platform"))
        }
    }
}

impl Drop for AuHostedInstance {
    fn drop(&mut self) {
        // View teardown must precede unit disposal. This is the fallback
        // path (teardown with an editor still open); the orderly path
        // closes the editor on the main thread first.
        self.gui_session = None;
        #[cfg(target_os = "macos")]
        unsafe {
            if self.state == HostedInstanceState::Active {
                let _ = ffi::AudioUnitUninitialize(self.unit);
            }
            let _ = ffi::AudioComponentInstanceDispose(self.unit);
        }
    }
}
