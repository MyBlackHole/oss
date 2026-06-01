use std::path::PathBuf;

pub const VERSION: &str = "1.0.0";
pub const VERSION_STRING: &str = "Emulator V1.0.0";

pub const LOG_FILE_NAME: &str = "s3-oss.log";
pub const LOG_DIR: &str = "/opt/aio/logs/tools/oss-emulator";
pub const STORE_ROOT_DIR: &str = "store";

pub const BUCKET_METADATA: &str = ".meta";
pub const OBJECT_METADATA: &str = ".meta_obj";
pub const OBJECT_CONTENT_PREFIX: &str = ".part_";
pub const OBJECT_CONTENT: &str = ".data";
pub const OBJECT_CONTENT_TWO: &str = ".data2";
pub const BUCKET_CONFIG: &str = ".config.toml";

pub const DEFAULT_PORT: u16 = 80;

pub const MAX_BUCKET_NUM: usize = 30;
pub const MAX_OBJECT_FILE_SIZE: i64 = 5 * 1024 * 1024 * 1024;
pub const STREAM_CHUNK_SIZE: usize = 32 * 1024;

pub const HOST: &str = "localhost";
pub const HOSTNAMES: &[&str] = &["localhost", "oss.aliyun.com", "oss.localhost"];
pub const ALIYUN_OSS_SERVER: &str = "AliyunOSS";
pub const XMLNS: &str = "http://doc.oss-cn-hangzhou.aliyuncs.com";
pub const OWNER_ID: &str = "00220120222";
pub const OWNER_DISPLAY_NAME: &str = "1390402650033798";

pub const OSS_ACL: &[&str] = &["public-read-write", "public-read", "private", "default"];
pub const BUCKET_ACL_LIST: &[&str] = &["public-read-write", "public-read", "private"];
pub const OBJECT_ACL_LIST: &[&str] = &["public-read-write", "public-read", "private", "default"];

/// TOML 配置文件的根结构，用于通过 `--config` 加载覆盖默认常量
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct AppConfig {
    pub storage: Option<StorageConfig>,
    pub defaults: Option<DefaultsConfig>,
    pub limits: Option<LimitsConfig>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct StorageConfig {
    pub bucket_metadata: Option<String>,
    pub object_metadata: Option<String>,
    pub object_content_prefix: Option<String>,
    pub object_content: Option<String>,
    pub object_content_two: Option<String>,
    pub bucket_config: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DefaultsConfig {
    pub hostnames: Option<Vec<String>>,
    pub owner_id: Option<String>,
    pub owner_display_name: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct LimitsConfig {
    pub max_bucket_num: Option<usize>,
    pub max_object_file_size: Option<i64>,
    pub stream_chunk_size: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub store_root: PathBuf,
    pub log_dir: PathBuf,
    pub port: u16,
    pub host: String,
    pub hostnames: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            store_root: PathBuf::from(STORE_ROOT_DIR),
            log_dir: PathBuf::from(LOG_DIR),
            port: DEFAULT_PORT,
            host: HOST.to_string(),
            hostnames: HOSTNAMES.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Config {
    pub fn new(store: Option<String>, log_path: Option<String>, port: u16) -> Self {
        Config {
            store_root: store.map(PathBuf::from).unwrap_or_else(|| PathBuf::from(STORE_ROOT_DIR)),
            log_dir: log_path.map(PathBuf::from).unwrap_or_else(|| PathBuf::from(LOG_DIR)),
            port,
            ..Default::default()
        }
    }

    /// 从 TOML 配置文件加载并合并配置（如果指定了路径）
    pub fn from_toml(config_path: Option<&str>) -> Self {
        let mut config = Config::default();
        if let Some(path) = config_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(app_config) = toml::from_str::<AppConfig>(&content) {
                    config.apply_toml(&app_config);
                }
            }
        }
        config
    }
}

/// 将 TOML 配置的覆盖值应用到运行时 Config。
/// 注意: BUCKET_METADATA 等编译时常量当前无法通过 TOML 覆盖，
/// 需要将常量改为 Config 字段或全局变量方可实现。
impl Config {
    pub fn apply_toml(&mut self, app_config: &AppConfig) {
        if let Some(storage) = &app_config.storage {
            tracing::info!(
                "TOML storage config loaded (runtime override requires refactoring consts): {:?}",
                storage
            );
        }
        if let Some(defaults) = &app_config.defaults {
            if let Some(hostnames) = &defaults.hostnames {
                self.hostnames = hostnames.clone();
            }
        }
        if let Some(limits) = &app_config.limits {
            tracing::info!(
                "TOML limits config loaded (runtime override requires refactoring consts): {:?}",
                limits
            );
        }
    }
}
