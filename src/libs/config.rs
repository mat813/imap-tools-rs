use crate::libs::{args::Generic, filters::Filters};
use anyhow::{anyhow, Context, Result};
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

    pub(self) password: Option<String>,

    pub(self) password_command: Option<String>,

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
    pub fn new_with_args(args: &Generic) -> Result<Self> {
        let mut config = if let Some(ref config) = args.config {
            serde_any::from_file(config)
                .map_err(|err| anyhow!("config file parsing failed: {err:?}"))?
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
            Err(anyhow!("The server must be set"))?;
        }

        if config.username.is_none() {
            Err(anyhow!("The username must be set"))?;
        }

        if config.password.is_none() && config.password_command.is_none() {
            Err(anyhow!("The password or password command must be set"))?;
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
    pub fn password(&self) -> Result<String> {
        if let Some(ref pass) = self.password {
            Ok(pass.clone())
        } else {
            let command = self.password_command.as_ref().unwrap();

            let args =
                split(command).with_context(|| format!("parsing command failed: {command}"))?;
            let (exe, args) = args.split_first().context("password command is empty")?;
            let output = Command::new(exe)
                .args(args)
                .output()
                .context("password command exec failed")?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

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
    fn test_new_with_args_minimal_config() {
        // Create a minimal args with required fields only
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_string()),
            username: Some("user@example.com".to_string()),
            password: Some("password123".to_string()),
            password_command: None,
            debug: true,
            dry_run: false,
        };

        let config: Config<()> = Config::new_with_args(&args).unwrap();

        assert_eq!(config.server, Some("imap.example.com".to_string()));
        assert_eq!(config.username, Some("user@example.com".to_string()));
        assert_eq!(config.password, Some("password123".to_string()));
        assert!(config.debug);
        assert!(!config.dry_run);
    }

    #[test]
    fn test_new_with_args_missing_server_error() {
        let args = Generic {
            config: None,
            server: None,
            username: Some("user@example.com".to_string()),
            password: Some("password123".to_string()),
            password_command: None,
            debug: false,
            dry_run: false,
        };

        let result: Result<Config<()>> = Config::new_with_args(&args);
        assert!(result.is_err());
        assert_eq!(
            format!("{:?}", result.unwrap_err()),
            "The server must be set"
        );
    }

    #[test]
    fn test_new_with_args_missing_username_error() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_string()),
            username: None,
            password: Some("password123".to_string()),
            password_command: None,
            debug: false,
            dry_run: false,
        };

        let result: Result<Config<()>> = Config::new_with_args(&args);
        assert!(result.is_err());
        assert_eq!(
            format!("{:?}", result.unwrap_err()),
            "The username must be set"
        );
    }

    #[test]
    fn test_password_fn_command_execution() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_string()),
            username: Some("user@example.com".to_string()),
            password: None,
            password_command: Some("echo secret_password".to_string()),
            debug: false,
            dry_run: false,
        };

        let config: Config<()> = Config::new_with_args(&args).unwrap();

        // Mock the command execution with a fake password output
        let password = config.password().unwrap();
        assert_eq!(password.trim(), "secret_password");
    }

    #[test]
    fn test_password_fn_command_cannot_be_parsed() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_string()),
            username: Some("user@example.com".to_string()),
            password: None,
            password_command: Some(r#"echo "secret_password"#.to_string()),
            debug: false,
            dry_run: false,
        };

        let config: Config<()> = Config::new_with_args(&args).unwrap();

        // Mock the command execution with a fake password output
        let result = config.password();
        assert!(result.is_err());
        assert_eq!(
            format!("{:?}", result.unwrap_err()),
            "parsing command failed: echo \"secret_password\n\nCaused by:\n    missing closing quote"
        );
    }

    #[test]
    fn test_password_fn_command_fails() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_string()),
            username: Some("user@example.com".to_string()),
            password: None,
            password_command: Some("exit 1".to_string()),
            debug: false,
            dry_run: false,
        };

        let config: Config<()> = Config::new_with_args(&args).unwrap();

        // Mock the command execution with a fake password output
        let result = config.password();
        assert!(result.is_err());
        assert_eq!(
            format!("{:?}", result.unwrap_err()),
            "password command exec failed\n\nCaused by:\n    No such file or directory (os error 2)"
        );
    }

    #[test]
    fn test_password_fn_command_empty() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_string()),
            username: Some("user@example.com".to_string()),
            password: None,
            password_command: Some(String::new()),
            debug: false,
            dry_run: false,
        };

        let config: Config<()> = Config::new_with_args(&args).unwrap();

        // Mock the command execution with a fake password output
        let result = config.password();
        assert!(result.is_err());
        assert_eq!(
            format!("{:?}", result.unwrap_err()),
            "password command is empty"
        );
    }

    #[test]
    fn test_password_fn_static() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_string()),
            username: Some("user@example.com".to_string()),
            password: Some("secret_password".to_string()),
            password_command: None,
            debug: false,
            dry_run: false,
        };

        let config: Config<()> = Config::new_with_args(&args).unwrap();

        // Mock the command execution with a fake password output
        let password = config.password().unwrap();
        assert_eq!(password.trim(), "secret_password");
    }

    #[test]
    fn test_password_error_when_missing() {
        let args = Generic {
            config: None,
            server: Some("imap.example.com".to_string()),
            username: Some("user@example.com".to_string()),
            password: None,
            password_command: None,
            debug: false,
            dry_run: false,
        };

        let config: Result<Config<()>> = Config::new_with_args(&args);

        assert!(config.is_err());
        assert_eq!(
            format!("{:?}", config.unwrap_err()),
            "The password or password command must be set"
        );
    }

    #[test]
    fn test_config_loading_from_file_bad_config() {
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

        let config: Result<Config<()>> = Config::new_with_args(&args);
        assert!(config.is_err());
        assert_eq!(
            format!("{:?}", config.unwrap_err()),
            "config file parsing failed: TomlDeserialize(Error { inner: ErrorInner { kind: NewlineInString, line: Some(1), col: 34, message: \"\", key: [] } })"
        );
    }

    #[test]
    fn test_config_loading_from_file() {
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

        let config: Config<()> = Config::new_with_args(&args).unwrap();
        assert_eq!(config.server, Some("imap.example.com".to_string()));
        assert_eq!(config.username, Some("user@example.com".to_string()));
        assert_eq!(config.password, Some("password123".to_string()));
        assert!(config.debug);
        assert!(config.dry_run);
    }

    #[test]
    fn test_arg_overrides_file_config() {
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
            server: Some("override.example.com".to_string()),
            username: Some("override_user@example.com".to_string()),
            password: Some("override_password".to_string()),
            password_command: None,
            debug: true,
            dry_run: true,
        };

        let config: Config<()> = Config::new_with_args(&args).unwrap();
        assert_eq!(config.server, Some("override.example.com".to_string()));
        assert_eq!(
            config.username,
            Some("override_user@example.com".to_string())
        );
        assert_eq!(config.password, Some("override_password".to_string()));
        assert!(config.debug);
        assert!(config.dry_run);
    }
}
