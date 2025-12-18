use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DefaultPaths {
    pub root: PathBuf,
    pub state_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl DefaultPaths {
    pub fn from_root(root: &Path) -> Self {
        let state_dir = root.join(".greentic");
        let cache_dir = state_dir.join("cache");
        let logs_dir = state_dir.join("logs");
        Self {
            root: root.to_path_buf(),
            state_dir,
            cache_dir,
            logs_dir,
        }
    }
}

pub fn discover_project_root(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        let greentic_dir = ancestor.join(".greentic");
        if greentic_dir.is_dir() {
            return Some(ancestor.to_path_buf());
        }
        let git_dir = ancestor.join(".git");
        if git_dir.is_dir() {
            return Some(ancestor.to_path_buf());
        }
        let cargo = ancestor.join("Cargo.toml");
        if cargo.is_file() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

pub fn absolute_path(path: &Path) -> anyhow::Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(path))
}
