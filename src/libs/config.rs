use crate::libs::{args::Generic, error::OurResult, filters::Filters};
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
    pub fn new_with_args(args: &Generic) -> OurResult<Self> {
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
            Err("The server must be set")?;
        }

        if config.username.is_none() {
            Err("The username must be set")?;
        }

        if config.password.is_none() && config.password_command.is_none() {
            Err("The password or password command must be set")?;
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
    pub fn password(&self) -> OurResult<String> {
        if let Some(ref pass) = self.password {
            Ok(pass.clone())
        } else if let Some(ref command) = self.password_command {
            let args = split(command)?;
            let (exe, args) = args.split_first().ok_or("password command is empty")?;
            let output = Command::new(exe).args(args).output()?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err("the password or password command must be set")?
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

        let result: OurResult<Config<()>> = Config::new_with_args(&args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Configuration error: The server must be set"
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

        let result: OurResult<Config<()>> = Config::new_with_args(&args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Configuration error: The username must be set"
        );
    }

    #[test]
    fn test_password_command_execution() {
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

        let config: OurResult<Config<()>> = Config::new_with_args(&args);

        assert!(config.is_err());
        assert_eq!(
            config.unwrap_err().to_string(),
            "Configuration error: The password or password command must be set"
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
