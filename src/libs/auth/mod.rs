mod cram_md5;
mod plain;
mod scram;
mod xoauth2;

use std::str::FromStr;

use exn::bail;
use rust_i18n::t;
use serde::Deserialize;

pub use self::{cram_md5::CramMd5Auth, plain::PlainAuth, scram::ScramAuth, xoauth2::XOAuth2Auth};

#[derive(Debug, derive_more::Display)]
/// Error type for auth-method parsing.
pub enum AuthError {
    #[display("{}", t!("error.auth.invalid_method"))]
    InvalidMethod,
    #[display("{}", t!("error.auth.scram_config"))]
    ScramConfig,
    #[display("{}", t!("error.auth.scram_invalid_mech"))]
    ScramInvalidMech,
    #[display("{}", t!("error.auth.scram_session_init"))]
    ScramSessionInit,
}
impl std::error::Error for AuthError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum)]
/// Authentication mechanism used when connecting to the IMAP server.
#[derive(Default)]
pub enum AuthMethod {
    /// Standard IMAP LOGIN command (default).
    #[default]
    #[value(help = t!("cli.auth.login"))]
    Login,
    /// SASL PLAIN — credentials in cleartext (RFC 4616). Requires TLS.
    #[value(name = "plain", help = t!("cli.auth.plain"))]
    Plain,
    /// SASL CRAM-MD5 — HMAC-MD5 challenge/response (RFC 2195).
    #[value(name = "cram-md5", help = t!("cli.auth.cram_md5"))]
    CramMd5,
    /// SASL SCRAM-SHA-1 — salted challenge/response (RFC 5802).
    #[value(name = "scram-sha-1", help = t!("cli.auth.scram_sha_1"))]
    ScramSha1,
    /// SASL SCRAM-SHA-256 — salted challenge/response, stronger hash (RFC 7677).
    #[value(name = "scram-sha-256", help = t!("cli.auth.scram_sha_256"))]
    ScramSha256,
    /// SASL XOAUTH2 — bearer-token authentication (Gmail, Office 365).
    #[value(name = "xoauth2", help = t!("cli.auth.xoauth2"))]
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
    type Err = exn::Exn<AuthError>;

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
                bail!(AuthError::InvalidMethod);
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

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "test")]
    use rstest::rstest;

    use super::AuthMethod;

    #[test]
    fn auth_method_default_is_login() {
        assert_eq!(AuthMethod::default(), AuthMethod::Login);
    }

    #[rstest]
    #[case("login", AuthMethod::Login)]
    #[case("plain", AuthMethod::Plain)]
    #[case("cram-md5", AuthMethod::CramMd5)]
    #[case("scram-sha-1", AuthMethod::ScramSha1)]
    #[case("scram-sha-256", AuthMethod::ScramSha256)]
    #[case("xoauth2", AuthMethod::XOAuth2)]
    #[case("XOAUTH2", AuthMethod::XOAuth2)]
    #[case("SCRAM-SHA-256", AuthMethod::ScramSha256)]
    fn auth_method_from_str(#[case] input: &str, #[case] expected: AuthMethod) {
        assert_eq!(
            input
                .parse::<AuthMethod>()
                .expect("valid auth method (case-insensitive)"),
            expected
        );
    }

    #[test]
    fn auth_method_from_str_invalid() {
        assert!("digest-md5".parse::<AuthMethod>().is_err());
    }

    #[rstest]
    #[case(AuthMethod::Login, true)]
    #[case(AuthMethod::Plain, true)]
    #[case(AuthMethod::CramMd5, true)]
    #[case(AuthMethod::ScramSha1, true)]
    #[case(AuthMethod::ScramSha256, true)]
    #[case(AuthMethod::XOAuth2, false)]
    fn auth_method_requires_password(#[case] method: AuthMethod, #[case] expected: bool) {
        assert_eq!(method.requires_password(), expected);
    }
}
