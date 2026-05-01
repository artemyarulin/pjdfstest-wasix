use std::{fs, io, path::Path};

use crate::RunError;

pub(crate) fn read_dir_sorted(path: &Path) -> Result<Vec<fs::DirEntry>, RunError> {
    let mut entries = fs::read_dir(path)
        .map_err(|err| read_dir_error(path, err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| read_dir_error(path, err))?;
    entries.sort_by_key(|entry| entry.path());
    Ok(entries)
}

pub(crate) fn read_to_string(path: &Path) -> Result<String, RunError> {
    fs::read_to_string(path).map_err(|err| read_file_error(path, err))
}

fn read_dir_error(path: &Path, err: io::Error) -> RunError {
    RunError::ReadDir {
        path: path.to_path_buf(),
        message: err.to_string(),
    }
}

fn read_file_error(path: &Path, err: io::Error) -> RunError {
    RunError::ReadFile {
        path: path.to_path_buf(),
        message: err.to_string(),
    }
}
