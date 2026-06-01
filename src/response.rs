use axum::http::header;
use axum::response::Response;
use quick_xml::se::to_string;

use crate::config::{ALIYUN_OSS_SERVER, HOST, OWNER_DISPLAY_NAME, OWNER_ID};
use crate::error::{ErrorCode, NOT_FOUND, NO_SUCH_BUCKET, NO_SUCH_CONFIGURATION, NOT_IMPLEMENTED};
use crate::models::{
    AccessControlList, AccessControlPolicy, BucketInfo, Content, DeleteResult, ErrorResult,
    InitiateMultipartUploadResult, ListAllMyBucketsResult, ListBucketResult, ListMultipartUploadsResult,
    ListPartsResult, Owner, Part, SymlinkTargetResponse, Upload, UploadPartCopyResult, XmlBucket,
};
use crate::storage::{ListObjectEntry, Storage};

fn xml_response<T: serde::Serialize>(data: &T) -> Result<String, String> {
    let xml_header = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let body = to_string(data).map_err(|e| e.to_string())?;
    Ok(format!("{xml_header}{body}"))
}

fn build_error_xml(code: &ErrorCode, bucket: Option<&str>) -> String {
    let err = ErrorResult {
        code: code.error_code.to_string(),
        message: code.message.to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        host_id: HOST.to_string(),
        bucket_name: bucket.map(|s| s.to_string()),
    };
    xml_response(&err).unwrap_or_default()
}

pub struct OssResponse;

