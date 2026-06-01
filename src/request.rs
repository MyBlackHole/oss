use std::collections::HashMap;

pub const PUT_BUCKET: &str = "PUT_BUCKET";
pub const PUT_BUCKET_ACL: &str = "PUT_BUCKET_ACL";
pub const PUT_BUCKET_LOGGING: &str = "PUT_BUCKET_LOGGING";
pub const PUT_BUCKET_REFERER: &str = "PUT_BUCKET_REFERER";
pub const PUT_BUCKET_WEBSITE: &str = "PUT_BUCKET_WEBSITE";
pub const PUT_BUCKET_LIFECYCLE: &str = "PUT_BUCKET_LIFECYCLE";

pub const PUT_OBJECT: &str = "PUT_OBJECT";
pub const PUT_OBJECT_ACL: &str = "PUT_OBJECT_ACL";
pub const PUT_SYMLINK: &str = "PUT_SYMLINK";
pub const PUT_UPLOAD_PART: &str = "PUT_UPLOAD_PART";
pub const PUT_UPLOAD_PART_COPY: &str = "PUT_UPLOAD_PART_COPY";
pub const PUT_COPY_OBJECT: &str = "PUT_COPY_OBJECT";

pub const LIST_BUCKETS: &str = "LIST_BUCKETS";
pub const GET_BUCKET: &str = "GET_BUCKET";
pub const GET_BUCKET_ACL: &str = "GET_BUCKET_ACL";
pub const GET_BUCKET_INFO: &str = "GET_BUCKET_INFO";
pub const GET_BUCKET_LOCATION: &str = "GET_BUCKET_LOCATION";
pub const GET_BUCKET_LOGGING: &str = "GET_BUCKET_LOGGING";
pub const GET_BUCKET_REFERER: &str = "GET_BUCKET_REFERER";
pub const GET_BUCKET_WEBSITE: &str = "GET_BUCKET_WEBSITE";
pub const GET_BUCKET_LIFECYCLE: &str = "GET_BUCKET_LIFECYCLE";

pub const GET_OBJECT: &str = "GET_OBJECT";
pub const GET_OBJECT_ACL: &str = "GET_OBJECT_ACL";
pub const GET_OBJECT_META: &str = "GET_OBJECT_META";
pub const GET_SYMLINK: &str = "GET_SYMLINK";

pub const GET_LIST_MULTIPART_UPLOADS: &str = "GET_LIST_MULTIPART_UPLOADS";
pub const GET_LIST_PARTS: &str = "GET_LIST_PARTS";

pub const HEAD_OBJECT: &str = "HEAD_OBJECT";

pub const DELETE_BUCKET: &str = "DELETE_BUCKET";
pub const DELETE_BUCKET_LOGGING: &str = "DELETE_BUCKET_LOGGING";
pub const DELETE_BUCKET_WEBSITE: &str = "DELETE_BUCKET_WEBSITE";
pub const DELETE_BUCKET_REFERER: &str = "DELETE_BUCKET_REFERER";
pub const DELETE_BUCKET_LIFECYCLE: &str = "DELETE_BUCKET_LIFECYCLE";
pub const DELETE_ABORT_MULTIPART_UPLOAD: &str = "DELETE_ABORT_MULTIPART_UPLOAD";

pub const DELETE_OBJECT: &str = "DELETE_OBJECT";
pub const DELETE_MULTIPLE_OBJECTS: &str = "DELETE_MULTIPLE_OBJECTS";

pub const POST_RESTORE_OBJECT: &str = "POST_RESTORE_OBJECT";
pub const POST_APPEND_OBJECT: &str = "POST_APPEND_OBJECT";
pub const POST_INIT_MULTIPART_UPLOAD: &str = "POST_INIT_MULTIPART_UPLOAD";
pub const POST_COMPLETE_MULTIPART_UPLOAD: &str = "POST_COMPLETE_MULTIPART_UPLOAD";
pub const POST_OBJECT: &str = "POST_OBJECT";
pub const POST_ELSE: &str = "POST_ELSE";

