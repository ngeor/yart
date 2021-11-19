//! Updates files

use crate::files::{DirUpdater, UpdateError};
use crate::writers::create_writer;
use crate::{delphi, rust, vb6, SemVer};
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

macro_rules! add_files {
    ($updater:expr, $dir: expr, $new_version: expr, $result: expr) => {
        let mut partial_files = $updater.update($dir, $new_version)?;
        $result.append(&mut partial_files);
    };
}

impl DirUpdater for CompositeDirUpdater {
    fn update(
        &self,
        dir: &str,
        new_version: SemVer,
    ) -> Result<Vec<(PathBuf, String)>, UpdateError> {
        let mut result = Vec::<(PathBuf, String)>::new();
        add_files!(vb6::VB6Updater {}, dir, new_version, result);
        add_files!(delphi::LpiUpdater {}, dir, new_version, result);
        add_files!(rust::CargoDirUpdater::new(), dir, new_version, result);
        Ok(result)
    }
}
