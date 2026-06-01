use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};


use crate::error::{INVALID_ARGUMENT, MISSING_CONTENT_LENGTH, OBJECT_NOT_APPENDABLE};
use crate::config::MAX_OBJECT_FILE_SIZE;
use crate::models::{DeleteRequest, Metadata};
use crate::response::OssResponse;
use crate::storage::Storage;

pub async fn put_object(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let content_length = headers
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(-1);

    if content_length < 0 && !is_chunked(&headers) {
        return OssResponse::error(MISSING_CONTENT_LENGTH);
    }

    if content_length > MAX_OBJECT_FILE_SIZE {
        return OssResponse::error(INVALID_ARGUMENT);
    }

    let obj_dir = storage.object_dir(&bucket, &object);
    let content_path = storage.object_content_path(&bucket, &object);

    let _ = std::fs::remove_dir_all(&obj_dir);
    std::fs::create_dir_all(&obj_dir).ok();

    if let Err(_e) = std::fs::write(&content_path, &body) {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");

    let meta = Metadata {
        bucket: bucket.clone(),
        object: object.clone(),
        size: body.len() as i64,
        content_type: content_type.to_string(),
        md5: "00000000000000000000000000000000".to_string(),
        ..Metadata::new(&bucket, &object)
    };

    let meta_path = storage.object_metadata_path(&bucket, &object);
    let _ = storage.save_metadata(&meta_path, &meta);

    OssResponse::put_object()
}

pub async fn get_object(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let meta_path = storage.object_metadata_path(&bucket, &object);
    let meta = match storage.load_metadata(&meta_path) {
        Ok(m) => m,
        Err(_) => return OssResponse::no_such_object(&bucket, &object),
    };

    let content_path = storage.object_content_path(&bucket, &object);
    let data = match std::fs::read(&content_path) {
        Ok(d) => d,
        Err(_) => return OssResponse::no_such_object(&bucket, &object),
    };

    let content_type = if meta.content_type.is_empty() {
        "application/octet-stream"
    } else {
        &meta.content_type
    };

    if let Some(range_str) = headers.get("range").and_then(|v| v.to_str().ok()) {
        if let Some(range) = parse_range(range_str, data.len() as i64) {
            let end = range.end.min(data.len() as i64);
            let sliced = data[range.start as usize..end as usize].to_vec();
            let content_range = format!("bytes {}-{}/{}", range.start, end - 1, data.len());
            return if !meta.content_type.is_empty() {
                OssResponse::get_object_range(sliced, &meta.content_type, &content_range)
            } else {
                OssResponse::get_object_range(sliced, content_type, &content_range)
            };
        }
    }

    OssResponse::get_object(data, content_type)
}

pub async fn head_object(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let meta_path = storage.object_metadata_path(&bucket, &object);
    let meta = match storage.load_metadata(&meta_path) {
        Ok(m) => m,
        Err(_) => return OssResponse::no_such_object(&bucket, &object),
    };

    OssResponse::head_object(meta.size)
}

pub async fn delete_object(
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

pub async fn get_object_meta_handler(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let meta_path = storage.object_metadata_path(&bucket, &object);
    let meta = match storage.load_metadata(&meta_path) {
        Ok(m) => m,
        Err(_) => return OssResponse::no_such_object(&bucket, &object),
    };

    OssResponse::get_object_meta(meta.size)
}

pub async fn get_object_acl_handler(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let meta_path = storage.object_metadata_path(&bucket, &object);
    if storage.load_metadata(&meta_path).is_err() {
        return OssResponse::no_such_object(&bucket, &object);
    }

    OssResponse::get_object_acl()
}

pub async fn put_object_acl_handler(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let meta_path = storage.object_metadata_path(&bucket, &object);
    if storage.load_metadata(&meta_path).is_err() {
        return OssResponse::no_such_object(&bucket, &object);
    }

    OssResponse::put_object()
}

pub async fn append_object(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let obj_dir = storage.object_dir(&bucket, &object);
    std::fs::create_dir_all(&obj_dir).ok();

    let content_path = storage.object_content_path(&bucket, &object);

    let meta_path = storage.object_metadata_path(&bucket, &object);
    if meta_path.exists() {
        if let Ok(meta) = storage.load_metadata(&meta_path) {
            if !meta.appendable {
                return OssResponse::error(OBJECT_NOT_APPENDABLE);
            }
        }
    }

    let current_size = std::fs::metadata(&content_path).map(|m| m.len()).unwrap_or(0) as i64;

    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&content_path)
    {
        let _ = file.write_all(&body);
    }

    let new_size = current_size + body.len() as i64;

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");

    let meta = Metadata {
        bucket: bucket.clone(),
        object: object.clone(),
        size: new_size,
        content_type: content_type.to_string(),
        md5: "00000000000000000000000000000000".to_string(),
        appendable: true,
        ..Metadata::new(&bucket, &object)
    };
    let _ = storage.save_metadata(&meta_path, &meta);

    OssResponse::append_object(new_size)
}

pub async fn copy_object(
    State(storage): State<Arc<dyn Storage>>,
    Path((dst_bucket, dst_object)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let copy_source = match headers.get("x-oss-copy-source").and_then(|v| v.to_str().ok()) {
        Some(s) => s.trim_start_matches('/').to_string(),
        None => return OssResponse::not_implemented(),
    };

    let parts: Vec<&str> = copy_source.splitn(2, '/').collect();
    if parts.len() < 2 {
        return OssResponse::not_implemented();
    }
    let src_bucket = parts[0];
    let src_object = parts[1];

    if !storage.bucket_exists(src_bucket) {
        return OssResponse::no_such_bucket(src_bucket);
    }

    let src_content_path = storage.object_content_path(src_bucket, src_object);
    let data = match std::fs::read(&src_content_path) {
        Ok(d) => d,
        Err(_) => return OssResponse::no_such_object(src_bucket, src_object),
    };

    if !storage.bucket_exists(&dst_bucket) {
        let _ = std::fs::create_dir_all(storage.bucket_dir(&dst_bucket));
        let meta = Metadata::new(&dst_bucket, "");
        let meta_path = storage.bucket_metadata_path(&dst_bucket);
        let _ = storage.save_metadata(&meta_path, &meta);
    }

    let dst_obj_dir = storage.object_dir(&dst_bucket, &dst_object);
    std::fs::create_dir_all(&dst_obj_dir).ok();
    let dst_content_path = storage.object_content_path(&dst_bucket, &dst_object);
    let _ = std::fs::write(&dst_content_path, &data);

    let meta = Metadata {
        bucket: dst_bucket.clone(),
        object: dst_object.clone(),
        size: data.len() as i64,
        content_type: "application/octet-stream".to_string(),
        md5: "00000000000000000000000000000000".to_string(),
        creation_date: chrono::Utc::now().to_rfc3339(),
        modified_date: chrono::Utc::now().to_rfc3339(),
        ..Metadata::new(&dst_bucket, &dst_object)
    };
    let dst_meta_path = storage.object_metadata_path(&dst_bucket, &dst_object);
    let _ = storage.save_metadata(&dst_meta_path, &meta);

    OssResponse::copy_object()
}

pub async fn delete_multiple_objects(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
    body: Bytes,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let body_str = String::from_utf8_lossy(&body);
    let delete_req: DeleteRequest = match quick_xml::de::from_str(&body_str) {
        Ok(req) => req,
        Err(_) => return OssResponse::error(crate::error::BAD_REQUEST),
    };

    let mut deleted_keys = Vec::new();
    for obj in &delete_req.objects {
        let obj_dir = storage.object_dir(&bucket, &obj.key);
        let _ = std::fs::remove_dir_all(&obj_dir);
        deleted_keys.push(obj.key.clone());
    }

    OssResponse::delete_multiple_objects(&deleted_keys)
}

// ============================================================
// Symlink — PUT/GET
// ============================================================

pub async fn put_symlink(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
    headers: HeaderMap,
    _body: Bytes,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let target = match headers.get("x-oss-symlink-target").and_then(|v| v.to_str().ok()) {
        Some(t) => t.to_string(),
        None => return OssResponse::error(crate::error::BAD_REQUEST),
    };

    let obj_dir = storage.object_dir(&bucket, &object);
    std::fs::create_dir_all(&obj_dir).ok();

    // 创建空内容文件
    let content_path = storage.object_content_path(&bucket, &object);
    let _ = std::fs::write(&content_path, []);

    let meta = Metadata {
        bucket: bucket.clone(),
        object: object.clone(),
        size: 0,
        content_type: "application/octet-stream".to_string(),
        md5: "00000000000000000000000000000000".to_string(),
        symlink_target: target,
        ..Metadata::new(&bucket, &object)
    };
    let meta_path = storage.object_metadata_path(&bucket, &object);
    let _ = storage.save_metadata(&meta_path, &meta);

    OssResponse::put_symlink()
}

pub async fn get_symlink(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let meta_path = storage.object_metadata_path(&bucket, &object);
    let meta = match storage.load_metadata(&meta_path) {
        Ok(m) => m,
        Err(_) => return OssResponse::no_such_object(&bucket, &object),
    };

    if meta.symlink_target.is_empty() {
        return OssResponse::no_such_object(&bucket, &object);
    }

    let parts: Vec<&str> = meta.symlink_target.splitn(2, '/').collect();
    let (target_size, target_etag) = if parts.len() == 2 {
        let target_meta_path = storage.object_metadata_path(parts[0], parts[1]);
        if let Ok(target_meta) = storage.load_metadata(&target_meta_path) {
            (target_meta.size, target_meta.md5)
        } else {
            default_target_info()
        }
    } else {
        default_target_info()
    };

    OssResponse::get_symlink(
        &meta.symlink_target,
        &target_etag,
        target_size,
        &chrono::Utc::now().to_rfc3339(),
    )
}

// ============================================================
// RestoreObject
// ============================================================

pub async fn restore_object(
    State(storage): State<Arc<dyn Storage>>,
    Path((bucket, object)): Path<(String, String)>,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let meta_path = storage.object_metadata_path(&bucket, &object);
    let mut meta = match storage.load_metadata(&meta_path) {
        Ok(m) => m,
        Err(_) => return OssResponse::no_such_object(&bucket, &object),
    };

    meta.restore_status = Some("ongoing-request".to_string());
    let _ = storage.save_metadata(&meta_path, &meta);

    OssResponse::accepted()
}

// ============================================================
// PostObject (表单上传)
// ============================================================

/// 从 multipart/form-data 的 Bytes 中提取表单字段
fn extract_multipart_field(body: &[u8], field_name: &str) -> Option<String> {
    let body_str = String::from_utf8_lossy(body);
    // 找 name="field_name"
    let search = format!("name=\"{field_name}\"");
    if let Some(pos) = body_str.find(&search) {
        // 跳过 name="xxx" 找到值开始位置（两个连续的 \r\n\r\n 之后）
        if let Some(value_start) = body_str[pos..].find("\r\n\r\n") {
            let value_start = pos + value_start + 4;
            // 值到下一个 \r\n 或结束
            if let Some(value_end) = body_str[value_start..].find("\r\n") {
                let value = &body_str[value_start..value_start + value_end];
                return Some(value.to_string());
            }
        }
    }
    None
}

/// 从 multipart/form-data 的 Bytes 中提取文件数据
fn extract_multipart_file(body: &[u8]) -> (Vec<u8>, String) {
    let body_str = String::from_utf8_lossy(body);
    // 找 name="file"
    if let Some(pos) = body_str.find("name=\"file\"") {
        // 跳过 Content-Type 行和空行
        if let Some(content_type_start) = body_str[pos..].find("Content-Type: ") {
            let ct_part = &body_str[pos + content_type_start + 14..];
            let content_type = ct_part.split("\r\n").next().unwrap_or("application/octet-stream").to_string();

            // 找到 \r\n\r\n 开始数据
            if let Some(data_start_rel) = body_str[pos + content_type_start..].find("\r\n\r\n") {
                let data_start = pos + content_type_start + data_start_rel + 4;
                // 文件数据到下一个 \r\n 或边界
                if let Some(data_end) = body_str[data_start..].find("\r\n") {
                    let file_bytes = body[data_start..data_start + data_end].to_vec();
                    return (file_bytes, content_type);
                }
                // 如果找不到 \r\n，说明数据到末尾
                return (body[data_start..].to_vec(), content_type);
            }
        }
    }
    (Vec::new(), "application/octet-stream".to_string())
}

pub async fn post_object(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
    body: Bytes,
) -> Response {
    if !storage.bucket_exists(&bucket) {
        return OssResponse::no_such_bucket(&bucket);
    }

    let object_key = extract_multipart_field(&body, "key").unwrap_or_default();
    if object_key.is_empty() {
        return OssResponse::error(crate::error::BAD_REQUEST);
    }

    let (file_data, content_type) = extract_multipart_file(&body);

    let obj_dir = storage.object_dir(&bucket, &object_key);
    let _ = std::fs::remove_dir_all(&obj_dir);
    std::fs::create_dir_all(&obj_dir).ok();

    let content_path = storage.object_content_path(&bucket, &object_key);
    if std::fs::write(&content_path, &file_data).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let meta = Metadata {
        bucket: bucket.clone(),
        object: object_key.clone(),
        size: file_data.len() as i64,
        content_type,
        md5: "00000000000000000000000000000000".to_string(),
        ..Metadata::new(&bucket, &object_key)
    };
    let meta_path = storage.object_metadata_path(&bucket, &object_key);
    let _ = storage.save_metadata(&meta_path, &meta);

    OssResponse::ok_no_content()
}

fn default_target_info() -> (i64, String) {
    (0, "00000000000000000000000000000000".to_string())
}

fn is_chunked(headers: &HeaderMap) -> bool {
    headers
        .get("transfer-encoding")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("chunked"))
        .unwrap_or(false)
}

fn parse_range(range_str: &str, file_size: i64) -> Option<RangeInclusive> {
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

    Some(RangeInclusive { start, end })
}

struct RangeInclusive {
    start: i64,
    end: i64,
}
