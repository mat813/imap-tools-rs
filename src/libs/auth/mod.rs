mod cram_md5;
mod plain;
mod scram;
mod xoauth2;

use std::str::FromStr;

use derive_more::Display;
use serde::{Deserialize, Serialize};

pub use self::{cram_md5::CramMd5Auth, plain::PlainAuth, scram::ScramAuth, xoauth2::XOAuth2Auth};

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
    /// SASL PLAIN — credentials in cleartext (RFC 4616). Requires TLS.
    #[value(name = "plain")]
    Plain,
    /// SASL CRAM-MD5 — HMAC-MD5 challenge/response (RFC 2195).
    #[value(name = "cram-md5")]
    CramMd5,
    /// SASL SCRAM-SHA-1 — salted challenge/response (RFC 5802).
    #[value(name = "scram-sha-1")]
    ScramSha1,
    /// SASL SCRAM-SHA-256 — salted challenge/response, stronger hash (RFC 7677).
    #[value(name = "scram-sha-256")]
    ScramSha256,
    /// SASL XOAUTH2 — bearer-token authentication (Gmail, Office 365).
    #[value(name = "xoauth2")]
    XOAuth2,
}

impl AuthMethod {
    /// Returns `true` for every mechanism that authenticates with a password.
    ///
    /// Used in config validation to decide whether `password`/`password-command`
    /// must be set.
    pub const fn requires_password(self) -> bool {
        !matches!(self, Self::XOAuth2)
    }
}

impl FromStr for AuthMethod {
    type Err = AuthError;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(s), ret, err(level = "debug"))
    )]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "login" => Self::Login,
            "plain" => Self::Plain,
            "cram-md5" => Self::CramMd5,
            "scram-sha-1" => Self::ScramSha1,
            "scram-sha-256" => Self::ScramSha256,
            "xoauth2" => Self::XOAuth2,
            _ => {
                return Err(AuthError(
                    "Invalid auth method, expects: login, plain, cram-md5, scram-sha-1, scram-sha-256, xoauth2".to_owned(),
                ));
            },
        })
    }
}

impl<'de> Deserialize<'de> for AuthMethod {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(deserializer), ret, err(level = "debug"))
    )]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for AuthMethod {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, serializer), err(level = "debug"))
    )]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match *self {
            Self::Login => "login",
            Self::Plain => "plain",
            Self::CramMd5 => "cram-md5",
            Self::ScramSha1 => "scram-sha-1",
            Self::ScramSha256 => "scram-sha-256",
            Self::XOAuth2 => "xoauth2",
        })
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "test")]
    use super::AuthMethod;

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
    fn auth_method_from_str_plain() {
        assert_eq!(
            "plain"
                .parse::<AuthMethod>()
                .expect("\"plain\" is a valid auth method"),
            AuthMethod::Plain
        );
    }

    #[test]
    fn auth_method_from_str_cram_md5() {
        assert_eq!(
            "cram-md5"
                .parse::<AuthMethod>()
                .expect("\"cram-md5\" is a valid auth method"),
            AuthMethod::CramMd5
        );
    }

    #[test]
    fn auth_method_from_str_scram_sha1() {
        assert_eq!(
            "scram-sha-1"
                .parse::<AuthMethod>()
                .expect("\"scram-sha-1\" is a valid auth method"),
            AuthMethod::ScramSha1
        );
    }

    #[test]
    fn auth_method_from_str_scram_sha256() {
        assert_eq!(
            "scram-sha-256"
                .parse::<AuthMethod>()
                .expect("\"scram-sha-256\" is a valid auth method"),
            AuthMethod::ScramSha256
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
        assert_eq!(
            "SCRAM-SHA-256"
                .parse::<AuthMethod>()
                .expect("auth method parsing must be case-insensitive"),
            AuthMethod::ScramSha256
        );
    }

    #[test]
    fn auth_method_from_str_invalid() {
        assert!("digest-md5".parse::<AuthMethod>().is_err());
    }

    #[test]
    fn auth_method_requires_password_login() {
        assert!(AuthMethod::Login.requires_password());
    }

    #[test]
    fn auth_method_requires_password_plain() {
        assert!(AuthMethod::Plain.requires_password());
    }

    #[test]
    fn auth_method_requires_password_cram_md5() {
        assert!(AuthMethod::CramMd5.requires_password());
    }

    #[test]
    fn auth_method_requires_password_scram_sha1() {
        assert!(AuthMethod::ScramSha1.requires_password());
    }

    #[test]
    fn auth_method_requires_password_scram_sha256() {
        assert!(AuthMethod::ScramSha256.requires_password());
    }

    #[test]
    fn auth_method_requires_password_xoauth2_false() {
        assert!(!AuthMethod::XOAuth2.requires_password());
    }
}
