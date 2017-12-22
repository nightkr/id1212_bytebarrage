use std::path::PathBuf;
use std::fs::{metadata, read_dir};
use std::io::Error as IOError;

pub fn walk_dir_files<Error, F>(path: PathBuf, f: &mut F) -> Result<(), Error>
where
    Error: From<IOError>,
    F: FnMut(PathBuf) -> Result<(), Error>,
{
    if metadata(&path)?.is_dir() {
        for entry in read_dir(path)? {
            walk_dir_files(entry?.path(), f)?;
        }
    } else {
        f(path)?;
    }
    Ok(())
}
