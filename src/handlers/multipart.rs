use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;


use crate::models::{Metadata, Part};
use crate::response::OssResponse;
use crate::storage::Storage;

pub async fn initiate_multipart_upload(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let obj_dir = storage.object_dir(&bucket, &object);
    std::fs::create_dir_all(&obj_dir).ok();

    OssResponse::initiate_multipart_upload(&bucket, &object)
}

pub async fn upload_part(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    _headers: HeaderMap,
    body: Bytes,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let part_number: i32 = params
        .get("partNumber")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let obj_dir = storage.object_dir(&bucket, &object);
    std::fs::create_dir_all(&obj_dir).ok();

    let part_path = storage.object_content_part_path(&bucket, &object, part_number);
    let _ = std::fs::write(&part_path, &body);

    OssResponse::put_object()
}

pub async fn complete_multipart_upload(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
    _body: Bytes,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let _obj_dir = storage.object_dir(&bucket, &object);
    let content_path = storage.object_content_path(&bucket, &object);

    let mut part_number = 1;
    let mut total_size: i64 = 0;
    let mut merged = Vec::new();

    loop {
        let part_path = storage.object_content_part_path(&bucket, &object, part_number);
        if !part_path.exists() {
            break;
        }
        if let Ok(data) = std::fs::read(&part_path) {
            total_size += data.len() as i64;
            merged.extend_from_slice(&data);
        }
        part_number += 1;
    }

    let _ = std::fs::write(&content_path, &merged);

    let meta = Metadata {
        bucket: bucket.clone(),
        object: object.clone(),
        size: total_size,
        md5: "00000000000000000000000000000000".to_string(),
        ..Metadata::new(&bucket, &object)
    };
    let meta_path = storage.object_metadata_path(&bucket, &object);
    let _ = storage.save_metadata(&meta_path, &meta);

    OssResponse::complete_multipart_upload(&bucket, &object)
}

pub async fn abort_multipart_upload(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let obj_dir = storage.object_dir(&bucket, &object);
    let _ = std::fs::remove_dir_all(&obj_dir);

    OssResponse::ok_no_content()
}

pub async fn list_parts(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let obj_dir = storage.object_dir(&bucket, &object);
    if !obj_dir.exists() {
        return OssResponse::no_such_object(&bucket, &object);
    }

    let mut parts: Vec<Part> = Vec::new();
    let mut part_number = 1;
    loop {
        let part_path = storage.object_content_part_path(&bucket, &object, part_number);
        if !part_path.exists() {
            break;
        }
        if let Ok(meta) = std::fs::metadata(&part_path) {
            parts.push(Part {
                part_number,
                last_modified: chrono::Utc::now().to_rfc3339(),
                etag: "00000000000".to_string(),
                size: meta.len() as i64,
            });
        }
        part_number += 1;
    }

    OssResponse::list_parts(&bucket, &object, &parts)
}

pub async fn list_multipart_uploads(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    match storage.list_multipart_uploads(&bucket) {
        Ok(uploads) => OssResponse::list_multipart_uploads(&bucket, &uploads),
        Err(_) => OssResponse::error(crate::error::INTERNAL_ERROR),
    }
}

pub async fn upload_part_copy(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let part_number: i32 = params
        .get("partNumber")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let copy_source = match headers.get("x-oss-copy-source").and_then(|v| v.to_str().ok()) {
        Some(s) => s.trim_start_matches('/').to_string(),
        None => return OssResponse::error(crate::error::BAD_REQUEST),
    };

    let parts: Vec<&str> = copy_source.splitn(2, '/').collect();
    if parts.len() < 2 {
        return OssResponse::error(crate::error::BAD_REQUEST);
    }
    let src_bucket = parts[0];
    let src_object = parts[1];

    if !storage.bucket_exists(src_bucket) {
        return OssResponse::no_such_bucket(src_bucket);
    }

    // 读取源对象内容
    let src_content_path = storage.object_content_path(src_bucket, src_object);
    let data = match std::fs::read(&src_content_path) {
        Ok(d) => d,
        Err(_) => return OssResponse::no_such_object(src_bucket, src_object),
    };

    // 处理范围拷贝
    let copy_data: Vec<u8> = if let Some(range_str) =
        headers.get("x-oss-copy-source-range").and_then(|v| v.to_str().ok())
    {
        if let Some(range) = parse_part_copy_range(range_str, data.len() as i64) {
            let end = range.end.min(data.len() as i64);
            data[range.start as usize..end as usize].to_vec()
        } else {
            data
        }
    } else {
        data
    };

    // 写入目标 part
    let obj_dir = storage.object_dir(&bucket, &object);
    std::fs::create_dir_all(&obj_dir).ok();

    let part_path = storage.object_content_part_path(&bucket, &object, part_number);
    let _ = std::fs::write(&part_path, &copy_data);

    OssResponse::upload_part_copy()
}

struct CopyRange {
    start: i64,
    end: i64,
}

fn parse_part_copy_range(range_str: &str, file_size: i64) -> Option<CopyRange> {
    if !range_str.starts_with("bytes=") {
        return None;
    }
    let range_val = range_str.trim_start_matches("bytes=");
    let parts: Vec<&str> = range_val.splitn(2, '-').collect();
    if parts.len() != 2 {
        return None;
    }

    let start: i64 = parts[0].parse().ok()?;
    let end: i64 = if parts[1].is_empty() {
        file_size - 1
    } else {
        parts[1].parse().ok()?
    };

    Some(CopyRange { start, end })
}
