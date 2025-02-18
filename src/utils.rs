use actix_web::{HttpResponse, HttpResponseBuilder, body::BoxBody};
use sha3::{Digest as _, Sha3_256};

/// Hashes a string using the Sha3_256 algorithm.
pub fn hash_str(str: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(str);

    let hash = hasher.finalize();
    let mut hash_buf = vec![0; base16ct::encoded_len(&hash)];

    base16ct::lower::encode_str(&hash, &mut hash_buf).expect("hex encoding failed");

    // SAFETY: If encode_str succeeds, it is guaranteed the hash_buf contents are valid UTF-8.
    unsafe { String::from_utf8_unchecked(hash_buf) }
}

pub trait HttpResponseBuilderExt {
    /// Respond with an XML body.
    fn xml<T: serde::Serialize>(self, value: &T) -> crate::Result<HttpResponse<BoxBody>>;
}

impl HttpResponseBuilderExt for HttpResponseBuilder {
    fn xml<T: serde::Serialize>(mut self, value: &T) -> crate::Result<HttpResponse<BoxBody>> {
        Ok(self
            .insert_header(actix_web::http::header::ContentType(mime::TEXT_XML))
            .body(quick_xml::se::to_string(value)?))
    }
}
