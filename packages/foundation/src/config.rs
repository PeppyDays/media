use envconfig::Envconfig;

use crate::common::tracing::LogFormat;
use crate::common::tracing::LogLevel;

#[derive(Debug, Envconfig)]
pub struct Config {
    #[envconfig(nested)]
    pub database: DatabaseConfig,

    #[envconfig(nested)]
    pub aws: AwsConfig,

    #[envconfig(nested)]
    pub storage: StorageConfig,

    #[envconfig(nested)]
    pub cdn: CdnConfig,

    #[envconfig(nested)]
    pub tracing: TracingConfig,
}

impl Config {
    pub fn load() -> Self {
        Self::init_from_env().expect("failed to load configuration from environment")
    }
}

#[derive(Debug, Envconfig)]
pub struct DatabaseConfig {
    #[envconfig(from = "FOUNDATION_DATABASE_HOST")]
    pub host: String,
    #[envconfig(from = "FOUNDATION_DATABASE_PORT", default = "5432")]
    pub port: u16,
    #[envconfig(from = "FOUNDATION_DATABASE_USERNAME")]
    pub username: String,
    #[envconfig(from = "FOUNDATION_DATABASE_PASSWORD")]
    pub password: String,
    #[envconfig(from = "FOUNDATION_DATABASE_NAME")]
    pub name: String,
}

impl DatabaseConfig {
    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.name
        )
    }
}

#[derive(Debug, Envconfig)]
pub struct AwsConfig {
    #[envconfig(from = "FOUNDATION_AWS_REGION", default = "ap-northeast-2")]
    pub region: String,
}

#[derive(Debug, Envconfig)]
pub struct StorageConfig {
    #[envconfig(from = "FOUNDATION_STORAGE_BUCKET_NAME")]
    pub bucket_name: String,
    #[envconfig(from = "FOUNDATION_STORAGE_UPLOAD_URL_EXPIRY_SECS", default = "300")]
    pub upload_url_expiry_secs: u64,
    #[envconfig(
        from = "FOUNDATION_STORAGE_MAX_UPLOAD_SIZE_BYTES",
        default = "10485760"
    )]
    pub max_upload_size_bytes: u64,
}

#[derive(Debug, Envconfig)]
pub struct CdnConfig {
    #[envconfig(from = "FOUNDATION_CDN_DOMAIN")]
    pub domain: String,
    #[envconfig(from = "FOUNDATION_CDN_KEY_PAIR_ID")]
    pub key_pair_id: String,
    #[envconfig(from = "FOUNDATION_CDN_PRIVATE_KEY_PEM")]
    pub private_key_pem: String,
    #[envconfig(from = "FOUNDATION_CDN_SIGNED_URL_EXPIRY_SECS", default = "600")]
    pub signed_url_expiry_secs: u64,
}

#[derive(Debug, Envconfig)]
pub struct TracingConfig {
    #[envconfig(from = "FOUNDATION_TRACING_LOG_LEVEL", default = "info")]
    pub log_level: LogLevel,
    #[envconfig(from = "FOUNDATION_TRACING_LOG_FORMAT", default = "pretty")]
    pub log_format: LogFormat,
}
