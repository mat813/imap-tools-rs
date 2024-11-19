#[derive(Debug)]
#[expect(clippy::module_name_repetitions)]
pub enum OurError {
    // External errors
    Imap(imap::Error),
    ShellWords(shell_words::ParseError),
    StdIo(std::io::Error),
    Serde(serde_any::Error),
    Strfmt(strfmt::FmtError),
    TryFromInt(std::num::TryFromIntError),

    // Internal errors
    Internal(String),
    Uidplus,
}

pub type OurResult<T> = std::result::Result<T, OurError>;

impl std::error::Error for OurError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Imap(e) => Some(e),
            Self::StdIo(e) => Some(e),
            Self::Serde(_e) => None, /* Some(e), */
            Self::ShellWords(e) => Some(e),
            Self::Strfmt(e) => Some(e),
            Self::TryFromInt(e) => Some(e),
            Self::Internal(_) | Self::Uidplus => None,
        }
    }
}

// Implement Display if you need to format the error message
impl std::fmt::Display for OurError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Imap(e) => write!(f, "IMAP Error: {e}"),
            Self::StdIo(e) => write!(f, "IO Error: {e}"),
            Self::Serde(e) => write!(f, "TOML De Error: {e}"),
            Self::ShellWords(e) => write!(f, "Command parse error: {e}"),
            Self::Strfmt(e) => write!(f, "Format error: {e}"),
            Self::TryFromInt(e) => write!(f, "Int conversion error: {e}"),
            Self::Internal(e) => write!(f, "Internal error: {e}"),
            Self::Uidplus => write!(
                f,
                "The server does not support the UIDPLUS capability, and all our operations need UIDs for safety",
            ),
        }
    }
}

macro_rules! impl_error_conversions {
    ($enum_type:ident, $( ($variant:ident, $from_type:ty) ),* $(,)?) => {
        $(
            impl From<$from_type> for $enum_type {
                fn from(err: $from_type) -> Self {
                    Self::$variant(err.into())
                }
            }
        )*
    };
}

impl_error_conversions!(
    OurError,
    (Internal, String),
    (Internal, &str),
    (StdIo, std::io::Error),
    (Serde, serde_any::Error),
    (ShellWords, shell_words::ParseError),
    (Strfmt, strfmt::FmtError),
    (Imap, imap::Error),
    (TryFromInt, std::num::TryFromIntError),
);

impl From<(imap::Error, imap::Client<Box<dyn imap::ImapConnection>>)> for OurError {
    fn from((err, _): (imap::Error, imap::Client<Box<dyn imap::ImapConnection>>)) -> Self {
        Self::Imap(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use imap::Error as ImapError;
    use serde_any::Error as SerdeError;
    use shell_words::ParseError as ShellParseError;
    use std::{error::Error, io};

    #[test]
    fn test_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::Other, "io error");
        let our_error: OurError = io_error.into();
        match our_error {
            OurError::StdIo(e) => assert_eq!(e.to_string(), "io error"),
            _ => panic!("Expected StdIo variant"),
        }
    }

    #[test]
    fn test_from_serde_error() {
        let serde_error = SerdeError::UnsupportedFileExtension("foo".into());
        let our_error: OurError = serde_error.into();
        match our_error {
            OurError::Serde(e) => assert_eq!(e.to_string(), "File extension foo not supported"),
            _ => panic!("Expected Serde variant"),
        }
    }

    #[test]
    fn test_from_shellwords_parse_error() {
        let shell_parse_error = ShellParseError;
        let our_error: OurError = shell_parse_error.into();
        match our_error {
            OurError::ShellWords(e) => assert_eq!(e.to_string(), "missing closing quote"),
            _ => panic!("Expected ShellWords variant"),
        }
    }

    #[test]
    fn test_from_imap_error() {
        let imap_error = ImapError::ConnectionLost;
        let our_error: OurError = imap_error.into();
        match our_error {
            OurError::Imap(e) => assert_eq!(e.to_string(), "Connection Lost"),
            _ => panic!("Expected Imap variant"),
        }
    }

    #[test]
    fn test_from_string() {
        let config_error = String::from("config error");
        let our_error: OurError = config_error.into();
        match our_error {
            OurError::Internal(e) => assert_eq!(e, "config error"),
            _ => panic!("Expected Config variant"),
        }
    }

    #[test]
    fn test_from_str() {
        let config_error = "config error";
        let our_error: OurError = config_error.into();
        match our_error {
            OurError::Internal(e) => assert_eq!(e, "config error"),
            _ => panic!("Expected Config variant"),
        }
    }

    #[test]
    fn test_display_imap() {
        let imap_error = ImapError::ConnectionLost;
        let our_error = OurError::Imap(imap_error);
        assert_eq!(our_error.to_string(), "IMAP Error: Connection Lost");
    }

    #[test]
    fn test_display_std_io() {
        let io_error = io::Error::new(io::ErrorKind::Other, "io error");
        let our_error = OurError::StdIo(io_error);
        assert_eq!(our_error.to_string(), "IO Error: io error");
    }

    #[test]
    fn test_display_serde() {
        let serde_error = SerdeError::UnsupportedFileExtension("foo".into());
        let our_error = OurError::Serde(serde_error);
        assert_eq!(
            our_error.to_string(),
            "TOML De Error: File extension foo not supported"
        );
    }

    #[test]
    fn test_display_shellwords() {
        let shell_parse_error = ShellParseError;
        let our_error = OurError::ShellWords(shell_parse_error);
        assert_eq!(
            our_error.to_string(),
            "Command parse error: missing closing quote"
        );
    }

    #[test]
    fn test_display_config() {
        let our_error = OurError::Internal("config error".to_string());
        assert_eq!(our_error.to_string(), "Internal error: config error");
    }

    #[test]
    fn test_display_uidplus() {
        let our_error = OurError::Uidplus;
        assert_eq!(
        our_error.to_string(),
        "The server does not support the UIDPLUS capability, and all our operations need UIDs for safety"
    );
    }

    #[test]
    fn test_error_source() {
        // Test source for each variant
        let imap_error = ImapError::ConnectionLost;
        let our_error = OurError::Imap(imap_error);
        assert!(our_error.source().is_some());

        let io_error = io::Error::new(io::ErrorKind::Other, "io error");
        let our_error = OurError::StdIo(io_error);
        assert!(our_error.source().is_some());

        let shell_parse_error = ShellParseError;
        let our_error = OurError::ShellWords(shell_parse_error);
        assert!(our_error.source().is_some());

        // Config and Uidplus errors have no source
        let config_error = OurError::Internal("config error".to_string());
        assert!(config_error.source().is_none());

        let uidplus_error = OurError::Uidplus;
        assert!(uidplus_error.source().is_none());
    }
}
