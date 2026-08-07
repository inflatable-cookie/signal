//! CLAP library entry loading shared with discovery.

use std::{ffi::CString, path::Path};

use clap_sys::{
    entry::clap_plugin_entry,
    factory::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID},
};
use libloading::Library;

use crate::discovery::clap_bundle_binary;

/// Error surface for hosting operations; carries a stable snake_case token
/// suitable for broker receipt details.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapHostingError {
    /// Stable snake_case failure token (e.g. `library_open_failed`).
    pub token: String,
}

impl ClapHostingError {
    pub(crate) fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl std::fmt::Display for ClapHostingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.token)
    }
}

impl std::error::Error for ClapHostingError {}

/// A dlopen'd CLAP library with its entry initialized. Shared by discovery
/// and hosting so both speak the same loading path. Deinitializes the entry
/// and closes the library on drop.
pub struct LoadedClapEntry {
    /// Keeps the dynamic library mapped for the entry's lifetime.
    _library: Library,
    entry: *const clap_plugin_entry,
}

impl LoadedClapEntry {
    /// dlopen `library_path`, resolve `clap_entry`, and run its `init`.
    pub fn load(library_path: &Path) -> Result<Self, ClapHostingError> {
        let load_path = clap_library_binary_path(library_path)?;
        let library = unsafe { Library::new(&load_path) }
            .map_err(|_| ClapHostingError::new("library_open_failed"))?;
        let entry = unsafe {
            library
                .get::<*const clap_plugin_entry>(b"clap_entry\0")
                .map_err(|_| ClapHostingError::new("clap_entry_missing"))
                .map(|symbol| *symbol)?
        };
        if entry.is_null() {
            return Err(ClapHostingError::new("clap_entry_null"));
        }
        let plugin_path =
            CString::new(clap_plugin_path(library_path).to_string_lossy().to_string())
                .map_err(|_| ClapHostingError::new("library_path_invalid"))?;
        if let Some(init) = unsafe { (*entry).init } {
            if !unsafe { init(plugin_path.as_ptr()) } {
                return Err(ClapHostingError::new("entry_init_failed"));
            }
        }
        Ok(Self {
            _library: library,
            entry,
        })
    }

    /// The library's plugin factory, when it exposes one.
    pub fn plugin_factory(&self) -> Option<*const clap_plugin_factory> {
        let get_factory = unsafe { (*self.entry).get_factory }?;
        let factory = unsafe { get_factory(CLAP_PLUGIN_FACTORY_ID.as_ptr()) };
        (!factory.is_null()).then(|| factory.cast::<clap_plugin_factory>())
    }

    /// The raw initialized entry.
    pub(crate) fn entry(&self) -> clap_plugin_entry {
        unsafe { *self.entry }
    }
}

fn clap_library_binary_path(library_path: &Path) -> Result<std::path::PathBuf, ClapHostingError> {
    if library_path.is_dir()
        && library_path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("clap"))
    {
        return clap_bundle_binary(library_path)
            .ok_or_else(|| ClapHostingError::new("bundle_binary_missing"));
    }
    Ok(library_path.to_path_buf())
}

fn clap_plugin_path(library_path: &Path) -> &Path {
    library_path
        .ancestors()
        .find(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("clap"))
        })
        .unwrap_or(library_path)
}

#[cfg(test)]
mod plugin_path_tests {
    use super::{clap_library_binary_path, clap_plugin_path};
    use std::path::Path;

    #[test]
    fn macos_bundle_binary_initializes_with_outer_clap_path() {
        let binary = Path::new("/Plug-Ins/Example.clap/Contents/MacOS/Example");
        assert_eq!(
            clap_plugin_path(binary),
            Path::new("/Plug-Ins/Example.clap")
        );
    }

    #[test]
    fn standalone_library_initializes_with_its_own_path() {
        let library = Path::new("/usr/lib/clap/example.clap.so");
        assert_eq!(clap_plugin_path(library), library);
    }

    #[test]
    fn macos_bundle_loads_its_contents_binary() {
        let root = std::env::temp_dir().join(format!(
            "signal-clap-bundle-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        ));
        let bundle = root.join("Example.clap");
        let binary = bundle.join("Contents/MacOS/Example");
        std::fs::create_dir_all(binary.parent().expect("binary parent")).expect("bundle dirs");
        std::fs::write(&binary, b"fixture").expect("bundle binary");

        assert_eq!(
            clap_library_binary_path(&bundle).expect("resolve bundle binary"),
            binary,
        );

        let _ = std::fs::remove_dir_all(root);
    }
}

impl Drop for LoadedClapEntry {
    fn drop(&mut self) {
        if let Some(deinit) = unsafe { (*self.entry).deinit } {
            unsafe { deinit() };
        }
    }
}
