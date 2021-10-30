//! Updates files

use crate::files::{DirUpdater, UpdateError};
use crate::writers::create_writer;
use crate::{delphi, vb6, SemVer};
use std::path::PathBuf;

pub fn update_files(
    dir: &str,
    new_version: SemVer,
    dry_run: bool,
) -> Result<Vec<(PathBuf, String)>, UpdateError> {
    let composite = CompositeDirUpdater {};
    let files = composite.update(dir, new_version)?;
    let writer = create_writer(PathBuf::from(dir), dry_run);
    for (path_buf, new_contents) in files.iter() {
        writer.write(path_buf, new_contents)?;
    }
    Ok(files)
}

struct CompositeDirUpdater {}

impl DirUpdater for CompositeDirUpdater {
    fn update(
        &self,
        dir: &str,
        new_version: SemVer,
    ) -> Result<Vec<(PathBuf, String)>, UpdateError> {
        let mut result = Vec::<(PathBuf, String)>::new();
        let mut vbp_files = vb6::VB6Updater {}.update(dir, new_version)?;
        result.append(&mut vbp_files);
        let mut delphi_files = delphi::LpiUpdater {}.update(dir, new_version)?;
        result.append(&mut delphi_files);
        Ok(result)
    }
}
