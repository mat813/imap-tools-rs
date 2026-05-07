use std::fmt::Write as _;

use hmac::{Hmac, KeyInit as _, Mac as _};
use md5::Md5;

/// SASL CRAM-MD5 authenticator for use with [`async_imap::Client::authenticate`].
///
/// Computes HMAC-MD5(password, challenge) and returns `"username hex_digest"`.
pub struct CramMd5Auth {
    /// IMAP username.
    pub user: String,
    /// Password used as the HMAC key.
    pub password: String,
}

impl async_imap::Authenticator for CramMd5Auth {
    type Response = Vec<u8>;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, challenge), ret)
    )]
    fn process(&mut self, challenge: &[u8]) -> Self::Response {
        #[expect(clippy::expect_used, reason = "HMAC accepts keys of any length")]
        let mut mac = Hmac::<Md5>::new_from_slice(self.password.as_bytes())
            .expect("HMAC key length is never invalid");
        mac.update(challenge);
        let digest = mac.finalize().into_bytes();

        let mut hex = String::with_capacity(digest.len() * 2);
        for b in &digest {
            let _ = write!(hex, "{b:02x}");
        }
        format!("{} {}", self.user, hex).into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use async_imap::Authenticator as _;

    use super::CramMd5Auth;

    #[test]
    fn cram_md5_process_format() {
        // RFC 2195 Appendix B test vector:
        // challenge: <1896.697170952@postoffice.reston.mci.net>
        // password:  tanstaaftanstaaf
        // expected:  tim b913a602c7eda7a495b4e6e7334d3890
        let mut auth = CramMd5Auth {
            user: "tim".to_owned(),
            password: "tanstaaftanstaaf".to_owned(),
        };
        let challenge = b"<1896.697170952@postoffice.reston.mci.net>";
        let response = auth.process(challenge);
        assert_eq!(
            response, b"tim b913a602c7eda7a495b4e6e7334d3890",
            "CRAM-MD5 HMAC-MD5 hex digest must match RFC 2195 Appendix B test vector"
        );
    }
}