pub const REQUEST_ERROR: &str = "REQUEST_ERROR";

#[derive(Debug, Clone)]
pub struct OssRequest {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub cmd: String,
    pub bucket: String,
    pub object: String,
    pub src_bucket: String,
    pub src_object: String,
}

impl OssRequest {
    pub fn new(method: &str, path: &str, query: &HashMap<String, String>, headers: &HashMap<String, String>) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            query: query.clone(),
            headers: headers.clone(),
            cmd: String::new(),
            bucket: String::new(),
            object: String::new(),
            src_bucket: String::new(),
            src_object: String::new(),
        }
    }

    pub fn parse(&mut self) {
        match self.method.as_str() {
            "GET" | "HEAD" => self.parse_get(),
            "PUT" => self.parse_put(),
            "DELETE" => self.parse_delete(),
            "POST" => self.parse_post(),
            _ => {}
        }
    }

    pub fn parse_get(&mut self) {
        if self.path == "/" {
            if self.query.contains_key("uploads") {
                self.cmd = GET_LIST_MULTIPART_UPLOADS.to_string();
            } else if self.query.contains_key("logging") {
                self.cmd = GET_BUCKET_LOGGING.to_string();
            } else if self.query.contains_key("website") {
                self.cmd = GET_BUCKET_WEBSITE.to_string();
            } else if self.query.contains_key("referer") {
                self.cmd = GET_BUCKET_REFERER.to_string();
            } else if self.query.contains_key("lifecycle") {
                self.cmd = GET_BUCKET_LIFECYCLE.to_string();
            } else {
                self.cmd = LIST_BUCKETS.to_string();
            }
        } else {
            let elems: Vec<&str> = self.path[1..].split('/').collect();
            self.bucket = elems[0].to_string();

            if elems.len() < 2 || (elems.len() == 2 && elems[1].is_empty()) {
                if self.query.contains_key("uploads") {
                    self.cmd = GET_LIST_MULTIPART_UPLOADS.to_string();
                } else if self.query.contains_key("acl") {
                    self.cmd = GET_BUCKET_ACL.to_string();
                } else if self.query.contains_key("location") {
                    self.cmd = GET_BUCKET_LOCATION.to_string();
                } else if self.query.contains_key("bucketInfo") {
                    self.cmd = GET_BUCKET_INFO.to_string();
                } else if self.query.contains_key("logging") {
                    self.cmd = GET_BUCKET_LOGGING.to_string();
                } else if self.query.contains_key("website") {
                    self.cmd = GET_BUCKET_WEBSITE.to_string();
                } else if self.query.contains_key("referer") {
                    self.cmd = GET_BUCKET_REFERER.to_string();
                } else if self.query.contains_key("lifecycle") {
                    self.cmd = GET_BUCKET_LIFECYCLE.to_string();
                } else {
                    self.cmd = GET_BUCKET.to_string();
                }
            } else if self.query.contains_key("acl") {
                self.cmd = GET_BUCKET_ACL.to_string();
            } else if self.query.contains_key("objectMeta") {
                self.cmd = GET_OBJECT_META.to_string();
            } else if self.query.contains_key("symlink") {
                self.cmd = GET_SYMLINK.to_string();
            } else if self.query.contains_key("uploadId") {
                self.cmd = GET_LIST_PARTS.to_string();
            } else if self.method == "HEAD" {
                self.cmd = HEAD_OBJECT.to_string();
            } else {
                self.cmd = GET_OBJECT.to_string();
            }
            self.object = elems[1..].join("/");
        }
    }

    pub fn parse_put(&mut self) {
        if self.path == "/" {
            self.cmd = PUT_BUCKET.to_string();
            return;
        }

        let elems: Vec<&str> = self.path[1..].split('/').collect();
        self.bucket = elems[0].to_string();

        if elems.len() < 2 || (elems.len() == 2 && elems[1].is_empty()) {
            if self.query.contains_key("acl") {
                self.cmd = PUT_BUCKET_ACL.to_string();
            } else if self.query.contains_key("logging") {
                self.cmd = PUT_BUCKET_LOGGING.to_string();
            } else if self.query.contains_key("website") {
                self.cmd = PUT_BUCKET_WEBSITE.to_string();
            } else if self.query.contains_key("referer") {
                self.cmd = PUT_BUCKET_REFERER.to_string();
            } else if self.query.contains_key("lifecycle") {
                self.cmd = PUT_BUCKET_LIFECYCLE.to_string();
            } else {
                self.cmd = PUT_BUCKET.to_string();
            }
        } else if self.query.contains_key("acl") {
            self.cmd = PUT_BUCKET_ACL.to_string();
        } else if self.query.contains_key("symlink") {
            self.cmd = PUT_SYMLINK.to_string();
        } else if self.query.contains_key("partNumber") {
            self.cmd = PUT_UPLOAD_PART.to_string();
        } else {
            self.cmd = PUT_OBJECT.to_string();
        }
        self.object = elems[1..].join("/");

        let copy_source = self.headers.get("x-oss-copy-source").cloned().unwrap_or_default();
        if !copy_source.is_empty() {
            let src_path = copy_source.trim_start_matches('/');
            let src_elems: Vec<&str> = src_path.split('/').collect();
            if !src_elems.is_empty() {
                self.src_bucket = src_elems[0].to_string();
                self.src_object = src_elems[1..].join("/");
                // 如果同时有 partNumber+uploadId，则是 UploadPartCopy
                if self.query.contains_key("partNumber") && self.query.contains_key("uploadId") {
                    self.cmd = PUT_UPLOAD_PART_COPY.to_string();
                } else {
                    self.cmd = PUT_COPY_OBJECT.to_string();
                }
            }
        }
    }

    pub fn parse_delete(&mut self) {
        if self.path == "/" {
            self.cmd = REQUEST_ERROR.to_string();
            return;
        }

        let elems: Vec<&str> = self.path[1..].split('/').collect();
        self.bucket = elems[0].to_string();

        if elems.len() == 1 {
            if self.query.contains_key("logging") {
                self.cmd = DELETE_BUCKET_LOGGING.to_string();
            } else if self.query.contains_key("website") {
                self.cmd = DELETE_BUCKET_WEBSITE.to_string();
            } else if self.query.contains_key("referer") {
                self.cmd = DELETE_BUCKET_REFERER.to_string();
            } else if self.query.contains_key("lifecycle") {
                self.cmd = DELETE_BUCKET_LIFECYCLE.to_string();
            } else {
                self.cmd = DELETE_BUCKET.to_string();
            }
        } else {
            if self.query.contains_key("uploadId") {
                self.cmd = DELETE_ABORT_MULTIPART_UPLOAD.to_string();
            } else {
                self.cmd = DELETE_OBJECT.to_string();
            }
            self.object = elems[1..].join("/");
        }
    }

    pub fn parse_post(&mut self) {
        let elems: Vec<&str> = self.path[1..].split('/').collect();
        self.bucket = elems[0].to_string();
        self.object = elems[1..].join("/");

        if self.query.contains_key("uploads") {
            self.cmd = POST_INIT_MULTIPART_UPLOAD.to_string();
        } else if self.query.contains_key("append") {
            self.cmd = POST_APPEND_OBJECT.to_string();
        } else if self.query.contains_key("uploadId") {
            self.cmd = POST_COMPLETE_MULTIPART_UPLOAD.to_string();
        } else if self.query.contains_key("delete") {
            self.cmd = DELETE_MULTIPLE_OBJECTS.to_string();
        } else if self.query.contains_key("restore") {
            self.cmd = POST_RESTORE_OBJECT.to_string();
        } else {
            let content_type = self.headers.get("content-type").cloned().unwrap_or_default();
            if content_type.contains("multipart/form-data") {
                self.cmd = POST_OBJECT.to_string();
            } else {
                self.cmd = POST_ELSE.to_string();
            }
        }
    }
}
