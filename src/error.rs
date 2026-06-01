use std::fmt;

#[derive(Debug, Clone)]
pub struct ErrorCode {
    pub error_code: &'static str,
    pub status_code: u16,
    pub message: &'static str,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.status_code, self.error_code, self.message)
    }
}

pub const NO_MODIFIED: ErrorCode = ErrorCode {
    error_code: "NoModified",
    status_code: 304,
    message: "The object has not been modified.",
};

pub const BAD_REQUEST: ErrorCode = ErrorCode {
    error_code: "BadRequest",
    status_code: 400,
    message: "The server cannot understand the request.",
};

pub const TOO_MANY_BUCKETS: ErrorCode = ErrorCode {
    error_code: "TooManyBuckets",
    status_code: 400,
    message: "Bucket number exceeds the limit.",
};

pub const INVALID_BUCKET_NAME: ErrorCode = ErrorCode {
    error_code: "InvalidBucketName",
    status_code: 400,
    message: "The bucket name is invalid.",
};

pub const INVALID_OBJECT_NAME: ErrorCode = ErrorCode {
    error_code: "InvalidObjectName",
    status_code: 400,
    message: "The object name is invalid.",
};

pub const INVALID_ARGUMENT: ErrorCode = ErrorCode {
    error_code: "InvalidArgument",
    status_code: 400,
    message: "The file size should be less than 5G.",
};

pub const FILE_PART_NO_EXIST: ErrorCode = ErrorCode {
    error_code: "FilePartNotExist",
    status_code: 400,
    message: "The file part does not exist.",
};

pub const ACCESS_DENIED: ErrorCode = ErrorCode {
    error_code: "AccessDenied",
    status_code: 403,
    message: "The access is forbidden.",
};

pub const NO_SUCH_BUCKET: ErrorCode = ErrorCode {
    error_code: "NoSuchBucket",
    status_code: 404,
    message: "The bucket does not exist.",
};

pub const NO_SUCH_KEY: ErrorCode = ErrorCode {
    error_code: "NoSuchKey",
    status_code: 404,
    message: "The specified object does not exist.",
};

pub const NOT_FOUND: ErrorCode = ErrorCode {
    error_code: "NotFound",
    status_code: 404,
    message: "The file has not been found.",
};

pub const BUCKET_ALREADY_EXISTS: ErrorCode = ErrorCode {
    error_code: "BucketAlreadyExists",
    status_code: 409,
    message: "The bucket already exists.",
};

pub const BUCKET_NOT_EMPTY: ErrorCode = ErrorCode {
    error_code: "BucketNotEmpty",
    status_code: 409,
    message: "The bucket is not empty.",
};

pub const OBJECT_NOT_APPENDABLE: ErrorCode = ErrorCode {
    error_code: "ObjectNotAppendable",
    status_code: 409,
    message: "The object is not appendable.",
};

pub const MISSING_CONTENT_LENGTH: ErrorCode = ErrorCode {
    error_code: "MissingContentLength",
    status_code: 411,
    message: "No Content-Length in request header.",
};

pub const INTERNAL_ERROR: ErrorCode = ErrorCode {
    error_code: "InternalError",
    status_code: 500,
    message: "An internal error occurs inside OSS.",
};

pub const NOT_IMPLEMENTED: ErrorCode = ErrorCode {
    error_code: "NotImplemented",
    status_code: 501,
    message: "The function is not supported yet.",
};

pub const NO_SUCH_CONFIGURATION: ErrorCode = ErrorCode {
    error_code: "NoSuchConfiguration",
    status_code: 404,
    message: "The configuration does not exist.",
};
