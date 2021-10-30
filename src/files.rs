use crate::SemVer;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;

/// Finds files in a folder.
pub trait FileFinder {
    /// Finds files in a folder.
    fn find(&self, dir: &str) -> std::io::Result<Vec<PathBuf>>;
}

/// Checks if the given path has the specified file extension,
/// without case sensitivity.
pub fn has_extension(path_buf: &PathBuf, extension: &str) -> bool {
    match path_buf.extension() {
        Some(os_str) => os_str.to_string_lossy().eq_ignore_ascii_case(extension),
        _ => false,
    }
}

/// Modifies the given text contents so that they indicate
/// the given semantic version.
pub trait ContentProcessor {
    type Err;

    /// Modifies the given text contents so that they indicate
    /// the given semantic version.
    fn process(&self, old_contents: &str, new_version: SemVer) -> Result<String, Self::Err>;
}

#[derive(Debug)]
pub enum UpdateError {
    IOError(std::io::Error),
    Other(Box<dyn std::error::Error>),
}

impl UpdateError {
    pub fn new_boxing_other<T: 'static + std::error::Error>(other: T) -> Self {
        Self::Other(Box::new(other))
    }
}
impl From<std::io::Error> for UpdateError {
    fn from(io_error: std::io::Error) -> Self {
        Self::IOError(io_error)
    }
}

impl Display for UpdateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IOError(e) => std::fmt::Display::fmt(&e, f),
            Self::Other(e) => std::fmt::Display::fmt(&e, f),
        }
    }
}

impl std::error::Error for UpdateError {}

pub trait DirUpdater {
    fn update(&self, dir: &str, new_version: SemVer)
        -> Result<Vec<(PathBuf, String)>, UpdateError>;
}

impl<T> DirUpdater for T
where
    T: FileFinder + ContentProcessor,
    <T as ContentProcessor>::Err: 'static + std::error::Error,
{
    fn update(
        &self,
        dir: &str,
        new_version: SemVer,
    ) -> Result<Vec<(PathBuf, String)>, UpdateError> {
        let files = self.find(dir)?;
        let mut result = Vec::<(PathBuf, String)>::new();
        for file in files {
            let old_contents = fs::read_to_string(&file)?;
            let changed_contents = self
                .process(&old_contents, new_version)
                .map_err(UpdateError::new_boxing_other)?;
            if old_contents != changed_contents {
                result.push((file, changed_contents));
            }
        }
        Ok(result)
    }
}

/// Finds files in a folder that match a given file extension.
/// Does not search sub-folders, only root folder.
pub struct RootFileFinderByExt {
    extension: String,
}

impl RootFileFinderByExt {
    pub fn new(extension: &str) -> Self {
        Self {
            extension: extension.to_owned(),
        }
    }
}

impl FileFinder for RootFileFinderByExt {
    fn find(&self, dir: &str) -> std::io::Result<Vec<PathBuf>> {
        let mut result = Vec::<PathBuf>::new();
        for res_entry in fs::read_dir(dir)? {
            let entry = res_entry?;
            let path = entry.path();
            if path.is_file() && has_extension(&path, &self.extension) {
                result.push(path);
            }
        }
        Ok(result)
    }
}
