use async_imap::Authenticator;

/// SASL PLAIN authenticator for use with [`async_imap::Client::authenticate`].
///
/// Encodes `\0authcid\0passwd`; async-imap handles base64 wrapping.
pub struct PlainAuth {
    /// IMAP username.
    pub user: String,
    /// Password.
    pub password: String,
}

impl Authenticator for PlainAuth {
    type Response = Vec<u8>;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, _challenge), ret)
    )]
    fn process(&mut self, _challenge: &[u8]) -> Self::Response {
        let mut r = Vec::with_capacity(self.user.len() + self.password.len() + 2);
        r.push(0); // empty authzid
        r.extend_from_slice(self.user.as_bytes());
        r.push(0);
        r.extend_from_slice(self.password.as_bytes());
        r
    }
}

#[cfg(test)]
mod tests {
    use async_imap::Authenticator as _;

    use super::PlainAuth;

    #[test]
    fn plain_process_format() {
        let mut auth = PlainAuth {
            user: "alice@example.com".to_owned(),
            password: "secret".to_owned(),
        };
        let response = auth.process(&[]);
        assert_eq!(
            response, b"\x00alice@example.com\x00secret",
            "SASL PLAIN must be NUL-delimited: <empty-authzid> NUL authcid NUL passwd"
        );
    }
}
