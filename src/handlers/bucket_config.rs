use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;

use crate::models::{
    BucketLoggingStatus, LifecycleConfig, LifecycleRule, LoggingConfig,
    RefererConfig, WebsiteConfig,
};
use crate::response::OssResponse;
use crate::storage::Storage;

// ============================================================
// BucketLogging — PUT/GET/DELETE
// ============================================================

pub async fn put_bucket_logging(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
    _headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Some(resp) = OssResponse::check_bucket_exists(storage.as_ref(), &bucket) {
        return resp;
    }

    let body_str = String::from_utf8_lossy(&body);
    let status: BucketLoggingStatus = match quick_xml::de::from_str(&body_str) {
        Ok(s) => s,
        Err(_) => return OssResponse::error(crate::error::BAD_REQUEST),
    };

    let mut config = storage.load_bucket_config(&bucket).unwrap_or_default().unwrap_or_default();
    config.logging = status.logging_enabled.map(|le| LoggingConfig {
        target_bucket: le.target_bucket,
        target_prefix: le.target_prefix,
    });

    if storage.save_bucket_config(&bucket, &config).is_err() {
        return OssResponse::error(crate::error::INTERNAL_ERROR);
    }

    OssResponse::ok_bucket_config()
}

pub async fn get_bucket_logging(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
) -> Response {
    if let Some(resp) = OssResponse::check_bucket_exists(storage.as_ref(), &bucket) {
        return resp;
    }

    let config = match storage.load_bucket_config(&bucket) {
        Ok(Some(c)) => c,
        _ => return OssResponse::no_such_configuration(),
    };

    let logging = match config.logging {
        Some(l) => l,
        None => return OssResponse::no_such_configuration(),
    };

    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<BucketLoggingStatus>
  <LoggingEnabled>
    <TargetBucket>{}</TargetBucket>
    <TargetPrefix>{}</TargetPrefix>
  </LoggingEnabled>
</BucketLoggingStatus>"#,
        logging.target_bucket, logging.target_prefix
    );

    Response::builder()
        .status(200)
        .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
        .header("Content-Type", "application/xml")
        .body(axum::body::Body::from(body))
        .unwrap()
}

pub async fn delete_bucket_logging(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
) -> Response {
    if let Some(resp) = OssResponse::check_bucket_exists(storage.as_ref(), &bucket) {
        return resp;
    }

    let mut config = storage.load_bucket_config(&bucket).unwrap_or_default().unwrap_or_default();
    config.logging = None;

    let _ = storage.save_bucket_config(&bucket, &config);

    OssResponse::ok_no_content()
}

// ============================================================
// BucketWebsite — PUT/GET/DELETE
// ============================================================

pub async fn put_bucket_website(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
    _headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Some(resp) = OssResponse::check_bucket_exists(storage.as_ref(), &bucket) {
        return resp;
    }

    let body_str = String::from_utf8_lossy(&body);
    let web_config: crate::models::WebsiteConfiguration = match quick_xml::de::from_str(&body_str) {
        Ok(c) => c,
        Err(_) => return OssResponse::error(crate::error::BAD_REQUEST),
    };

    let mut config = storage.load_bucket_config(&bucket).unwrap_or_default().unwrap_or_default();
    config.website = Some(WebsiteConfig {
        index_document: web_config.index_document.suffix,
        error_document: web_config.error_document.key,
    });

    if storage.save_bucket_config(&bucket, &config).is_err() {
        return OssResponse::error(crate::error::INTERNAL_ERROR);
    }

    OssResponse::ok_bucket_config()
}

pub async fn get_bucket_website(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
) -> Response {
    if let Some(resp) = OssResponse::check_bucket_exists(storage.as_ref(), &bucket) {
        return resp;
    }

    let config = match storage.load_bucket_config(&bucket) {
        Ok(Some(c)) => c,
        _ => return OssResponse::no_such_configuration(),
    };

    let website = match config.website {
        Some(w) => w,
        None => return OssResponse::no_such_configuration(),
    };

    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<WebsiteConfiguration>
  <IndexDocument>
    <Suffix>{}</Suffix>
  </IndexDocument>
  <ErrorDocument>
    <Key>{}</Key>
  </ErrorDocument>
</WebsiteConfiguration>"#,
        website.index_document, website.error_document
    );

    Response::builder()
        .status(200)
        .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
        .header("Content-Type", "application/xml")
        .body(axum::body::Body::from(body))
        .unwrap()
}

pub async fn delete_bucket_website(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
) -> Response {
    if let Some(resp) = OssResponse::check_bucket_exists(storage.as_ref(), &bucket) {
        return resp;
    }

    let mut config = storage.load_bucket_config(&bucket).unwrap_or_default().unwrap_or_default();
    config.website = None;

    let _ = storage.save_bucket_config(&bucket, &config);

    OssResponse::ok_no_content()
}

// ============================================================
// BucketReferer — PUT/GET/DELETE
// ============================================================

