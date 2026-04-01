use std::str::FromStr;

use derive_more::{Display, From, Into};
use imap::ConnectionMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display)]
pub struct ModeError(String);
impl std::error::Error for ModeError {}

#[derive(Debug, Clone, PartialEq, Eq, From, Into)]
pub struct Mode(ConnectionMode);

impl FromStr for Mode {
    type Err = ModeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let v = s.to_ascii_lowercase();
        let mode = match v.as_str() {
            "auto_tls" | "autotls" => ConnectionMode::AutoTls,
            "auto" => ConnectionMode::Auto,
            "plaintext" | "none" => ConnectionMode::Plaintext,
            #[cfg(any(feature = "rustls", feature = "openssl"))]
            "tls" => ConnectionMode::Tls,
            #[cfg(any(feature = "rustls", feature = "openssl"))]
            "start_tls" | "starttls" => ConnectionMode::StartTls,
            _ => return Err(ModeError(
                if cfg!(any(feature = "rustls", feature = "openssl")) {
                    "Invalid connection mode, expects: auto_tls, auto, plaintext, tls, start_tls"
                } else {
                    "Invalid connection mode, expects: auto_tls, auto, plaintext"
                }
                .to_owned(),
            )),
        };
        Ok(Self(mode))
    }
}

impl<'de> Deserialize<'de> for Mode {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Mode {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self.0 {
            ConnectionMode::AutoTls => "auto_tls",
            ConnectionMode::Auto => "auto",
            ConnectionMode::Plaintext => "plaintext",
            #[cfg(any(feature = "rustls", feature = "openssl"))]
            ConnectionMode::Tls => "tls",
            #[cfg(any(feature = "rustls", feature = "openssl"))]
            ConnectionMode::StartTls => "start_tls",
            ref m => {
                return Err(serde::ser::Error::custom(format!(
                    "cannot serialize unknown ConnectionMode variant: {m:?}",
                )));
            },
        };
        serializer.serialize_str(s)
    }
}

impl Default for Mode {
    fn default() -> Self {
        if cfg!(any(feature = "rustls", feature = "openssl")) {
            Self(ConnectionMode::AutoTls)
        } else {
            Self(ConnectionMode::Plaintext)
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "tests")]

    use imap::ConnectionMode;
    use serde_any::{Format, from_str, to_string};

    use super::Mode;

    #[test]
    fn from_str_auto_tls() {
        let mode = "auto_tls".parse::<Mode>().expect("auto_tls should parse");
        assert_eq!(mode, Mode::from(ConnectionMode::AutoTls));
    }

    #[test]
    fn from_str_autotls_alias() {
        let mode = "autotls".parse::<Mode>().expect("autotls should parse");
        assert_eq!(mode, Mode::from(ConnectionMode::AutoTls));
    }

    #[test]
    fn from_str_auto() {
        let mode = "auto".parse::<Mode>().expect("auto should parse");
        assert_eq!(mode, Mode::from(ConnectionMode::Auto));
    }

    #[test]
    fn from_str_plaintext() {
        let mode = "plaintext".parse::<Mode>().expect("plaintext should parse");
        assert_eq!(mode, Mode::from(ConnectionMode::Plaintext));
    }

    #[test]
    fn from_str_none_alias() {
        let mode = "none".parse::<Mode>().expect("none should parse");
        assert_eq!(mode, Mode::from(ConnectionMode::Plaintext));
    }

    #[cfg(any(feature = "rustls", feature = "openssl"))]
    #[test]
    fn from_str_tls() {
        let mode = "tls".parse::<Mode>().expect("tls should parse");
        assert_eq!(mode, Mode::from(ConnectionMode::Tls));
    }

    #[cfg(any(feature = "rustls", feature = "openssl"))]
    #[test]
    fn from_str_start_tls() {
        let mode = "start_tls".parse::<Mode>().expect("start_tls should parse");
        assert_eq!(mode, Mode::from(ConnectionMode::StartTls));
    }

    #[cfg(any(feature = "rustls", feature = "openssl"))]
    #[test]
    fn from_str_starttls_alias() {
        let mode = "starttls".parse::<Mode>().expect("starttls should parse");
        assert_eq!(mode, Mode::from(ConnectionMode::StartTls));
    }

    #[test]
    fn from_str_case_insensitive() {
        let mode = "AUTO_TLS".parse::<Mode>().expect("AUTO_TLS should parse");
        assert_eq!(mode, Mode::from(ConnectionMode::AutoTls));
    }

    #[test]
    fn from_str_invalid() {
        let result = "bogus".parse::<Mode>();
        assert!(result.is_err(), "bogus should not parse as a valid mode");
    }

    #[test]
    fn serde_roundtrip() {
        for s in &["auto_tls", "auto", "plaintext"] {
            let mode: Mode = s.parse().expect("should parse");
            let json = to_string(&mode, Format::Json).expect("should serialize");
            let back: Mode = from_str(&json, Format::Json).expect("should deserialize");
            assert_eq!(mode, back);
        }
    }

    #[cfg(any(feature = "rustls", feature = "openssl"))]
    #[test]
    fn serde_roundtrip_tls_modes() {
        for s in &["tls", "start_tls"] {
            let mode: Mode = s.parse().expect("should parse");
            let json = to_string(&mode, Format::Json).expect("should serialize");
            let back: Mode = from_str(&json, Format::Json).expect("should deserialize");
            assert_eq!(mode, back);
        }
    }

    #[cfg(any(feature = "rustls", feature = "openssl"))]
    #[test]
    fn default_is_auto_tls() {
        assert_eq!(Mode::default(), Mode::from(ConnectionMode::AutoTls));
    }
}
