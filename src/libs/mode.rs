use derive_more::{Display, From, Into};
use imap::ConnectionMode;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Display)]
pub struct ModeError(String);
impl std::error::Error for ModeError {}

#[derive(Debug, Clone, PartialEq, Eq, From, Into)]
pub struct Mode(ConnectionMode);

impl FromStr for Mode {
    type Err = ModeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let v = s.to_ascii_lowercase();
        let mode =
            match v.as_str() {
                "auto_tls" | "autotls" => ConnectionMode::AutoTls,
                "auto" => ConnectionMode::Auto,
                "plaintext" | "none" => ConnectionMode::Plaintext,
                #[cfg(any(feature = "rustls", feature = "openssl"))]
                "tls" => ConnectionMode::Tls,
                #[cfg(any(feature = "rustls", feature = "openssl"))]
                "start_tls" | "starttls" => ConnectionMode::StartTls,
                _ => return Err(ModeError(
                    "Invalid connection mode, expects auto_tls, auto, plaintext, tls and start_tls"
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