pub async fn put_bucket_referer(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
    _headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Some(resp) = OssResponse::check_bucket_exists(storage.as_ref(), &bucket) {
        return resp;
    }

    let body_str = String::from_utf8_lossy(&body);
    let referer_config: crate::models::RefererConfiguration = match quick_xml::de::from_str(&body_str) {
        Ok(c) => c,
        Err(_) => return OssResponse::error(crate::error::BAD_REQUEST),
    };

    let mut config = storage.load_bucket_config(&bucket).unwrap_or_default().unwrap_or_default();

    // ossutil "delete" sends PUT with empty RefererList — treat as deletion
    if referer_config.referer_list.referers.is_empty() {
        config.referer = None;
    } else {
        config.referer = Some(RefererConfig {
            allow_empty: referer_config.allow_empty,
            referer_list: referer_config.referer_list.referers,
        });
    }

    if storage.save_bucket_config(&bucket, &config).is_err() {
        return OssResponse::error(crate::error::INTERNAL_ERROR);
    }

    OssResponse::ok_bucket_config()
}

pub async fn get_bucket_referer(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
) -> Response {
    if let Some(resp) = OssResponse::check_bucket_exists(storage.as_ref(), &bucket) {
        return resp;
    }

    let config = match storage.load_bucket_config(&bucket) {
        Ok(Some(c)) => c,
        _ => return OssResponse::no_such_configuration(),
    };

    let referer = match config.referer {
        Some(r) => r,
        None => return OssResponse::no_such_configuration(),
    };

    let referers_xml: String = referer
        .referer_list
        .iter()
        .map(|r| format!("  <Referer>{r}</Referer>"))
        .collect::<Vec<_>>()
        .join("\n");

    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<RefererConfiguration>
  <AllowEmptyReferer>{}</AllowEmptyReferer>
  <RefererList>
{}
  </RefererList>
</RefererConfiguration>"#,
        if referer.allow_empty { "true" } else { "false" },
        referers_xml
    );

    Response::builder()
        .status(200)
        .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
        .header("Content-Type", "application/xml")
        .body(axum::body::Body::from(body))
        .unwrap()
}

pub async fn delete_bucket_referer(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
) -> Response {
    if let Some(resp) = OssResponse::check_bucket_exists(storage.as_ref(), &bucket) {
        return resp;
    }

    let mut config = storage.load_bucket_config(&bucket).unwrap_or_default().unwrap_or_default();
    config.referer = None;

    let _ = storage.save_bucket_config(&bucket, &config);

    OssResponse::ok_no_content()
}

// ============================================================
// BucketLifecycle — PUT/GET/DELETE
// ============================================================

pub async fn put_bucket_lifecycle(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
    _headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Some(resp) = OssResponse::check_bucket_exists(storage.as_ref(), &bucket) {
        return resp;
    }

    let body_str = String::from_utf8_lossy(&body);
    let lc_config: crate::models::LifecycleConfiguration = match quick_xml::de::from_str(&body_str) {
        Ok(c) => c,
        Err(_) => return OssResponse::error(crate::error::BAD_REQUEST),
    };

    let mut config = storage.load_bucket_config(&bucket).unwrap_or_default().unwrap_or_default();
    config.lifecycle = Some(LifecycleConfig {
        rules: lc_config
            .rules
            .into_iter()
            .map(|r| LifecycleRule {
                id: r.id,
                prefix: r.prefix,
                status: r.status,
                expiration_days: r.expiration.map(|e| e.days),
                transition_days: None,
                transition_storage_class: None,
            })
            .collect(),
    });

    if storage.save_bucket_config(&bucket, &config).is_err() {
        return OssResponse::error(crate::error::INTERNAL_ERROR);
    }

    OssResponse::ok_bucket_config()
}

pub async fn get_bucket_lifecycle(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
) -> Response {
    if let Some(resp) = OssResponse::check_bucket_exists(storage.as_ref(), &bucket) {
        return resp;
    }

    let config = match storage.load_bucket_config(&bucket) {
        Ok(Some(c)) => c,
        _ => return OssResponse::no_such_configuration(),
    };

    let lifecycle = match config.lifecycle {
        Some(l) => l,
        None => return OssResponse::no_such_configuration(),
    };

    let rules_xml: String = lifecycle
        .rules
        .iter()
        .map(|r| {
            let exp_xml = r
                .expiration_days
                .map(|d| format!("    <Expiration><Days>{d}</Days></Expiration>"))
                .unwrap_or_default();
            format!(
                r#"  <Rule>
    <ID>{}</ID>
    <Prefix>{}</Prefix>
    <Status>{}</Status>
{}
  </Rule>"#,
                r.id, r.prefix, r.status, exp_xml
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let body = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<LifecycleConfiguration>\n{rules_xml}\n</LifecycleConfiguration>"
    );

    Response::builder()
        .status(200)
        .header("x-oss-request-id", uuid::Uuid::new_v4().to_string())
        .header("Content-Type", "application/xml")
        .body(axum::body::Body::from(body))
        .unwrap()
}

pub async fn delete_bucket_lifecycle(
    State(storage): State<Arc<dyn Storage>>,
    Path(bucket): Path<String>,
) -> Response {
    if let Some(resp) = OssResponse::check_bucket_exists(storage.as_ref(), &bucket) {
        return resp;
    }

    let mut config = storage.load_bucket_config(&bucket).unwrap_or_default().unwrap_or_default();
    config.lifecycle = None;

    let _ = storage.save_bucket_config(&bucket, &config);

    OssResponse::ok_no_content()
}
