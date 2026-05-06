use std::{fmt::Write as _, str::FromStr};

use derive_more::Display;
use hmac::{Hmac, Mac as _};
use md5::Md5;
use rsasl::prelude::{Mechname, SASLClient, SASLConfig};
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
            Self::Plain => "plain",
            Self::CramMd5 => "cram-md5",
            Self::ScramSha1 => "scram-sha-1",
            Self::ScramSha256 => "scram-sha-256",
            Self::XOAuth2 => "xoauth2",
        })
    }
}

/// SASL PLAIN authenticator for use with [`async_imap::Client::authenticate`].
///
/// Encodes `\0authcid\0passwd`; async-imap handles base64 wrapping.
pub struct PlainAuth {
    /// IMAP username.
    pub user: String,
    /// Password.
    pub password: String,
}

impl async_imap::Authenticator for PlainAuth {
    type Response = Vec<u8>;

    fn process(&mut self, _challenge: &[u8]) -> Self::Response {
        let mut r = Vec::with_capacity(self.user.len() + self.password.len() + 2);
        r.push(0); // empty authzid
        r.extend_from_slice(self.user.as_bytes());
        r.push(0);
        r.extend_from_slice(self.password.as_bytes());
        r
    }
}

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

/// SASL SCRAM authenticator (SHA-1 or SHA-256) via `rsasl`.
///
/// Drives the multi-step SCRAM exchange; async-imap handles base64 wrapping.
pub struct ScramAuth {
    /// Active rsasl session driving the SCRAM state machine.
    session: rsasl::prelude::Session,
    /// `false` on the first call to `process` (client-first step needs `None` input).
    started: bool,
}

impl ScramAuth {
    /// Create a new SCRAM session for the given mechanism name (e.g. `"SCRAM-SHA-256"`).
    ///
    /// # Errors
    /// Returns [`AuthError`] if rsasl cannot initialise the session.
    pub fn new(mech: &[u8], user: String, password: String) -> Result<Self, AuthError> {
        let config = SASLConfig::with_credentials(None, user, password)
            .map_err(|e| AuthError(format!("SCRAM config error: {e}")))?;
        let client = SASLClient::new(config);
        let mechname =
            Mechname::parse(mech).map_err(|e| AuthError(format!("invalid mechanism: {e}")))?;
        let session = client
            .start_suggested(&[mechname])
            .map_err(|e| AuthError(format!("SCRAM session init: {e}")))?;
        Ok(Self {
            session,
            started: false,
        })
    }
}

impl async_imap::Authenticator for ScramAuth {
    type Response = Vec<u8>;

    fn process(&mut self, challenge: &[u8]) -> Self::Response {
        let input = if self.started {
            Some(challenge)
        } else {
            self.started = true;
            None // client-first: drive the mechanism before any server challenge
        };
        let mut out = Vec::new();
        match self.session.step(input, &mut out) {
            Ok(_) | Err(_) => out,
        }
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

    use super::{AuthMethod, CramMd5Auth, PlainAuth, ScramAuth, XOAuth2Auth};

    // --- PLAIN ---

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

    // --- CRAM-MD5 ---

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

    // --- SCRAM ---

    #[test]
    fn scram_sha1_first_step_starts_with_client_first() {
        let mut auth = ScramAuth::new(
            b"SCRAM-SHA-1",
            "user@example.com".to_owned(),
            "secret".to_owned(),
        )
        .expect("SCRAM-SHA-1 session should initialise");
        let response = auth.process(&[]);
        let s = std::str::from_utf8(&response).expect("SCRAM response should be UTF-8");
        assert!(
            s.starts_with("n,,n="),
            "SCRAM client-first-message must begin with 'n,,n=' (gs2-cbind-flag + username); got: {s:?}"
        );
    }

    #[test]
    fn scram_sha256_first_step_starts_with_client_first() {
        let mut auth = ScramAuth::new(
            b"SCRAM-SHA-256",
            "user@example.com".to_owned(),
            "secret".to_owned(),
        )
        .expect("SCRAM-SHA-256 session should initialise");
        let response = auth.process(&[]);
        let s = std::str::from_utf8(&response).expect("SCRAM response should be UTF-8");
        assert!(
            s.starts_with("n,,n="),
            "SCRAM client-first-message must begin with 'n,,n='; got: {s:?}"
        );
    }

    // --- XOAUTH2 ---

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

    // --- AuthMethod enum ---

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
