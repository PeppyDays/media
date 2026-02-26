use std::borrow::Cow;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use cloudfront_sign::SignedOptions;
use cloudfront_sign::get_signed_url;

#[derive(Debug, thiserror::Error)]
#[error("CDN signing failed: {0}")]
pub struct CdnSigningError(String);

impl From<cloudfront_sign::EncodingError> for CdnSigningError {
    fn from(e: cloudfront_sign::EncodingError) -> Self {
        Self(e.to_string())
    }
}

pub trait CdnSigner: Send + Sync {
    fn generate_signed_url(
        &self,
        object_key: &str,
        expiry_secs: u64,
    ) -> Result<String, CdnSigningError>;
}

pub struct CloudFrontSigner {
    domain: String,
    key_pair_id: String,
    private_key_pem: String,
}

impl CloudFrontSigner {
    pub fn new(domain: String, key_pair_id: String, private_key_pem: String) -> Self {
        Self {
            domain,
            key_pair_id,
            private_key_pem,
        }
    }
}

impl CdnSigner for CloudFrontSigner {
    fn generate_signed_url(
        &self,
        object_key: &str,
        expiry_secs: u64,
    ) -> Result<String, CdnSigningError> {
        let url = format!("https://{}/{}", self.domain, object_key);
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is always after the Unix epoch")
            .as_secs()
            + expiry_secs;

        let options = SignedOptions {
            key_pair_id: Cow::Borrowed(&self.key_pair_id),
            private_key: Cow::Borrowed(&self.private_key_pem),
            date_less_than: expires_at,
            ..Default::default()
        };

        Ok(get_signed_url(&url, &options)?)
    }
}
