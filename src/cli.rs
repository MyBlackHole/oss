use clap::Parser;

use crate::config::Config;

#[derive(Parser, Debug)]
#[command(name = "oss-emulator", version = "1.0.0", about = "轻量级的阿里云 OSS 服务模拟器")]
pub struct Cli {
    #[arg(long, default_value = "store")]
    pub store: String,

    #[arg(long, default_value = "/tmp/oss-emulator")]
    pub log_path: String,

    #[arg(long, default_value_t = 80)]
    pub port: u16,

    /// TOML 配置文件路径，用于覆盖默认常量
    #[arg(long)]
    pub config: Option<String>,
}

impl Cli {
    pub fn to_config(&self) -> Config {
        let mut config = Config::new(Some(self.store.clone()), Some(self.log_path.clone()), self.port);
        if let Some(ref config_path) = self.config {
            config = Config::from_toml(Some(config_path));
            // 保留 CLI 参数覆盖
            config.store_root = self.store.clone().into();
            config.log_dir = self.log_path.clone().into();
            config.port = self.port;
        }
        config
    }
}
