use std::str::FromStr;

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display)]
/// Error type for auth-method parsing.
pub struct AuthError(String);
impl std::error::Error for AuthError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
/// Authentication mechanism used when connecting to the IMAP server.
#[derive(Default)]
pub enum AuthMethod {
    /// Standard IMAP LOGIN command (default).
    #[default]
    Login,
    /// SASL XOAUTH2 — bearer-token authentication (Gmail, Office 365).
    #[value(name = "xoauth2")]
    XOAuth2,
}

impl FromStr for AuthMethod {
    type Err = AuthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "login" => Self::Login,
            "xoauth2" => Self::XOAuth2,
            _ => {
                return Err(AuthError(
                    "Invalid auth method, expects: login, xoauth2".to_owned(),
                ));
            },
        })
    }
}

impl<'de> Deserialize<'de> for AuthMethod {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for AuthMethod {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match *self {
            Self::Login => "login",
            Self::XOAuth2 => "xoauth2",
        })
    }
}

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

    fn process(&mut self, _challenge: &[u8]) -> Self::Response {
        format!("user={}\x01auth=Bearer {}\x01\x01", self.user, self.token).into_bytes()
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "test")]
    use async_imap::Authenticator as _;

    use super::{AuthMethod, XOAuth2Auth};

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

    #[test]
    fn auth_method_default_is_login() {
        assert_eq!(AuthMethod::default(), AuthMethod::Login);
    }

    #[test]
    fn auth_method_from_str_login() {
        assert_eq!(
            "login"
                .parse::<AuthMethod>()
                .expect("\"login\" is a valid auth method"),
            AuthMethod::Login
        );
    }

    #[test]
    fn auth_method_from_str_xoauth2() {
        assert_eq!(
            "xoauth2"
                .parse::<AuthMethod>()
                .expect("\"xoauth2\" is a valid auth method"),
            AuthMethod::XOAuth2
        );
    }

    #[test]
    fn auth_method_from_str_case_insensitive() {
        assert_eq!(
            "XOAUTH2"
                .parse::<AuthMethod>()
                .expect("auth method parsing must be case-insensitive"),
            AuthMethod::XOAuth2
        );
    }

    #[test]
    fn auth_method_from_str_invalid() {
        assert!("plain".parse::<AuthMethod>().is_err());
    }
}
