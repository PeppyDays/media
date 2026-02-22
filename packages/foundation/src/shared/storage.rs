use aws_config::SdkConfig;
use aws_sdk_s3::Client as S3Client;

pub fn create_s3_client(sdk_config: &SdkConfig) -> S3Client {
    S3Client::new(sdk_config)
}
