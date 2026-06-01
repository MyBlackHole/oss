use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, Method};
use axum::response::Response;
use axum::routing::{any, delete, get, head, post, put};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::Config;
use crate::handlers;
use crate::request::OssRequest;
use crate::response::OssResponse;
use crate::storage::FsStorage;
use crate::storage::Storage;

pub async fn start_server(config: Config) {
    let storage: Arc<dyn Storage> = Arc::new(FsStorage::new(config.store_root.clone()));
    let state = AppState { storage };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::HEAD, Method::OPTIONS])
        .allow_headers(Any)
        .expose_headers([axum::http::header::ETAG]);

    let app = Router::new()
        .route("/", get(root_get_handler))
        .route("/", put(root_put_handler))
        .route("/", delete(root_delete_handler))
        .route("/", post(root_post_handler))
        .route("/", head(root_head_handler))
        .route("/*path", any(catch_all_handler))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    info!("OSS Emulator starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone)]
struct AppState {
    storage: Arc<dyn Storage>,
}

async fn root_get_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    handle_request("GET", "/", &params, &headers, &Bytes::new(), &state).await
}

async fn root_put_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    handle_request("PUT", "/", &params, &headers, &body, &state).await
}

async fn root_delete_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    handle_request("DELETE", "/", &params, &headers, &Bytes::new(), &state).await
}

async fn root_post_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    handle_request("POST", "/", &params, &headers, &body, &state).await
}

async fn root_head_handler(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    handle_request("HEAD", "/", &params, &headers, &Bytes::new(), &state).await
}

async fn catch_all_handler(
    State(state): State<AppState>,
    method: Method,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let path = format!("/{path}");
    handle_request(method.as_str(), &path, &params, &headers, &body, &state).await
}

