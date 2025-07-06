use crate::Result;
use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileManager {
    recent_files: Vec<PathBuf>,
    max_recent_files: usize,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            recent_files: Vec::new(),
            max_recent_files: 10,
        }
    }

    pub fn read_file<P: AsRef<Path>>(&mut self, path: P) -> Result<String> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        self.add_recent_file(path.to_path_buf());
        Ok(content)
    }

    pub fn write_file<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<()> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        fs::write(path, content)
            .with_context(|| format!("Failed to write file: {}", path.display()))?;

        Ok(())
    }

    pub fn file_exists<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().exists()
    }

    pub fn is_readable<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_file()
            && fs::metadata(path.as_ref())
                .map(|m| !m.permissions().readonly())
                .unwrap_or(false)
    }

    pub fn get_file_size<P: AsRef<Path>>(&self, path: P) -> Result<u64> {
        let metadata = fs::metadata(path.as_ref())
            .with_context(|| format!("Failed to get file metadata: {}", path.as_ref().display()))?;
        Ok(metadata.len())
    }

    pub fn backup_file<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let path = path.as_ref();
        let backup_path = path.with_extension(format!(
            "{}.backup",
            path.extension().and_then(|s| s.to_str()).unwrap_or("")
        ));

        fs::copy(path, &backup_path)
            .with_context(|| format!("Failed to create backup: {}", backup_path.display()))?;

        Ok(backup_path)
    }

    pub fn add_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);

        self.recent_files.insert(0, path);

        if self.recent_files.len() > self.max_recent_files {
            self.recent_files.truncate(self.max_recent_files);
        }
    }

    pub fn get_recent_files(&self) -> &[PathBuf] {
        &self.recent_files
    }

    pub fn clear_recent_files(&mut self) {
        self.recent_files.clear();
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}
