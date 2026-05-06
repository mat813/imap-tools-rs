/// SASL XOAUTH2 authenticator for use with [`async_imap::Client::authenticate`].
///
/// Encodes the bearer-token SASL string; async-imap handles the base64 wrapping.
pub struct XOAuth2Auth {
    /// IMAP username (typically an email address).
    pub user: String,
    /// `OAuth2` access token.
    pub token: String,
}

impl async_imap::Authenticator for XOAuth2Auth {
    type Response = Vec<u8>;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, _challenge), ret)
    )]
    fn process(&mut self, _challenge: &[u8]) -> Self::Response {
        format!("user={}\x01auth=Bearer {}\x01\x01", self.user, self.token).into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use async_imap::Authenticator as _;

    use super::XOAuth2Auth;

    #[test]
    fn xoauth2_process_format() {
        let mut auth = XOAuth2Auth {
            user: "alice@example.com".to_owned(),
            token: "ya29.TOKEN".to_owned(),
        };
        let response = auth.process(&[]);
        assert_eq!(
            response, b"user=alice@example.com\x01auth=Bearer ya29.TOKEN\x01\x01",
            "SASL XOAUTH2 format must be exact"
        );
    }
}
