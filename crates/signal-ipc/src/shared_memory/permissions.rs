use std::{
    fs::{self, File},
    io,
    path::Path,
};

use memmap2::{MmapMut, MmapOptions};

pub(super) fn map_file(file: &File, len: usize) -> io::Result<MmapMut> {
    // SAFETY: the file is explicitly sized by the broker before mapping, and
    // callers pass the exact byte length they expect to use.
    unsafe { MmapOptions::new().len(len).map_mut(file) }
}

pub(super) fn sanitize_identifier(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch,
            _ => '-',
        })
        .collect()
}

#[cfg(unix)]
pub(super) fn tighten_directory_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
pub(super) fn tighten_directory_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
pub(super) fn tighten_file_permissions(file: &File) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    file.set_permissions(fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
pub(super) fn tighten_file_permissions(_file: &File) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
pub(super) fn tighten_path_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
pub(super) fn tighten_path_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}
