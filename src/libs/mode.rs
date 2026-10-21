use std::str::FromStr;

use clap::ValueEnum;
use rust_i18n::t;
use serde::Deserialize;

#[derive(Debug, derive_more::Display)]
pub enum ModeError {
    #[display("{}", t!("error.mode.invalid_mode_tls"))]
    InvalidModeTls,
    #[display("{}", t!("error.mode.invalid_mode"))]
    InvalidMode,
}
impl std::error::Error for ModeError {}

#[derive(Clone, Debug, Eq, PartialEq, ValueEnum)]
#[non_exhaustive]
pub enum Mode {
    #[cfg_attr(
        feature = "__tls",
        doc = "Automatically detect what connection mode should be used.",
        doc = "This will use TLS if the port is 993, and otherwise STARTTLS if available.",
        doc = "If no TLS communication mechanism is available, the connection will fail."
    )]
    #[cfg_attr(
        not(feature = "__tls"),
        doc = "TLS is disabled, plaintext will be used"
    )]
    #[cfg_attr(feature = "__tls", value(help = t!("cli.mode.auto_tls")))]
    #[cfg_attr(not(feature = "__tls"), value(help = t!("cli.mode.tls_disabled")))]
    #[value(aliases = ["auto_tls", "autotls"])]
    AutoTls,
    #[cfg_attr(
        feature = "__tls",
        doc = "Automatically detect what connection mode should be used.",
        doc = "This will use TLS if the port is 993, and otherwise STARTTLS if available.",
        doc = "It will fallback to a plaintext connection if no TLS option can be used."
    )]
    #[cfg_attr(
        not(feature = "__tls"),
        doc = "TLS is disabled, plaintext will be used"
    )]
    #[cfg_attr(feature = "__tls", value(help = t!("cli.mode.auto")))]
    #[cfg_attr(not(feature = "__tls"), value(help = t!("cli.mode.tls_disabled")))]
    Auto,
    /// A plain unencrypted TCP connection
    #[value(help = t!("cli.mode.plaintext"))]
    #[value(alias = "none")]
    Plaintext,
    /// An encrypted TLS connection
    #[cfg(feature = "__tls")]
    #[value(help = t!("cli.mode.tls"))]
    Tls,
    /// An eventually-encrypted (i.e., STARTTLS) connection
    #[cfg(feature = "__tls")]
    #[value(help = t!("cli.mode.start_tls"))]
    #[value(aliases = ["start_tls", "starttls"])]
    StartTls,
}

impl FromStr for Mode {
    type Err = exn::Exn<ModeError>;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", ret, err(level = "debug"))
    )]
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        <Self as ValueEnum>::from_str(s, true).map_err(|_| {
            exn::Exn::new(
                if cfg!(feature = "__tls") {
                    ModeError::InvalidModeTls
                } else {
                    ModeError::InvalidMode
                },
            )
        })
    }
}

impl<'de> Deserialize<'de> for Mode {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(deserializer), ret, err(level = "debug"))
    )]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl Default for Mode {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip()))]
    fn default() -> Self {
        if cfg!(feature = "__tls") {
            Self::AutoTls
        } else {
            Self::Plaintext
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "tests")]

    use rstest::rstest;

    use super::Mode;

    #[rstest]
    #[case("auto_tls", Mode::AutoTls)]
    #[case("autotls", Mode::AutoTls)]
    #[case("AUTO_TLS", Mode::AutoTls)]
    #[case("auto", Mode::Auto)]
    #[case("plaintext", Mode::Plaintext)]
    #[case("none", Mode::Plaintext)]
    fn from_str(#[case] input: &str, #[case] expected: Mode) {
        assert_eq!(input.parse::<Mode>().expect("should parse"), expected);
    }

    #[cfg(feature = "__tls")]
    #[rstest]
    #[case("tls", Mode::Tls)]
    #[case("start_tls", Mode::StartTls)]
    #[case("starttls", Mode::StartTls)]
    fn from_str_tls(#[case] input: &str, #[case] expected: Mode) {
        assert_eq!(input.parse::<Mode>().expect("should parse"), expected);
    }

    #[test]
    fn from_str_invalid() {
        let result = "bogus".parse::<Mode>();
        assert!(result.is_err(), "bogus should not parse as a valid mode");
    }

    #[cfg(feature = "__tls")]
    #[test]
    fn default_is_auto_tls() {
        assert_eq!(Mode::default(), Mode::AutoTls);
    }
}
