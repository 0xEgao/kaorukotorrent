use std::{
    error::Error,
    fmt, fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ItemMetadata {
    pub item: String,
    pub info_hash: String,
    pub total_size: u64,
    pub files: Vec<FileEntry>,
}

pub fn build_metadata(item: &str, root: &Path) -> Result<ItemMetadata, MetadataError> {
    let mut files = Vec::new();
    if !root.exists() {
        return Err(MetadataError::new("item path does not exist"));
    }

    collect_files(root, root, &mut files)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));

    let mut hasher = Sha256::new();
    let mut total_size = 0u64;

    for entry in &files {
        let file_path = root.join(&entry.path);
        total_size = total_size.saturating_add(entry.size);
        hasher.update(entry.path.as_bytes());
        hasher.update(entry.size.to_le_bytes());
        hash_file_contents(&file_path, &mut hasher)?;
    }

    let info_hash = format!("{:x}", hasher.finalize());

    Ok(ItemMetadata {
        item: item.to_string(),
        info_hash,
        total_size,
        files,
    })
}

pub fn resolve_item_path(base_dir: &Path, item_path: &str) -> Result<PathBuf, MetadataError> {
    let clean = sanitize_relative_path(item_path)?;
    Ok(base_dir.join(clean))
}

pub fn sanitize_relative_path(input: &str) -> Result<PathBuf, MetadataError> {
    let path = Path::new(input);
    if path.is_absolute() {
        return Err(MetadataError::new("absolute paths are not allowed"));
    }

    for component in path.components() {
        match component {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(MetadataError::new("path traversal is not allowed"));
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }

    Ok(path.to_path_buf())
}

fn collect_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<FileEntry>,
) -> Result<(), MetadataError> {
    let metadata = fs::metadata(current)?;
    if metadata.is_file() {
        let relative = current
            .strip_prefix(root)
            .map_err(|_| MetadataError::new("failed to compute relative path"))?;
        files.push(FileEntry {
            path: to_slash_path(relative),
            size: metadata.len(),
        });
        return Ok(());
    }

    if metadata.is_dir() {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            collect_files(root, &entry.path(), files)?;
        }
        return Ok(());
    }

    Err(MetadataError::new("unsupported file type"))
}

fn to_slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn hash_file_contents(path: &Path, hasher: &mut Sha256) -> Result<(), MetadataError> {
    let mut file = fs::File::open(path)?;
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(())
}

#[derive(Debug)]
pub struct MetadataError {
    message: &'static str,
    source: Option<io::Error>,
}

impl MetadataError {
    fn new(message: &'static str) -> Self {
        Self {
            message,
            source: None,
        }
    }

    fn with_source(message: &'static str, source: io::Error) -> Self {
        Self {
            message,
            source: Some(source),
        }
    }
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for MetadataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|err| err as &dyn Error)
    }
}

impl From<io::Error> for MetadataError {
    fn from(err: io::Error) -> Self {
        MetadataError::with_source("io error", err)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{build_metadata, sanitize_relative_path};

    fn make_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("sender_test_{}", nanos));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn sanitize_rejects_parent() {
        assert!(sanitize_relative_path("../secret").is_err());
        assert!(sanitize_relative_path("/abs/path").is_err());
    }

    #[test]
    fn build_metadata_collects_files() {
        let root = make_temp_dir();
        let file_a = root.join("a.txt");
        let file_b = root.join("nested").join("b.txt");
        fs::create_dir_all(file_b.parent().expect("parent should exist"))
            .expect("create nested dir");
        fs::write(&file_a, b"hello").expect("write file a");
        fs::write(&file_b, b"world").expect("write file b");

        let metadata = build_metadata("sample", &root).expect("metadata should build");

        assert_eq!(metadata.total_size, 10);
        assert_eq!(metadata.files.len(), 2);
        assert_eq!(metadata.item, "sample");
    }
}
