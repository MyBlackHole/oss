pub mod fs;
pub use fs::FsStorage;

use std::path::PathBuf;

use crate::models::{BucketConfig, Metadata};

#[derive(Debug, Default)]
pub struct ListObjectsResult {
    pub contents: Vec<ListObjectEntry>,
    pub common_prefixes: Vec<String>,
    pub next_marker: String,
    pub is_truncated: bool,
}

#[derive(Debug)]
pub struct ListObjectEntry {
    pub key: String,
    pub etag: String,
    pub size: i64,
    pub last_modified: String,
    pub storage_class: String,
}

pub trait Storage: Send + Sync {
    fn store_root(&self) -> &PathBuf;

    fn bucket_metadata_path(&self, bucket: &str) -> PathBuf {
        self.store_root().join(bucket).join(crate::config::BUCKET_METADATA)
    }

    fn bucket_dir(&self, bucket: &str) -> PathBuf {
        self.store_root().join(bucket)
    }

    fn object_dir(&self, bucket: &str, object: &str) -> PathBuf {
        self.store_root().join(bucket).join(object)
    }

    fn object_metadata_path(&self, bucket: &str, object: &str) -> PathBuf {
        self.object_dir(bucket, object).join(crate::config::OBJECT_METADATA)
    }

    fn object_content_path(&self, bucket: &str, object: &str) -> PathBuf {
        self.object_dir(bucket, object).join(crate::config::OBJECT_CONTENT)
    }

    fn object_content_part_path(&self, bucket: &str, object: &str, part: i32) -> PathBuf {
        self.object_dir(bucket, object)
            .join(format!("{}{}", crate::config::OBJECT_CONTENT_PREFIX, part))
    }

    fn save_metadata(&self, path: &std::path::Path, metadata: &Metadata) -> std::io::Result<()>;
    fn load_metadata(&self, path: &std::path::Path) -> std::io::Result<Metadata>;
    fn list_buckets(&self) -> std::io::Result<Vec<String>>;
    fn bucket_exists(&self, bucket: &str) -> bool;
    fn bucket_count(&self) -> usize;
    fn bucket_is_empty(&self, bucket: &str) -> std::io::Result<bool>;
    fn delete_bucket(&self, bucket: &str) -> std::io::Result<()>;
    fn list_objects(&self, bucket: &str, prefix: &str, marker: &str, max_keys: i32, delimiter: &str)
        -> std::io::Result<ListObjectsResult>;

    /// List in-progress multipart uploads in a bucket.
    /// Returns Vec of (object_key, upload_id).
    fn list_multipart_uploads(&self, bucket: &str) -> std::io::Result<Vec<(String, String)>>;

    /// Bucket config 文件路径
    fn bucket_config_path(&self, bucket: &str) -> PathBuf {
        self.bucket_dir(bucket).join(crate::config::BUCKET_CONFIG)
    }

    /// 加载 Bucket 配置（LOGGING/WEBSITE/REFERER/LIFECYCLE）
    fn load_bucket_config(&self, bucket: &str) -> std::io::Result<Option<BucketConfig>> {
        let path = self.bucket_config_path(bucket);
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        match toml::from_str(&content) {
            Ok(config) => Ok(Some(config)),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
        }
    }

    /// 保存 Bucket 配置
    fn save_bucket_config(&self, bucket: &str, config: &BucketConfig) -> std::io::Result<()> {
        let path = self.bucket_config_path(bucket);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string(config).map_err(|e| {
            std::io::Error::other(e.to_string())
        })?;
        std::fs::write(&path, &content)
    }
}

#[derive(Debug)]
pub struct ListBucketInfo {
    pub name: String,
    pub creation_date: String,
}
