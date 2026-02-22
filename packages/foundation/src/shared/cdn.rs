use std::borrow::Cow;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use cloudfront_sign::EncodingError;
use cloudfront_sign::SignedOptions;
use cloudfront_sign::get_signed_url;

#[derive(Clone)]
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

    pub fn generate_signed_url(
        &self,
        object_key: &str,
        expiry_secs: u64,
    ) -> Result<String, EncodingError> {
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

        get_signed_url(&url, &options)
    }
}
