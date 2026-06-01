use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub bucket: String,
    pub object: String,
    #[serde(default = "default_acl")]
    pub acl: String,
    pub creation_date: String,
    pub modified_date: String,
    pub content_type: String,
    pub content_disposition: String,
    pub content_encoding: String,
    pub size: i64,
    pub part_size: i32,
    pub md5: String,
    pub appendable: bool,
    pub oss_metadata: HashMap<String, String>,
    pub custom_metadata: HashMap<String, String>,
    /// 符号链接目标路径（仅 Symlink 类型对象使用）
    #[serde(default)]
    pub symlink_target: String,
    /// 解冻状态（仅归档对象使用）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restore_status: Option<String>,
}

fn default_acl() -> String {
    "default".to_string()
}

impl Metadata {
    pub fn new(bucket: &str, object: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            object: object.to_string(),
            acl: "default".to_string(),
            creation_date: String::new(),
            modified_date: String::new(),
            content_type: String::new(),
            content_disposition: String::new(),
            content_encoding: String::new(),
            size: 0,
            part_size: 0,
            md5: String::new(),
            appendable: false,
            oss_metadata: HashMap::new(),
            custom_metadata: HashMap::new(),
            symlink_target: String::new(),
            restore_status: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dataset {
    pub cmd: String,
    pub bucket: String,
    pub object: String,
    pub md5: String,
    pub multipart: bool,
    pub content_type: String,
    pub content_length: i64,
    pub content_range: String,
    pub content_disposition: String,
    pub content_encoding: String,
    pub size: i64,
    pub part_size: i32,
    pub creation_date: String,
    pub modified_date: String,
    pub pos: i64,
    pub bytes_to_read: i64,
    pub acl: String,
    pub appendable: bool,
    pub oss_metadata: HashMap<String, String>,
    pub custom_metadata: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Default for Dataset {
    fn default() -> Self {
        Self::new()
    }
}

impl Dataset {
    pub fn new() -> Self {
        Self {
            cmd: String::new(),
            bucket: String::new(),
            object: String::new(),
            md5: String::new(),
            multipart: false,
            content_type: "application/octet-stream".to_string(),
            content_length: 0,
            content_range: String::new(),
            content_disposition: String::new(),
            content_encoding: String::new(),
            size: 0,
            part_size: 0,
            creation_date: String::new(),
            modified_date: String::new(),
            pos: 0,
            bytes_to_read: 0,
            acl: "default".to_string(),
            appendable: false,
            oss_metadata: HashMap::new(),
            custom_metadata: HashMap::new(),
            body: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    #[serde(rename = "DisplayName")]
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlList {
    #[serde(rename = "Grant")]
    pub grant: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlBucket {
    #[serde(rename = "CreationDate")]
    pub creation_date: String,
    #[serde(rename = "ExtranetEndpoint")]
    pub extranet_endpoint: String,
    #[serde(rename = "IntranetEndpoint")]
    pub intranet_endpoint: String,
    #[serde(rename = "Location")]
    pub location: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "StorageClass")]
    pub storage_class: String,
    #[serde(rename = "Owner")]
    pub owner: Owner,
    #[serde(rename = "AccessControlList")]
    pub access_control_list: AccessControlList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlPolicy {
    #[serde(rename = "Owner")]
    pub owner: Owner,
    #[serde(rename = "AccessControlList")]
    pub access_control_list: AccessControlList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketInfo {
    #[serde(rename = "Bucket")]
    pub bucket: XmlBucket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "CreationDate")]
    pub creation_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAllMyBucketsResult {
    #[serde(rename = "Owner")]
    pub owner: Owner,
    #[serde(rename = "Buckets")]
    pub buckets: BucketsInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketsInner {
    #[serde(rename = "Bucket")]
    pub bucket: Vec<Bucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "LastModified")]
    pub last_modified: String,
    #[serde(rename = "ETag")]
    pub etag: String,
    #[serde(rename = "Type")]
    pub object_type: String,
    #[serde(rename = "Size")]
    pub size: i64,
    #[serde(rename = "StorageClass")]
    pub storage_class: String,
    #[serde(rename = "Owner")]
    pub owner: Owner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonPrefix {
    #[serde(rename = "Prefix")]
    pub prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBucketResult {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Prefix")]
    pub prefix: String,
    #[serde(rename = "Marker")]
    pub marker: String,
    #[serde(rename = "MaxKeys")]
    pub max_keys: String,
    #[serde(rename = "Delimiter")]
    pub delimiter: String,
    #[serde(rename = "EncodingType")]
    pub encoding_type: String,
    #[serde(rename = "NextMarker")]
    pub next_marker: String,
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,
    #[serde(rename = "Contents")]
    pub contents: Vec<Content>,
    #[serde(rename = "CommonPrefixes")]
    pub common_prefixes: Vec<CommonPrefix>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiateMultipartUploadResult {
    #[serde(rename = "Bucket")]
    pub bucket: String,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "UploadId")]
    pub upload_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteMultipartUploadResult {
    #[serde(rename = "Location")]
    pub location: String,
    #[serde(rename = "Bucket")]
    pub bucket: String,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "ETag")]
    pub etag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostResponse {
    #[serde(rename = "Location")]
    pub location: String,
    #[serde(rename = "Bucket")]
    pub bucket: String,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "ETag")]
    pub etag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deleted {
    #[serde(rename = "Key")]
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResult {
    #[serde(rename = "Deleted")]
    pub deleted: Vec<Deleted>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResult {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "RequestId")]
    pub request_id: String,
    #[serde(rename = "HostId")]
    pub host_id: String,
    #[serde(rename = "BucketName", skip_serializing_if = "Option::is_none")]
    pub bucket_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Delete")]
pub struct DeleteRequest {
    #[serde(rename = "Object")]
    pub objects: Vec<DeleteObject>,
    #[serde(rename = "Quiet")]
    pub quiet: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteObject {
    #[serde(rename = "Key")]
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPartsResult {
    #[serde(rename = "Bucket")]
    pub bucket: String,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "UploadId")]
    pub upload_id: String,
    #[serde(rename = "PartNumberMarker")]
    pub part_number_marker: i32,
    #[serde(rename = "NextPartNumberMarker")]
    pub next_part_number_marker: i32,
    #[serde(rename = "MaxParts")]
    pub max_parts: i32,
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,
    #[serde(rename = "Part")]
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    #[serde(rename = "PartNumber")]
    pub part_number: i32,
    #[serde(rename = "LastModified")]
    pub last_modified: String,
    #[serde(rename = "ETag")]
    pub etag: String,
    #[serde(rename = "Size")]
    pub size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMultipartUploadsResult {
    #[serde(rename = "Bucket")]
    pub bucket: String,
    #[serde(rename = "KeyMarker")]
    pub key_marker: String,
    #[serde(rename = "UploadIdMarker")]
    pub upload_id_marker: String,
    #[serde(rename = "NextKeyMarker")]
    pub next_key_marker: String,
    #[serde(rename = "NextUploadIdMarker")]
    pub next_upload_id_marker: String,
    #[serde(rename = "MaxUploads")]
    pub max_uploads: i32,
    #[serde(rename = "Delimiter")]
    pub delimiter: String,
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,
    #[serde(rename = "Upload")]
    pub uploads: Vec<Upload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upload {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "UploadId")]
    pub upload_id: String,
    #[serde(rename = "Initiated")]
    pub initiated: String,
}

// ============================================================
// Bucket 配置模型 (TOML 存储)
// ============================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BucketConfig {
    pub logging: Option<LoggingConfig>,
    pub website: Option<WebsiteConfig>,
    pub referer: Option<RefererConfig>,
    pub lifecycle: Option<LifecycleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub target_bucket: String,
    pub target_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteConfig {
    pub index_document: String,
    pub error_document: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefererConfig {
    pub allow_empty: bool,
    pub referer_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleConfig {
    pub rules: Vec<LifecycleRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleRule {
    pub id: String,
    pub prefix: String,
    pub status: String,
    #[serde(default)]
    pub expiration_days: Option<i32>,
    #[serde(default)]
    pub transition_days: Option<i32>,
    #[serde(default)]
    pub transition_storage_class: Option<String>,
}

// ============================================================
// Bucket 配置 XML 模型（用于接收请求 XML 和返回响应 XML）
// ============================================================

/// BucketLoggingStatus XML
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "BucketLoggingStatus")]
pub struct BucketLoggingStatus {
    #[serde(rename = "LoggingEnabled")]
    pub logging_enabled: Option<LoggingEnabled>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingEnabled {
    #[serde(rename = "TargetBucket")]
    pub target_bucket: String,
    #[serde(rename = "TargetPrefix")]
    pub target_prefix: String,
}

/// BucketLoggingStatus 响应（GET）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "BucketLoggingStatus")]
pub struct BucketLoggingStatusResponse {
    #[serde(rename = "LoggingEnabled")]
    pub logging_enabled: Option<LoggingEnabledResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingEnabledResponse {
    #[serde(rename = "TargetBucket")]
    pub target_bucket: String,
    #[serde(rename = "TargetPrefix")]
    pub target_prefix: String,
}

/// WebsiteConfiguration XML
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "WebsiteConfiguration")]
pub struct WebsiteConfiguration {
    #[serde(rename = "IndexDocument")]
    pub index_document: IndexDocument,
    #[serde(rename = "ErrorDocument")]
    pub error_document: ErrorDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocument {
    #[serde(rename = "Suffix")]
    pub suffix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDocument {
    #[serde(rename = "Key")]
    pub key: String,
}

/// RefererConfiguration XML
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "RefererConfiguration")]
pub struct RefererConfiguration {
    #[serde(rename = "AllowEmptyReferer")]
    pub allow_empty: bool,
    #[serde(rename = "RefererList")]
    pub referer_list: RefererList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefererList {
    #[serde(rename = "Referer", default)]
    pub referers: Vec<String>,
}

/// LifecycleConfiguration XML
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "LifecycleConfiguration")]
pub struct LifecycleConfiguration {
    #[serde(rename = "Rule")]
    pub rules: Vec<LifecycleRuleXml>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleRuleXml {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Prefix")]
    pub prefix: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Expiration")]
    pub expiration: Option<Expiration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expiration {
    #[serde(rename = "Days")]
    pub days: i32,
}

/// Symlink 响应 XML
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "SymlinkTarget")]
pub struct SymlinkTargetResponse {
    #[serde(rename = "Target")]
    pub target: String,
    #[serde(rename = "ETag")]
    pub etag: String,
    #[serde(rename = "Size")]
    pub size: String,
    #[serde(rename = "LastModified")]
    pub last_modified: String,
}

/// UploadPartCopy 响应 XML
/// 阿里云 OSS 返回 `<CopyPartResult>` 作为根元素
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "CopyPartResult")]
pub struct UploadPartCopyResult {
    #[serde(rename = "LastModified")]
    pub last_modified: String,
    #[serde(rename = "ETag")]
    pub etag: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_referer_deser_empty() {
        let xml = "<RefererConfiguration><AllowEmptyReferer>true</AllowEmptyReferer><RefererList></RefererList></RefererConfiguration>";
        let result: Result<RefererConfiguration, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok(), "Empty RefererList deserialization failed: {:?}", result.err());
    }

    #[test]
    fn test_referer_deser_with_items() {
        let xml = "<RefererConfiguration><AllowEmptyReferer>false</AllowEmptyReferer><RefererList><Referer>www.t1.com</Referer><Referer>www.t2.com</Referer></RefererList></RefererConfiguration>";
        let result: Result<RefererConfiguration, _> = quick_xml::de::from_str(xml);
        assert!(result.is_ok());
        let cfg = result.unwrap();
        assert!(!cfg.allow_empty);
        assert_eq!(cfg.referer_list.referers.len(), 2);
    }
}
