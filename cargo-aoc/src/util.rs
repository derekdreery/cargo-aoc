use std::{fs, io, path::Path};

/// Convert `NotFound` to `None`.
pub trait IoResultExt<T> {
    /// Convert `NotFound` to `None`.
    fn optional(self) -> io::Result<Option<T>>;
}

impl<T> IoResultExt<T> for io::Result<T> {
    fn optional(self) -> io::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(e) if matches!(e.kind(), io::ErrorKind::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

pub fn path_exists(path: impl AsRef<Path>) -> io::Result<bool> {
    Ok(fs::metadata(path).optional()?.is_some())
}
