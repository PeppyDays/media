use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub aws_region: String,
    pub s3_input_bucket: String,
    pub s3_upload_url_expiry_secs: u64,
    pub s3_max_upload_size_bytes: u64,
    pub sqs_media_queue_url: String,
    pub sqs_media_dlq_url: String,
    pub cloudfront_domain: String,
    pub cloudfront_key_pair_id: String,
    pub cloudfront_private_key_pem: String,
    pub cloudfront_signed_url_expiry_secs: u64,
}

impl Config {
    pub fn load() -> Self {
        Self {
            database_url: require_env("DATABASE_URL"),
            aws_region: require_env("AWS_REGION"),
            s3_input_bucket: require_env("S3_INPUT_BUCKET"),
            s3_upload_url_expiry_secs: optional_env_u64("S3_UPLOAD_URL_EXPIRY_SECS", 300),
            s3_max_upload_size_bytes: optional_env_u64("S3_MAX_UPLOAD_SIZE_BYTES", 10_485_760),
            sqs_media_queue_url: require_env("SQS_MEDIA_QUEUE_URL"),
            sqs_media_dlq_url: require_env("SQS_MEDIA_DLQ_URL"),
            cloudfront_domain: require_env("CLOUDFRONT_DOMAIN"),
            cloudfront_key_pair_id: require_env("CLOUDFRONT_KEY_PAIR_ID"),
            cloudfront_private_key_pem: require_env("CLOUDFRONT_PRIVATE_KEY_PEM"),
            cloudfront_signed_url_expiry_secs: optional_env_u64(
                "CLOUDFRONT_SIGNED_URL_EXPIRY_SECS",
                600,
            ),
        }
    }
}

fn require_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("missing required environment variable: {key}"))
}

fn optional_env_u64(key: &str, default: u64) -> u64 {
    match env::var(key) {
        Ok(val) => val
            .parse()
            .unwrap_or_else(|_| panic!("environment variable {key} must be a valid integer")),
        Err(_) => default,
    }
}
