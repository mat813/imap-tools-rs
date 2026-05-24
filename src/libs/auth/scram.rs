use exn::ResultExt as _;
use rsasl::prelude::{Mechname, SASLClient, SASLConfig};

use crate::libs::auth::AuthError;

/// SASL SCRAM authenticator (SHA-1 or SHA-256) via `rsasl`.
///
/// Drives the multi-step SCRAM exchange; async-imap handles base64 wrapping.
#[derive(derive_more::Debug)]
pub struct ScramAuth {
    /// Active rsasl session driving the SCRAM state machine.
    #[debug(skip)]
    session: rsasl::prelude::Session,
    /// `false` on the first call to `process` (client-first step needs `None` input).
    started: bool,
}

impl ScramAuth {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "trace",
            skip(mech, user, password),
            ret,
            err(level = "debug")
        )
    )]
    /// Create a new SCRAM session for the given mechanism name (e.g. `"SCRAM-SHA-256"`).
    ///
    /// # Errors
    /// Returns [`AuthError`] if rsasl cannot initialise the session.
    pub fn new(mech: &[u8], user: String, password: String) -> exn::Result<Self, AuthError> {
        let config = SASLConfig::with_credentials(None, user, password)
            .or_raise(|| AuthError::ScramConfig)?;
        let client = SASLClient::new(config);
        let mechname = Mechname::parse(mech).or_raise(|| AuthError::ScramInvalidMech)?;
        let session = client
            .start_suggested(&[mechname])
            .or_raise(|| AuthError::ScramSessionInit)?;
        Ok(Self {
            session,
            started: false,
        })
    }
}

impl async_imap::Authenticator for ScramAuth {
    type Response = Vec<u8>;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, challenge), ret)
    )]
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

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "test")]
    use async_imap::Authenticator as _;

    use super::ScramAuth;

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
}
