use anyhow::{Context, Result, ensure};
use std::{fs, io::Write, path::Path};

/// Check that an editable file still matches the editor's opening snapshot.
pub(super) fn verify(path: &Path, original: Option<&[u8]>) -> Result<()> {
    let current = match fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    ensure!(
        current.as_deref() == original,
        "The file changed on disk. Close the editor and reopen the workspace before saving: {}",
        path.display()
    );
    ensure!(
        !fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()),
        "Cannot replace a symbolic link: {}",
        path.display()
    );
    Ok(())
}

pub(super) fn save(path: &Path, original: Option<&[u8]>, bytes: &[u8]) -> Result<()> {
    verify(path, original)?;
    let parent = path.parent().context("File has no parent folder")?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(bytes)?;
    file.as_file().sync_all()?;
    if original.is_some() {
        file.persist(path)?;
    } else {
        file.persist_noclobber(path)?;
    }
    Ok(())
}
