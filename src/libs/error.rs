#[derive(Debug)]
pub enum OurError {
    // External errors
    Imap(imap::Error),
    ShellWords(shell_words::ParseError),
    StdIo(std::io::Error),
    Serde(serde_any::Error),

    // Internal errors
    Config(String),
    Uidplus,
}

pub type OurResult<T> = std::result::Result<T, OurError>;

impl std::error::Error for OurError {}

impl OurError {
    // Constructor for Config variant that takes any AsRef<str>
    pub fn config<S>(message: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::Config(message.as_ref().to_string())
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
            Self::Config(e) => write!(f, "Configuration error: {e}"),
            Self::Uidplus => write!(
                f,
                "The server does not support the UIDPLUS capability, and all our operations need UIDs for safety",
            ),
        }
    }
}

// Implement the conversion from std::io::Error
impl From<std::io::Error> for OurError {
    fn from(err: std::io::Error) -> Self {
        Self::StdIo(err)
    }
}

impl From<serde_any::Error> for OurError {
    fn from(err: serde_any::Error) -> Self {
        Self::Serde(err)
    }
}

impl From<shell_words::ParseError> for OurError {
    fn from(err: shell_words::ParseError) -> Self {
        Self::ShellWords(err)
    }
}

impl From<imap::Error> for OurError {
    fn from(err: imap::Error) -> Self {
        Self::Imap(err)
    }
}

impl From<(imap::Error, imap::Client<Box<dyn imap::ImapConnection>>)> for OurError {
    fn from(err: (imap::Error, imap::Client<Box<dyn imap::ImapConnection>>)) -> Self {
        Self::Imap(err.0)
    }
}
