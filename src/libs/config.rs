use crate::libs::{args::Generic, error::Error, filters::Filters};
use serde::{Deserialize, Serialize};
use shell_words::split;
use std::fmt::Debug;
use std::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config<T>
where
    T: Clone + Debug + Serialize,
{
    pub server: Option<String>,

    pub username: Option<String>,

    pub password: Option<String>,

    pub password_command: Option<String>,

    #[serde(default)]
    pub debug: bool,

    #[serde(default)]
    pub dry_run: bool,

    pub extra: Option<T>,

    pub filters: Option<Filters<T>>,
}

impl<T> Config<T>
where
    T: Clone + Debug + Serialize + for<'de> Deserialize<'de>,
{
    /// Creates from a file and arguments
    /// # Errors
    /// Many errors can happen
    pub fn new_with_args(args: &Generic) -> Result<Self, Error> {
        let mut config = if let Some(ref config) = args.config {
            serde_any::from_file(config)?
        } else {
            Self {
                server: None,
                username: None,
                password: None,
                password_command: None,
                debug: false,
                dry_run: false,
                extra: None,
                filters: None,
            }
        };

        if let Some(ref server) = args.server {
            config.server = Some(server.clone());
        }

        if let Some(ref username) = args.username {
            config.username = Some(username.clone());
        }

        if let Some(ref password) = args.password {
            config.password = Some(password.clone());
        }

        if let Some(ref password_command) = args.password_command {
            config.password_command = Some(password_command.clone());
        }

        if args.debug {
            config.debug = args.debug;
        }

        if args.dry_run {
            config.dry_run = args.dry_run;
        }

        if config.server.is_none() {
            return Err(Error::config("The server must be set"));
        }

        if config.username.is_none() {
            return Err(Error::config("The username must be set"));
        }

        if config.password.is_none() && config.password_command.is_none() {
            return Err(Error::config(
                "The password or password command must be set",
            ));
        }

        Ok(config)
    }
}

impl<T> Config<T>
where
    T: Clone + Debug + Serialize,
{
    /// Figure out the password from literal or command
    /// # Errors
    /// Many errors can happen
    pub fn password(&self) -> Result<String, Error> {
        if let Some(ref pass) = self.password {
            Ok(pass.clone())
        } else if let Some(ref command) = self.password_command {
            let args = split(command)?;
            let (exe, args) = args
                .split_first()
                .ok_or_else(|| Error::config("password command is empty"))?;
            let output = Command::new(exe).args(args).output()?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(Error::config(
                "the password or password command must be set",
            ))
        }
    }
}
