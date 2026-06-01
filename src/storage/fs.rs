use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::{BUCKET_METADATA, OBJECT_CONTENT_PREFIX, OBJECT_METADATA};
use crate::models::Metadata;
use crate::storage::{ListObjectEntry, ListObjectsResult, Storage};

pub struct FsStorage {
    store_root: PathBuf,
}

impl FsStorage {
    pub fn new(store_root: PathBuf) -> Self {
        std::fs::create_dir_all(&store_root).ok();
        Self { store_root }
    }
}

impl Storage for FsStorage {
    fn store_root(&self) -> &PathBuf {
        &self.store_root
    }

    fn save_metadata(&self, path: &Path, metadata: &Metadata) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string(metadata).map_err(|e| {
            std::io::Error::other(e.to_string())
        })?;
        std::fs::write(path, &toml_str)
    }

    fn load_metadata(&self, path: &Path) -> std::io::Result<Metadata> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })
    }

    fn list_buckets(&self) -> std::io::Result<Vec<String>> {
        let mut buckets = Vec::new();
        if !self.store_root.exists() {
            return Ok(buckets);
        }
        for entry in std::fs::read_dir(&self.store_root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let bucket_name = entry.file_name().to_string_lossy().to_string();
                let meta_path = entry.path().join(BUCKET_METADATA);
                if meta_path.exists() {
                    buckets.push(bucket_name);
                }
            }
        }
        Ok(buckets)
    }

    fn bucket_exists(&self, bucket: &str) -> bool {
        self.bucket_metadata_path(bucket).exists()
    }

    fn bucket_count(&self) -> usize {
        self.list_buckets().map(|b| b.len()).unwrap_or(0)
    }

    fn bucket_is_empty(&self, bucket: &str) -> std::io::Result<bool> {
        let dir = self.bucket_dir(bucket);
        if !dir.exists() {
            return Ok(true);
        }
        let mut has_entries = false;
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name != BUCKET_METADATA {
                has_entries = true;
                break;
            }
        }
        Ok(!has_entries)
    }

    fn delete_bucket(&self, bucket: &str) -> std::io::Result<()> {
        let dir = self.bucket_dir(bucket);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }

    fn list_objects(
        &self,
        bucket: &str,
        prefix: &str,
        marker: &str,
        max_keys: i32,
        delimiter: &str,
    ) -> std::io::Result<ListObjectsResult> {
        let mut result = ListObjectsResult::default();
        let mut marker_found = marker.is_empty();
        let mut count = 0;
        let mut common_prefixes_map: HashSet<String> = HashSet::new();

        let bucket_path = self.bucket_dir(bucket);
        if !bucket_path.exists() {
            return Ok(result);
        }

        let max = if max_keys <= 0 { 100 } else { max_keys.min(1000) };

        if !bucket_path.exists() {
            return Ok(result);
        }

        let walker = walkdir::WalkDir::new(&bucket_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == OBJECT_METADATA);

        for entry in walker {
            let path = entry.path();
            if let Some(parent) = path.parent() {
                let key = parent
                    .strip_prefix(&bucket_path)
                    .unwrap_or(parent)
                    .to_string_lossy()
                    .to_string()
                    .replace('\\', "/");

                if !marker_found {
                    if key.as_str() >= marker {
                        marker_found = true;
                    } else {
                        continue;
                    }
                }

                if !prefix.is_empty() && !key.starts_with(prefix) {
                    continue;
                }

                if delimiter == "/" {
                    let remaining = key.strip_prefix(prefix).unwrap_or(&key);
                    if let Some(slash_pos) = remaining.find('/') {
                        let common = format!("{}{}/", prefix, &remaining[..=slash_pos]);
                        if common_prefixes_map.insert(common.clone()) {
                            count += 1;
                            if count <= max as usize {
                                result.common_prefixes.push(common);
                            } else {
                                result.is_truncated = true;
                                break;
                            }
                            continue;
                        } else {
                            continue;
                        }
                    }
                }

                count += 1;
                if count <= max as usize {
                    if let Ok(metadata) = self.load_metadata(path) {
                        result.contents.push(ListObjectEntry {
                            key: key.clone(),
                            etag: metadata.md5,
                            size: metadata.size,
                            last_modified: "2012-02-24T08:42:32.000Z".to_string(),
                            storage_class: "Standard".to_string(),
                        });
                    }
                } else {
                    result.is_truncated = true;
                    break;
                }

                result.next_marker = key;
            }
        }

        Ok(result)
    }

    fn list_multipart_uploads(&self, bucket: &str) -> std::io::Result<Vec<(String, String)>> {
        let mut uploads = Vec::new();
        let bucket_path = self.bucket_dir(bucket);
        if !bucket_path.exists() {
            return Ok(uploads);
        }

        for entry in std::fs::read_dir(&bucket_path)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let obj_key = entry.file_name().to_string_lossy().to_string();
            let obj_dir = entry.path();

            let meta_path = obj_dir.join(OBJECT_METADATA);
            if meta_path.exists() {
                continue;
            }

            let has_parts = std::fs::read_dir(&obj_dir)
                .map(|mut dir| dir.any(|e| {
                    e.ok().and_then(|e| {
                        e.file_name().to_str().map(|n| n.starts_with(OBJECT_CONTENT_PREFIX))
                    }).unwrap_or(false)
                }))
                .unwrap_or(false);

            if has_parts {
                uploads.push((obj_key, "0000000".to_string()));
            }
        }

        Ok(uploads)
    }
}
