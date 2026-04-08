use std::str::FromStr;

use derive_more::Display;
use imap::ConnectionMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display)]
pub struct ModeError(String);
impl std::error::Error for ModeError {}

// #[derive(Debug, Clone, PartialEq, Eq, From, Into)]
// pub struct Mode(ConnectionMode);
#[derive(Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
#[non_exhaustive]
pub enum Mode {
    #[cfg_attr(
        any(feature = "openssl", feature = "rustls"),
        doc = "Automatically detect what connection mode should be used.",
        doc = "This will use TLS if the port is 993, and otherwise STARTTLS if available.",
        doc = "If no TLS communication mechanism is available, the connection will fail."
    )]
    #[cfg_attr(
        not(any(feature = "openssl", feature = "rustls")),
        doc = "TLS is disabled, plaintext will be used"
    )]
    AutoTls,
    #[cfg_attr(
        any(feature = "openssl", feature = "rustls"),
        doc = "Automatically detect what connection mode should be used.",
        doc = "This will use TLS if the port is 993, and otherwise STARTTLS if available.",
        doc = "It will fallback to a plaintext connection if no TLS option can be used."
    )]
    #[cfg_attr(
        not(any(feature = "openssl", feature = "rustls")),
        doc = "TLS is disabled, plaintext will be used"
    )]
    Auto,
    /// A plain unencrypted TCP connection
    Plaintext,
    /// An encrypted TLS connection
    #[cfg(any(feature = "openssl", feature = "rustls"))]
    Tls,
    /// An eventually-encrypted (i.e., STARTTLS) connection
    #[cfg(any(feature = "openssl", feature = "rustls"))]
    StartTls,
}

impl From<Mode> for ConnectionMode {
    fn from(value: Mode) -> Self {
        match value {
            Mode::AutoTls => Self::AutoTls,
            Mode::Auto => Self::Auto,
            Mode::Plaintext => Self::Plaintext,
            #[cfg(any(feature = "openssl", feature = "rustls"))]
            Mode::Tls => Self::Tls,
            #[cfg(any(feature = "openssl", feature = "rustls"))]
            Mode::StartTls => Self::StartTls,
        }
    }
}

impl TryFrom<ConnectionMode> for Mode {
    type Error = ModeError;

    fn try_from(value: ConnectionMode) -> Result<Self, Self::Error> {
        match value {
            ConnectionMode::AutoTls => Ok(Self::AutoTls),
            ConnectionMode::Auto => Ok(Self::Auto),
            ConnectionMode::Plaintext => Ok(Self::Plaintext),
            #[cfg(any(feature = "openssl", feature = "rustls"))]
            ConnectionMode::Tls => Ok(Self::Tls),
            #[cfg(any(feature = "openssl", feature = "rustls"))]
            ConnectionMode::StartTls => Ok(Self::StartTls),
            _ => Err(ModeError("Invalid connection mode".to_owned())),
        }
    }
}

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
        mode.try_into()
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
        let s = match *self {
            Self::AutoTls => "auto_tls",
            Self::Auto => "auto",
            Self::Plaintext => "plaintext",
            #[cfg(any(feature = "rustls", feature = "openssl"))]
            Self::Tls => "tls",
            #[cfg(any(feature = "rustls", feature = "openssl"))]
            Self::StartTls => "start_tls",
        };
        serializer.serialize_str(s)
    }
}

impl Default for Mode {
    fn default() -> Self {
        if cfg!(any(feature = "rustls", feature = "openssl")) {
            Self::AutoTls
        } else {
            Self::Plaintext
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "tests")]

    use serde_any::{Format, from_str, to_string};

    use super::Mode;

    #[test]
    fn from_str_auto_tls() {
        let mode = "auto_tls".parse::<Mode>().expect("auto_tls should parse");
        assert_eq!(mode, Mode::AutoTls);
    }

    #[test]
    fn from_str_autotls_alias() {
        let mode = "autotls".parse::<Mode>().expect("autotls should parse");
        assert_eq!(mode, Mode::AutoTls);
    }

    #[test]
    fn from_str_auto() {
        let mode = "auto".parse::<Mode>().expect("auto should parse");
        assert_eq!(mode, Mode::Auto);
    }

    #[test]
    fn from_str_plaintext() {
        let mode = "plaintext".parse::<Mode>().expect("plaintext should parse");
        assert_eq!(mode, Mode::Plaintext);
    }

    #[test]
    fn from_str_none_alias() {
        let mode = "none".parse::<Mode>().expect("none should parse");
        assert_eq!(mode, Mode::Plaintext);
    }

    #[cfg(any(feature = "rustls", feature = "openssl"))]
    #[test]
    fn from_str_tls() {
        let mode = "tls".parse::<Mode>().expect("tls should parse");
        assert_eq!(mode, Mode::Tls);
    }

    #[cfg(any(feature = "rustls", feature = "openssl"))]
    #[test]
    fn from_str_start_tls() {
        let mode = "start_tls".parse::<Mode>().expect("start_tls should parse");
        assert_eq!(mode, Mode::StartTls);
    }

    #[cfg(any(feature = "rustls", feature = "openssl"))]
    #[test]
    fn from_str_starttls_alias() {
        let mode = "starttls".parse::<Mode>().expect("starttls should parse");
        assert_eq!(mode, Mode::StartTls);
    }

    #[test]
    fn from_str_case_insensitive() {
        let mode = "AUTO_TLS".parse::<Mode>().expect("AUTO_TLS should parse");
        assert_eq!(mode, Mode::AutoTls);
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
        assert_eq!(Mode::default(), Mode::AutoTls);
    }
}