async fn handle_request(
    method: &str,
    path: &str,
    query: &HashMap<String, String>,
    headers: &HeaderMap,
    body: &Bytes,
    state: &AppState,
) -> Response {
    if method == "OPTIONS" {
        return OssResponse::options();
    }

    let headers_map: HashMap<String, String> = headers
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_lowercase().to_string(),
                v.to_str().unwrap_or_default().to_string(),
            )
        })
        .collect();

    let mut oss_req = OssRequest::new(method, path, query, &headers_map);
    oss_req.parse();

    match oss_req.cmd.as_str() {
        crate::request::LIST_BUCKETS => handlers::list_buckets(State(state.storage.clone())).await,
        crate::request::PUT_BUCKET => {
            let bucket = oss_req.bucket.clone();
            handlers::create_bucket(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
                headers.clone(),
            )
            .await
        }
        crate::request::GET_BUCKET => {
            let bucket = oss_req.bucket.clone();
            handlers::get_bucket(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
                Query(query.clone()),
            )
            .await
        }
        crate::request::DELETE_BUCKET => {
            let bucket = oss_req.bucket.clone();
            handlers::delete_bucket(State(state.storage.clone()), axum::extract::Path(bucket)).await
        }
        crate::request::GET_BUCKET_ACL => {
            let bucket = oss_req.bucket.clone();
            handlers::get_bucket_acl(State(state.storage.clone()), axum::extract::Path(bucket)).await
        }
        crate::request::PUT_BUCKET_ACL => {
            let bucket = oss_req.bucket.clone();
            handlers::put_bucket_acl(State(state.storage.clone()), axum::extract::Path(bucket)).await
        }
        crate::request::GET_BUCKET_INFO => {
            let bucket = oss_req.bucket.clone();
            handlers::get_bucket_info(State(state.storage.clone()), axum::extract::Path(bucket)).await
        }
        crate::request::GET_BUCKET_LOCATION => {
            let bucket = oss_req.bucket.clone();
            handlers::get_bucket_location(State(state.storage.clone()), axum::extract::Path(bucket)).await
        }

        crate::request::PUT_OBJECT => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::put_object(
                State(state.storage.clone()),
                Path((bucket, object)),
                headers.clone(),
                body.clone(),
            )
            .await
        }
        crate::request::PUT_UPLOAD_PART => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::upload_part(
                State(state.storage.clone()),
                Path((bucket, object)),
                Query(query.clone()),
                headers.clone(),
                body.clone(),
            )
            .await
        }
        crate::request::GET_OBJECT => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::get_object(
                State(state.storage.clone()),
                Path((bucket, object)),
                headers.clone(),
            )
            .await
        }
        crate::request::HEAD_OBJECT => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::head_object(State(state.storage.clone()), Path((bucket, object))).await
        }
        crate::request::DELETE_OBJECT => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::delete_object(State(state.storage.clone()), Path((bucket, object))).await
        }
        crate::request::GET_OBJECT_META => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::get_object_meta_handler(
                State(state.storage.clone()),
                Path((bucket, object)),
            )
            .await
        }
        crate::request::GET_OBJECT_ACL => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::get_object_acl_handler(
                State(state.storage.clone()),
                Path((bucket, object)),
            )
            .await
        }
        crate::request::PUT_OBJECT_ACL => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::put_object_acl_handler(
                State(state.storage.clone()),
                Path((bucket, object)),
            )
            .await
        }
        crate::request::POST_APPEND_OBJECT => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::append_object(
                State(state.storage.clone()),
                Path((bucket, object)),
                headers.clone(),
                body.clone(),
            )
            .await
        }
        crate::request::PUT_COPY_OBJECT => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::copy_object(
                State(state.storage.clone()),
                Path((bucket, object)),
                headers.clone(),
            )
            .await
        }

        crate::request::POST_INIT_MULTIPART_UPLOAD => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::initiate_multipart_upload(
                State(state.storage.clone()),
                Path((bucket, object)),
            )
            .await
        }
        crate::request::POST_COMPLETE_MULTIPART_UPLOAD => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::complete_multipart_upload(
                State(state.storage.clone()),
                Path((bucket, object)),
                body.clone(),
            )
            .await
        }
        crate::request::DELETE_ABORT_MULTIPART_UPLOAD => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::abort_multipart_upload(
                State(state.storage.clone()),
                Path((bucket, object)),
            )
            .await
        }

        crate::request::DELETE_MULTIPLE_OBJECTS => {
            let bucket = oss_req.bucket.clone();
            handlers::delete_multiple_objects(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
                body.clone(),
            )
            .await
        }
        crate::request::GET_LIST_PARTS => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::list_parts(
                State(state.storage.clone()),
                Path((bucket, object)),
            )
            .await
        }
        crate::request::GET_LIST_MULTIPART_UPLOADS => {
            let bucket = oss_req.bucket.clone();
            handlers::list_multipart_uploads(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
            )
            .await
        }

        crate::request::PUT_BUCKET_LOGGING => {
            let bucket = oss_req.bucket.clone();
            handlers::put_bucket_logging(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
                headers.clone(),
                body.clone(),
            )
            .await
        }
        crate::request::GET_BUCKET_LOGGING => {
            let bucket = oss_req.bucket.clone();
            handlers::get_bucket_logging(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
            )
            .await
        }
        crate::request::DELETE_BUCKET_LOGGING => {
            let bucket = oss_req.bucket.clone();
            handlers::delete_bucket_logging(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
            )
            .await
        }
        crate::request::PUT_BUCKET_WEBSITE => {
            let bucket = oss_req.bucket.clone();
            handlers::put_bucket_website(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
                headers.clone(),
                body.clone(),
            )
            .await
        }
        crate::request::GET_BUCKET_WEBSITE => {
            let bucket = oss_req.bucket.clone();
            handlers::get_bucket_website(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
            )
            .await
        }
        crate::request::DELETE_BUCKET_WEBSITE => {
            let bucket = oss_req.bucket.clone();
            handlers::delete_bucket_website(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
            )
            .await
        }
        crate::request::PUT_BUCKET_REFERER => {
            let bucket = oss_req.bucket.clone();
            handlers::put_bucket_referer(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
                headers.clone(),
                body.clone(),
            )
            .await
        }
        crate::request::GET_BUCKET_REFERER => {
            let bucket = oss_req.bucket.clone();
            handlers::get_bucket_referer(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
            )
            .await
        }
        crate::request::DELETE_BUCKET_REFERER => {
            let bucket = oss_req.bucket.clone();
            handlers::delete_bucket_referer(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
            )
            .await
        }
        crate::request::PUT_BUCKET_LIFECYCLE => {
            let bucket = oss_req.bucket.clone();
            handlers::put_bucket_lifecycle(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
                headers.clone(),
                body.clone(),
            )
            .await
        }
        crate::request::GET_BUCKET_LIFECYCLE => {
            let bucket = oss_req.bucket.clone();
            handlers::get_bucket_lifecycle(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
            )
            .await
        }
        crate::request::DELETE_BUCKET_LIFECYCLE => {
            let bucket = oss_req.bucket.clone();
            handlers::delete_bucket_lifecycle(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
            )
            .await
        }
        crate::request::PUT_SYMLINK => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::put_symlink(
                State(state.storage.clone()),
                Path((bucket, object)),
                headers.clone(),
                body.clone(),
            )
            .await
        }
        crate::request::GET_SYMLINK => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::get_symlink(State(state.storage.clone()), Path((bucket, object))).await
        }
        crate::request::PUT_UPLOAD_PART_COPY => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::upload_part_copy(
                State(state.storage.clone()),
                Path((bucket, object)),
                Query(query.clone()),
                headers.clone(),
            )
            .await
        }
        crate::request::POST_RESTORE_OBJECT => {
            let bucket = oss_req.bucket.clone();
            let object = oss_req.object.clone();
            handlers::restore_object(State(state.storage.clone()), Path((bucket, object))).await
        }
        crate::request::POST_OBJECT => {
            let bucket = oss_req.bucket.clone();
            handlers::post_object(
                State(state.storage.clone()),
                axum::extract::Path(bucket),
                body.clone(),
            )
            .await
        }

        _ => OssResponse::error(crate::error::BAD_REQUEST),
    }
}
