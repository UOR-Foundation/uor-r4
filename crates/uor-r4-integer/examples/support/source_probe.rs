//! Resolve an output through existing ancestors without creating directories.
use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

pub fn prospective_output(path: &Path) -> io::Result<PathBuf> {
    if path
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "output cannot contain parent traversal",
        ));
    }
    let mut ancestor = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut suffix = Vec::new();
    loop {
        match fs::symlink_metadata(&ancestor) {
            Ok(_) => break,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                suffix.push(
                    ancestor
                        .file_name()
                        .ok_or_else(|| {
                            io::Error::new(io::ErrorKind::InvalidInput, "output ancestor absent")
                        })?
                        .to_os_string(),
                );
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "output parent absent")
                    })?
                    .to_path_buf();
            }
            Err(error) => return Err(error),
        }
    }
    let mut output = fs::canonicalize(ancestor)?;
    for part in suffix.into_iter().rev() {
        output.push(part);
    }
    Ok(output)
}
