use std::{fs, path::Path};

use crate::RunError;

pub(crate) fn read_dir_sorted(path: &Path) -> Result<Vec<fs::DirEntry>, RunError> {
    let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    Ok(entries)
}

pub(crate) fn read_to_string(path: &Path) -> Result<String, RunError> {
    let data = fs::read_to_string(path)?;
    Ok(data)
}
