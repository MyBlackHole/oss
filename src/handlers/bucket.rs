use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Response;

use crate::config::MAX_BUCKET_NUM;
use crate::error::NO_SUCH_BUCKET;
use crate::models::Metadata;
use crate::response::OssResponse;
use crate::storage::{ListBucketInfo, Storage};

pub async fn list_buckets(State(storage): State<Arc<dyn Storage>>) -> Response {
    let buckets = match storage.list_buckets() {
        Ok(b) => b,
        Err(_) => return OssResponse::error(NO_SUCH_BUCKET),
    };

    let mut infos = Vec::new();
    for name in &buckets {
        let meta_path = storage.bucket_metadata_path(name);
        if let Ok(meta) = storage.load_metadata(&meta_path) {
            infos.push(ListBucketInfo {
                name: name.clone(),
                creation_date: meta.creation_date,
            });
        }
    }

    OssResponse::list_buckets(&infos)
}

pub async fn create_bucket(
    State(storage): State<Arc<dyn Storage>>,
    axum::extract::Path(bucket): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Response {
    if storage.bucket_count() >= MAX_BUCKET_NUM {
        return crate::response::OssResponse::error(crate::error::TOO_MANY_BUCKETS);
    }

    let bucket_exists = storage.bucket_exists(&bucket);
    if !bucket_exists {
        let acl = headers
            .get("x-oss-acl")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("private");

        let meta = Metadata {
            bucket: bucket.clone(),
            object: String::new(),
            acl: acl.to_string(),
            creation_date: chrono::Utc::now().to_rfc3339(),
            modified_date: chrono::Utc::now().to_rfc3339(),
            ..Metadata::new(&bucket, "")
        };

        let meta_path = storage.bucket_metadata_path(&bucket);
        let _ = storage.save_metadata(&meta_path, &meta);
    }

    OssResponse::put_bucket()
}

pub async fn get_bucket(
    State(storage): State<Arc<dyn Storage>>,
    axum::extract::Path(bucket): axum::extract::Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let prefix = params.get("prefix").map(|s| s.as_str()).unwrap_or("");
    let marker = params.get("marker").map(|s| s.as_str()).unwrap_or("");
    let max_keys_str = params.get("max-keys").map(|s| s.as_str()).unwrap_or("100");
    let delimiter = params.get("delimiter").map(|s| s.as_str()).unwrap_or("");
    let max_keys: i32 = max_keys_str.parse().unwrap_or(100);

    match storage.list_objects(&bucket, prefix, marker, max_keys, delimiter) {
        Ok(result) => {
            OssResponse::list_objects(
                &bucket,
                &result.contents,
                &result.common_prefixes,
                prefix,
                marker,
                max_keys,
                delimiter,
                &result.next_marker,
                result.is_truncated,
            )
        }
        Err(_) => OssResponse::error(crate::error::INTERNAL_ERROR),
    }
}

pub async fn delete_bucket(
    State(storage): State<Arc<dyn Storage>>,
    axum::extract::Path(bucket): axum::extract::Path<String>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    match storage.bucket_is_empty(&bucket) {
        Ok(true) => {
            let _ = storage.delete_bucket(&bucket);
            OssResponse::ok_no_content()
        }
        Ok(false) => {
            OssResponse::error(crate::error::BUCKET_NOT_EMPTY)
        }
        Err(_) => OssResponse::error(crate::error::INTERNAL_ERROR),
    }
}

pub async fn get_bucket_acl(
    State(storage): State<Arc<dyn Storage>>,
    axum::extract::Path(bucket): axum::extract::Path<String>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }
    OssResponse::get_bucket_acl()
}

pub async fn put_bucket_acl(
    State(storage): State<Arc<dyn Storage>>,
    axum::extract::Path(bucket): axum::extract::Path<String>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }
    OssResponse::put_bucket()
}

pub async fn get_bucket_info(
    State(storage): State<Arc<dyn Storage>>,
    axum::extract::Path(bucket): axum::extract::Path<String>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }
    OssResponse::get_bucket_info(&bucket)
}

pub async fn get_bucket_location(
    State(storage): State<Arc<dyn Storage>>,
    axum::extract::Path(bucket): axum::extract::Path<String>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }
    OssResponse::get_bucket_location()
}