impl OssResponse {
    pub fn error(code: ErrorCode) -> Response {
        let body = build_error_xml(&code, None);
        Response::builder()
            .status(code.status_code)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn error_with_bucket(code: ErrorCode, bucket: &str) -> Response {
        let body = build_error_xml(&code, Some(bucket));
        Response::builder()
            .status(code.status_code)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn not_implemented() -> Response {
        Self::error(NOT_IMPLEMENTED)
    }

    pub fn options() -> Response {
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "PUT, POST, HEAD, GET, OPTIONS")
            .header(
                "Access-Control-Allow-Headers",
                "Accept, Content-Type, Authorization, Content-Length, ETag, X-CSRF-Token, Content-Disposition",
            )
            .header("Access-Control-Expose-Headers", "ETag")
            .body(axum::body::Body::empty())
            .unwrap()
    }

    pub fn no_such_bucket(bucket: &str) -> Response {
        let body = ErrorResult {
            code: NO_SUCH_BUCKET.error_code.to_string(),
            message: NO_SUCH_BUCKET.message.to_string(),
            request_id: uuid::Uuid::new_v4().to_string(),
            host_id: HOST.to_string(),
            bucket_name: Some(bucket.to_string()),
        };
        let body = xml_response(&body).unwrap_or_default();
        Response::builder()
            .status(NO_SUCH_BUCKET.status_code)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn no_such_object(bucket: &str, _object: &str) -> Response {
        let body = ErrorResult {
            code: NOT_FOUND.error_code.to_string(),
            message: NOT_FOUND.message.to_string(),
            request_id: uuid::Uuid::new_v4().to_string(),
            host_id: HOST.to_string(),
            bucket_name: Some(bucket.to_string()),
        };
        let body = xml_response(&body).unwrap_or_default();
        Response::builder()
            .status(404)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn check_bucket_exists(storage: &dyn Storage, bucket: &str) -> Option<Response> {
        if !storage.bucket_exists(bucket) {
            return Some(Self::no_such_bucket(bucket));
        }
        None
    }

    pub fn ok(_cmd: &str, headers: Vec<(&str, String)>, body: Vec<u8>) -> Response {
        let mut builder = Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string());

        for (key, val) in headers {
            builder = builder.header(key, val);
        }

        builder.body(axum::body::Body::from(body)).unwrap()
    }

    pub fn ok_no_content() -> Response {
        Response::builder()
            .status(204)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .body(axum::body::Body::empty())
            .unwrap()
    }

    pub fn list_buckets(buckets: &[crate::storage::ListBucketInfo]) -> Response {
        let result = ListAllMyBucketsResult {
            owner: Owner {
                display_name: OWNER_DISPLAY_NAME.to_string(),
                id: OWNER_ID.to_string(),
            },
            buckets: crate::models::BucketsInner {
                bucket: buckets
                    .iter()
                    .map(|b| crate::models::Bucket {
                        name: b.name.clone(),
                        creation_date: b.creation_date.clone(),
                    })
                    .collect(),
            },
        };
        let body = xml_response(&result).unwrap_or_default();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn list_objects(bucket: &str, contents: &[ListObjectEntry], common_prefixes: &[String], prefix: &str, marker: &str, max_keys: i32, delimiter: &str, next_marker: &str, is_truncated: bool) -> Response {
        let result = ListBucketResult {
            name: bucket.to_string(),
            prefix: prefix.to_string(),
            marker: marker.to_string(),
            max_keys: max_keys.to_string(),
            delimiter: delimiter.to_string(),
            encoding_type: "url".to_string(),
            next_marker: next_marker.to_string(),
            is_truncated,
            contents: contents
                .iter()
                .map(|c| Content {
                    key: c.key.clone(),
                    last_modified: c.last_modified.clone(),
                    etag: c.etag.clone(),
                    object_type: "Normal".to_string(),
                    size: c.size,
                    storage_class: c.storage_class.clone(),
                    owner: Owner {
                        id: OWNER_ID.to_string(),
                        display_name: OWNER_DISPLAY_NAME.to_string(),
                    },
                })
                .collect(),
            common_prefixes: common_prefixes
                .iter()
                .map(|p| crate::models::CommonPrefix { prefix: p.clone() })
                .collect(),
        };
        let body = xml_response(&result).unwrap_or_default();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn put_bucket() -> Response {
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header("Location", "oss-example")
            .header("Access-Control-Allow-Origin", "*")
            .body(axum::body::Body::empty())
            .unwrap()
    }

    pub fn get_bucket_info(bucket: &str) -> Response {
        let info = BucketInfo {
            bucket: XmlBucket {
                creation_date: String::new(),
                extranet_endpoint: "oss-cn-hangzhou-zmf.aliyuncs.com".to_string(),
                intranet_endpoint: "oss-cn-hangzhou-zmf-internal.aliyuncs.com".to_string(),
                location: "cn-hangzhou".to_string(),
                name: bucket.to_string(),
                storage_class: "Standard".to_string(),
                owner: Owner {
                    display_name: "1390402650033793".to_string(),
                    id: "1390402650033793".to_string(),
                },
                access_control_list: AccessControlList {
                    grant: "private".to_string(),
                },
            },
        };
        let body = xml_response(&info).unwrap_or_default();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header("x-oss-server-time", chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn get_bucket_acl() -> Response {
        let policy = AccessControlPolicy {
            owner: Owner {
                id: OWNER_ID.to_string(),
                display_name: OWNER_DISPLAY_NAME.to_string(),
            },
            access_control_list: AccessControlList {
                grant: "private".to_string(),
            },
        };
        let body = xml_response(&policy).unwrap_or_default();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn get_bucket_location() -> Response {
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(
                r#"<?xml version="1.0" encoding="UTF-8"?><LocationConstraint>oss-cn-hangzhou</LocationConstraint>"#,
            ))
            .unwrap()
    }

    pub fn put_object() -> Response {
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header("ETag", "00000000000")
            .header("x-oss-bucket-version", "1418321259")
            .body(axum::body::Body::empty())
            .unwrap()
    }

    pub fn get_object(body: Vec<u8>, content_type: &str) -> Response {
        let now = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CONTENT_LENGTH, body.len().to_string())
            .header("Accept-Ranges", "bytes")
            .header("ETag", "\"00000000000\"")
            .header("Last-Modified", &now)
            .header("x-oss-object-type", "Appendable")
            .header("x-oss-storage-class", "Standard")
            .header("Access-Control-Allow-Origin", "*")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn get_object_range(body: Vec<u8>, content_type: &str, content_range: &str) -> Response {
        let now = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        Response::builder()
            .status(206)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CONTENT_LENGTH, body.len().to_string())
            .header("Content-Range", content_range)
            .header("Accept-Ranges", "bytes")
            .header("ETag", "\"00000000000\"")
            .header("Last-Modified", &now)
            .header("x-oss-object-type", "Appendable")
            .header("x-oss-storage-class", "Standard")
            .header("Access-Control-Allow-Origin", "*")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn head_object(size: i64) -> Response {
        let now = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header("Accept-Ranges", "bytes")
            .header("ETag", "\"00000000000\"")
            .header(header::CONTENT_LENGTH, size.to_string())
            .header("Last-Modified", &now)
            .header("x-oss-object-type", "Appendable")
            .header("x-oss-storage-class", "Standard")
            .header("x-oss-server-time", now)
            .header("x-oss-next-append-position", size.to_string())
            .body(axum::body::Body::empty())
            .unwrap()
    }

    pub fn get_object_meta(size: i64) -> Response {
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_LENGTH, size.to_string())
            .header("Last-Modified", chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .header("ETag", "00000000000")
            .body(axum::body::Body::empty())
            .unwrap()
    }

    pub fn get_object_acl() -> Response {
        let policy = AccessControlPolicy {
            owner: Owner {
                id: OWNER_ID.to_string(),
                display_name: OWNER_DISPLAY_NAME.to_string(),
            },
            access_control_list: AccessControlList {
                grant: "default".to_string(),
            },
        };
        let body = xml_response(&policy).unwrap_or_default();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn append_object(next_position: i64) -> Response {
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header("x-oss-next-append-position", next_position.to_string())
            .body(axum::body::Body::empty())
            .unwrap()
    }

    pub fn initiate_multipart_upload(bucket: &str, object: &str) -> Response {
        let result = InitiateMultipartUploadResult {
            bucket: bucket.to_string(),
            key: object.to_string(),
            upload_id: "0000000".to_string(),
        };
        let body = xml_response(&result).unwrap_or_default();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn complete_multipart_upload(bucket: &str, object: &str) -> Response {
        let result = crate::models::CompleteMultipartUploadResult {
            location: "Location".to_string(),
            bucket: bucket.to_string(),
            key: object.to_string(),
            etag: "00000000000".to_string(),
        };
        let body = xml_response(&result).unwrap_or_default();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn copy_object() -> Response {
        let body = r#"<?xml version="1.0" encoding="UTF-8"?><CopyObjectResult><LastModified>2012-02-24T08:42:32.000Z</LastModified><ETag>"00000000"</ETag></CopyObjectResult>"#;
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .header("Last-Modified", chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn delete_multiple_objects(deleted_keys: &[String]) -> Response {
        let result = DeleteResult {
            deleted: deleted_keys
                .iter()
                .map(|k| crate::models::Deleted { key: k.clone() })
                .collect(),
        };
        let body = xml_response(&result).unwrap_or_default();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn list_parts(bucket: &str, object: &str, parts: &[Part]) -> Response {
        let result = ListPartsResult {
            bucket: bucket.to_string(),
            key: object.to_string(),
            upload_id: "0000000".to_string(),
            part_number_marker: 0,
            next_part_number_marker: 0,
            max_parts: 1000,
            is_truncated: false,
            parts: parts.to_vec(),
        };
        let body = xml_response(&result).unwrap_or_default();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    pub fn list_multipart_uploads(bucket: &str, uploads: &[(String, String)]) -> Response {
        let now = chrono::Utc::now().to_rfc3339();
        let result = ListMultipartUploadsResult {
            bucket: bucket.to_string(),
            key_marker: String::new(),
            upload_id_marker: String::new(),
            next_key_marker: String::new(),
            next_upload_id_marker: String::new(),
            max_uploads: 1000,
            delimiter: String::new(),
            is_truncated: false,
            uploads: uploads
                .iter()
                .map(|(key, upload_id)| Upload {
                    key: key.clone(),
                    upload_id: upload_id.clone(),
                    initiated: now.clone(),
                })
                .collect(),
        };
        let body = xml_response(&result).unwrap_or_default();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    // ============================================================
    // Bucket Config 响应
    // ============================================================

    /// 200 OK（配置 PUT/DELETE 成功）
    pub fn ok_bucket_config() -> Response {
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .body(axum::body::Body::empty())
            .unwrap()
    }

    /// 返回配置不存在错误
    pub fn no_such_configuration() -> Response {
        let body = ErrorResult {
            code: NO_SUCH_CONFIGURATION.error_code.to_string(),
            message: NO_SUCH_CONFIGURATION.message.to_string(),
            request_id: uuid::Uuid::new_v4().to_string(),
            host_id: HOST.to_string(),
            bucket_name: None,
        };
        let body = xml_response(&body).unwrap_or_default();
        Response::builder()
            .status(404)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    /// 返回 202 Accepted（RestoreObject）
    pub fn accepted() -> Response {
        Response::builder()
            .status(202)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .body(axum::body::Body::empty())
            .unwrap()
    }

    // ============================================================
    // Symlink 响应
    // ============================================================

    /// PUT Symlink 成功
    pub fn put_symlink() -> Response {
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .body(axum::body::Body::empty())
            .unwrap()
    }

    /// GET Symlink 返回目标信息
    pub fn get_symlink(target: &str, etag: &str, size: i64, last_modified: &str) -> Response {
        let result = SymlinkTargetResponse {
            target: target.to_string(),
            etag: etag.to_string(),
            size: size.to_string(),
            last_modified: last_modified.to_string(),
        };
        let body = xml_response(&result).unwrap_or_default();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header("x-oss-symlink-target", target)
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    // ============================================================
    // UploadPartCopy 响应
    // ============================================================

    pub fn upload_part_copy() -> Response {
        let result = UploadPartCopyResult {
            last_modified: chrono::Utc::now().to_rfc3339(),
            etag: "00000000000".to_string(),
        };
        let body = xml_response(&result).unwrap_or_default();
        Response::builder()
            .status(200)
            .header(header::SERVER, ALIYUN_OSS_SERVER)
            .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
            .header(header::CONTENT_TYPE, "application/xml")
            .body(axum::body::Body::from(body))
            .unwrap()
    }
}
