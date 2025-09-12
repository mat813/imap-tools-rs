use eyre::{bail, Result};
use imap::ConnectionMode;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mode(ConnectionMode);

impl FromStr for Mode {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s.to_ascii_lowercase();
        let mode = match v.as_str() {
            "auto_tls" | "autotls" => ConnectionMode::AutoTls,
            "auto" => ConnectionMode::Auto,
            "plaintext" | "none" => ConnectionMode::Plaintext,
            #[cfg(any(feature = "rustls", feature = "openssl"))]
            "tls" => ConnectionMode::Tls,
            #[cfg(any(feature = "rustls", feature = "openssl"))]
            "start_tls" | "starttls" => ConnectionMode::StartTls,
            _ => bail!(
                "Invalid connection mode, expects auto_tls, auto, plaintext, tls and start_tls"
            ),
        };
        Ok(Self(mode))
    }
}

impl<'de> Deserialize<'de> for Mode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Mode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
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
            _ => todo!(),
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

impl From<Mode> for ConnectionMode {
    fn from(value: Mode) -> Self {
        value.0
    }
}

impl From<ConnectionMode> for Mode {
    fn from(value: ConnectionMode) -> Self {
        Self(value)
    }
}
