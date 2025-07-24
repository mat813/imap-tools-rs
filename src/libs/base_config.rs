use crate::libs::args::Generic;
use anyhow::{anyhow, bail, Context as _, Result};
use serde::{Deserialize, Serialize};
use shell_words::split;
use std::fmt::Debug;
use std::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct BaseConfig {
    pub server: Option<String>,

    pub username: Option<String>,

    pub(self) password: Option<String>,

    pub(self) password_command: Option<String>,

    #[serde(default)]
    pub debug: bool,

    #[serde(default)]
    pub dry_run: bool,
}

impl BaseConfig {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(args), err(level = "info"))
    )]
    /// Creates from a file and arguments
    /// # Errors
    /// Many errors can happen
    pub fn new(args: &Generic) -> Result<Self> {
        #[cfg(feature = "tracing")]
        tracing::trace!(?args);

        let config = if let Some(ref config) = args.config {
            serde_any::from_file(config)
                .map_err(|err| anyhow!("config file parsing failed: {err:?}"))?
        } else {
            Self::default()
        };

        config.apply_args(args)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, args), ret, err(level = "info"))
    )]
    pub fn apply_args(mut self, args: &Generic) -> Result<Self> {
        if let Some(ref server) = args.server {
            self.server = Some(server.clone());
        }

        if let Some(ref username) = args.username {
            self.username = Some(username.clone());
        }

        if let Some(ref password) = args.password {
            self.password = Some(password.clone());
        }

        if let Some(ref password_command) = args.password_command {
            self.password_command = Some(password_command.clone());
        }

        if args.debug {
            self.debug = args.debug;
        }

        if args.dry_run {
            self.dry_run = args.dry_run;
        }

        if self.server.is_none() {
            bail!("The server must be set");
        }

        if self.username.is_none() {
            bail!("The username must be set");
        }

        if self.password.is_none() && self.password_command.is_none() {
            bail!("The password or password command must be set");
        }

        Ok(self)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), ret, err(level = "info"))
    )]
    /// Figure out the password from literal or command
    /// # Errors
    /// Many errors can happen
    pub fn password(&self) -> Result<String> {
        #[expect(clippy::needless_borrowed_reference, reason = "ok")]
        match (&self.password, &self.password_command) {
            (&Some(ref pass), _) => Ok(pass.clone()),
            (_, &Some(ref command)) => {
                let args =
                    split(command).with_context(|| format!("parsing command failed: {command}"))?;
                let (exe, args) = args.split_first().context("password command is empty")?;
                let output = Command::new(exe)
                    .args(args)
                    .output()
                    .context("password command exec failed")?;
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
            _ => Err(anyhow!("The password or password command must be set")),
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test")]

    use super::*;
    use std::fs::File;
    use std::io::Write as _;

    // Helper to create temporary config files with given content.
    // We have to return the directory too otherwise it goes out of scope, gets
    // destroyed, and the directory is deleteda.
    fn write_temp_config(content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_file_path = temp_dir.path().join("config.toml");
        let mut file = File::create(&temp_file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        (temp_dir, temp_file_path)
    }

    #[test]
    fn new_with_args_minimal_config() {
        // Create a minimal args with required fields only
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password: Some("password123".to_owned()),
            password_command: None,
            debug: true,
            dry_run: false,
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();

        assert_eq!(config.server, Some("imap.example.com".to_owned()));
        assert_eq!(config.username, Some("user@example.com".to_owned()));
        assert_eq!(config.password, Some("password123".to_owned()));
        assert!(config.debug);
        assert!(!config.dry_run);
    }

    #[test]
    fn new_with_args_missing_server_error() {
        let args = Generic {
            config: None,
            server: None,
            username: Some("user@example.com".to_owned()),
            password: Some("password123".to_owned()),
            password_command: None,
            debug: false,
            dry_run: false,
        };

        let result: Result<BaseConfig> = BaseConfig::new(&args);
        assert!(result.is_err());
        assert_eq!(
            format!("{:?}", result.unwrap_err()),
            "The server must be set"
        );
    }

    #[test]
    fn new_with_args_missing_username_error() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_owned()),
            username: None,
            password: Some("password123".to_owned()),
            password_command: None,
            debug: false,
            dry_run: false,
        };

        let result: Result<BaseConfig> = BaseConfig::new(&args);
        assert!(result.is_err());
        assert_eq!(
            format!("{:?}", result.unwrap_err()),
            "The username must be set"
        );
    }

    #[test]
    fn password_fn_command_execution() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password: None,
            password_command: Some("echo secret_password".to_owned()),
            debug: false,
            dry_run: false,
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let password = config.password().unwrap();
        assert_eq!(password.trim(), "secret_password");
    }

    #[test]
    fn password_fn_command_cannot_be_parsed() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password: None,
            password_command: Some(r#"echo "secret_password"#.to_owned()),
            debug: false,
            dry_run: false,
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let result = config.password();
        assert!(result.is_err());
        assert_eq!(
            format!("{:?}", result.unwrap_err()),
            "parsing command failed: echo \"secret_password\n\nCaused by:\n    missing closing quote"
        );
    }

    #[test]
    fn password_fn_command_fails() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password: None,
            password_command: Some("exit 1".to_owned()),
            debug: false,
            dry_run: false,
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let result = config.password();
        assert!(result.is_err());
        assert_eq!(
            format!("{:?}", result.unwrap_err()),
            "password command exec failed\n\nCaused by:\n    No such file or directory (os error 2)"
        );
    }

    #[test]
    fn password_fn_command_empty() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password: None,
            password_command: Some(String::new()),
            debug: false,
            dry_run: false,
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let result = config.password();
        assert!(result.is_err());
        assert_eq!(
            format!("{:?}", result.unwrap_err()),
            "password command is empty"
        );
    }

    #[test]
    fn password_fn_static() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password: Some("secret_password".to_owned()),
            password_command: None,
            debug: false,
            dry_run: false,
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let password = config.password().unwrap();
        assert_eq!(password.trim(), "secret_password");
    }

    #[test]
    fn password_error_when_missing() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password: None,
            password_command: None,
            debug: false,
            dry_run: false,
        };

        let config: Result<BaseConfig> = BaseConfig::new(&args);

        assert!(config.is_err());
        assert_eq!(
            format!("{:?}", config.unwrap_err()),
            "The password or password command must be set"
        );
    }

    #[test]
    fn config_loading_from_file_bad_config() {
        let config_content = r#"
        server = "imap.example.com
    "#;

        let (_temp_dir, config_path) = write_temp_config(config_content);

        let args = Generic {
            config: Some(config_path),
            server: None,
            username: None,
            password: None,
            password_command: None,
            debug: false,
            dry_run: false,
        };

        let config: Result<BaseConfig> = BaseConfig::new(&args);
        assert!(config.is_err());
        assert_eq!(
            format!("{:?}", config.unwrap_err()),
            "config file parsing failed: TomlDeserialize(Error { inner: ErrorInner { kind: NewlineInString, line: Some(1), col: 34, message: \"\", key: [] } })"
        );
    }

    #[test]
    fn config_loading_from_file() {
        let config_content = r#"
        server = "imap.example.com"
        username = "user@example.com"
        password = "password123"
        debug = true
        dry-run = true
    "#;

        let (_temp_dir, config_path) = write_temp_config(config_content);

        let args = Generic {
            config: Some(config_path),
            server: None,
            username: None,
            password: None,
            password_command: None,
            debug: false,
            dry_run: false,
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();
        assert_eq!(config.server, Some("imap.example.com".to_owned()));
        assert_eq!(config.username, Some("user@example.com".to_owned()));
        assert_eq!(config.password, Some("password123".to_owned()));
        assert!(config.debug);
        assert!(config.dry_run);
    }

    #[test]
    fn arg_overrides_file_config() {
        let config_content = r#"
        server = "imap.example.com"
        username = "user@example.com"
        password = "password123"
        debug = false
        dry-run = false
    "#;

        let (_temp_dir, config_path) = write_temp_config(config_content);

        let args = Generic {
            config: Some(config_path),
            server: Some("override.example.com".to_owned()),
            username: Some("override_user@example.com".to_owned()),
            password: Some("override_password".to_owned()),
            password_command: None,
            debug: true,
            dry_run: true,
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();
        assert_eq!(config.server, Some("override.example.com".to_owned()));
        assert_eq!(
            config.username,
            Some("override_user@example.com".to_owned())
        );
        assert_eq!(config.password, Some("override_password".to_owned()));
        assert!(config.debug);
        assert!(config.dry_run);
    }
}
