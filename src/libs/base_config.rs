use std::process::Command;

use derive_more::Display;
use exn::{OptionExt as _, Result, ResultExt as _, bail};
use serde::{Deserialize, Serialize};
use shell_words::split;

use crate::libs::{args::Generic, mode::Mode, render::RendererArg};

#[derive(Debug, Display)]
pub struct BaseConfigError(String);
impl std::error::Error for BaseConfigError {}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct BaseConfig {
    pub renderer: Option<RendererArg>,

    pub server: Option<String>,

    #[serde(default)]
    pub port: Option<u16>,

    pub username: Option<String>,

    pub(self) password: Option<String>,

    pub(self) password_command: Option<String>,

    #[serde(default)]
    pub mode: Option<Mode>,

    #[serde(default)]
    pub debug: bool,

    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Display)]
pub struct SerdeAnyWrapper(pub serde_any::Error);
impl std::error::Error for SerdeAnyWrapper {}

impl BaseConfig {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(args), err(level = "info"))
    )]
    /// Creates from a file and arguments
    /// # Errors
    /// Many errors can happen
    pub fn new(args: &Generic) -> Result<Self, BaseConfigError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(?args);

        let config = if let Some(ref config) = args.config {
            serde_any::from_file(config)
                .map_err(SerdeAnyWrapper)
                .or_raise(|| BaseConfigError("config file parsing failed".to_owned()))?
        } else {
            Self::default()
        };

        config.apply_args(args)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self, args), ret, err(level = "info"))
    )]
    pub fn apply_args(mut self, args: &Generic) -> Result<Self, BaseConfigError> {
        if let Some(ref server) = args.server {
            self.server = Some(server.clone());
        }

        if let Some(port) = args.port {
            self.port = Some(port);
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

        if args.dry_run {
            self.dry_run = args.dry_run;
        }

        if args.mode.is_some() {
            self.mode.clone_from(&args.mode);
        } else {
            self.mode.get_or_insert_with(Mode::default);
        }

        if self.server.is_none() {
            bail!(BaseConfigError("The server must be set".to_owned()));
        }

        if self.username.is_none() {
            bail!(BaseConfigError("The username must be set".to_owned()));
        }

        if self.password.is_none() && self.password_command.is_none() {
            bail!(BaseConfigError(
                "The password or password command must be set".to_owned()
            ));
        }

        if let Some(renderer) = args.renderer {
            self.renderer = Some(renderer);
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
    pub fn password(&self) -> Result<String, BaseConfigError> {
        #[expect(clippy::needless_borrowed_reference, reason = "ok")]
        match (&self.password, &self.password_command) {
            (&Some(ref pass), _) => Ok(pass.clone()),
            (_, &Some(ref command)) => {
                let args = split(command)
                    .or_raise(|| BaseConfigError(format!("parsing command failed: {command}")))?;
                let (exe, args) = args
                    .split_first()
                    .ok_or_raise(|| BaseConfigError("password command is empty".to_owned()))?;
                let output = Command::new(exe)
                    .args(args)
                    .output()
                    .or_raise(|| BaseConfigError("password command exec failed".to_owned()))?;
                let mut password = String::from_utf8(output.stdout).or_raise(|| {
                    BaseConfigError("password command output is not valid UTF-8".to_owned())
                })?;
                // Strip trailing newline added by most password commands
                if password.ends_with('\n') {
                    password.pop();
                    if password.ends_with('\r') {
                        password.pop();
                    }
                }
                Ok(password)
            },
            _ => bail!(BaseConfigError(
                "The password or password command must be set".to_owned()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test")]

    use std::{fs::File, io::Write as _};

    use insta::{assert_debug_snapshot, assert_snapshot};

    use super::*;

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
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password: Some("password123".to_owned()),
            ..Default::default()
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();

        if cfg!(feature = "__tls") {
            assert_debug_snapshot!(config, @r#"
            BaseConfig {
                renderer: None,
                server: Some(
                    "imap.example.com",
                ),
                port: None,
                username: Some(
                    "user@example.com",
                ),
                password: Some(
                    "password123",
                ),
                password_command: None,
                mode: Some(
                    AutoTls,
                ),
                debug: false,
                dry_run: false,
            }
            "#);
        } else {
            assert_debug_snapshot!(config, @r#"
            BaseConfig {
                renderer: None,
                server: Some(
                    "imap.example.com",
                ),
                port: None,
                username: Some(
                    "user@example.com",
                ),
                password: Some(
                    "password123",
                ),
                password_command: None,
                mode: Some(
                    Plaintext,
                ),
                debug: false,
                dry_run: false,
            }
            "#);
        }
    }

    #[test]
    fn new_with_args_missing_server_error() {
        let args = Generic {
            username: Some("user@example.com".to_owned()),
            password: Some("password123".to_owned()),
            ..Default::default()
        };

        let result: Result<BaseConfig, BaseConfigError> = BaseConfig::new(&args);
        assert!(result.is_err());
        assert_debug_snapshot!(result, @"
        Err(
            The server must be set, at src/libs/base_config.rs:103:13,
        )
        ");
    }

    #[test]
    fn new_with_args_missing_username_error() {
        let args = Generic {
            server: Some("imap.example.com".to_owned()),
            password: Some("password123".to_owned()),
            ..Default::default()
        };

        let result: Result<BaseConfig, BaseConfigError> = BaseConfig::new(&args);
        assert!(result.is_err());
        assert_debug_snapshot!(result, @"
        Err(
            The username must be set, at src/libs/base_config.rs:107:13,
        )
        ");
    }

    #[test]
    fn password_fn_command_execution() {
        let args = Generic {
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password_command: Some("echo secret_password".to_owned()),
            ..Default::default()
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let password = config.password().unwrap();
        assert_snapshot!(password.trim(), @"secret_password");
    }

    #[test]
    fn password_fn_command_cannot_be_parsed() {
        let args = Generic {
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password_command: Some(r#"echo "secret_password"#.to_owned()),
            ..Default::default()
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let result = config.password();
        assert!(result.is_err());
        assert_debug_snapshot!(result, @r#"
        Err(
            parsing command failed: echo "secret_password, at src/libs/base_config.rs:136:22
            |
            |-> missing closing quote, at src/libs/base_config.rs:136:22,
        )
        "#);
    }

    #[test]
    fn password_fn_command_fails() {
        let args = Generic {
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password_command: Some("exit 1".to_owned()),
            ..Default::default()
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let result = config.password();
        assert!(result.is_err());
        assert_debug_snapshot!(result, @"
        Err(
            password command exec failed, at src/libs/base_config.rs:143:22
            |
            |-> No such file or directory (os error 2), at src/libs/base_config.rs:143:22,
        )
        ");
    }

    #[test]
    fn password_fn_command_empty() {
        let args = Generic {
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password_command: Some(String::new()),
            ..Default::default()
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let result = config.password();
        assert!(result.is_err());
        assert_debug_snapshot!(result, @"
        Err(
            password command is empty, at src/libs/base_config.rs:139:22,
        )
        ");
    }

    #[test]
    fn password_fn_static() {
        let args = Generic {
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password: Some("secret_password".to_owned()),
            ..Default::default()
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let password = config.password().unwrap();
        assert_snapshot!(password.trim(), @"secret_password");
    }

    #[test]
    fn password_error_when_missing() {
        let args = Generic {
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            ..Default::default()
        };

        let config: Result<BaseConfig, BaseConfigError> = BaseConfig::new(&args);

        assert!(config.is_err());
        assert_debug_snapshot!( config, @"
        Err(
            The password or password command must be set, at src/libs/base_config.rs:111:13,
        )
        ");
    }

    #[test]
    fn config_loading_from_file_bad_config() {
        let config_content = r#"
        server = "imap.example.com
    "#;

        let (_temp_dir, config_path) = write_temp_config(config_content);

        let args = Generic {
            config: Some(config_path),
            ..Default::default()
        };

        let config: Result<BaseConfig, BaseConfigError> = BaseConfig::new(&args);
        assert!(config.is_err());
        assert_debug_snapshot!(
            config,
            @"
        Err(
            config file parsing failed, at src/libs/base_config.rs:59:18
            |
            |-> TOML deserialize error: newline in string found at line 2, at src/libs/base_config.rs:59:18,
        )
        "
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
            ..Default::default()
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();
        if cfg!(feature = "__tls") {
            assert_debug_snapshot!(config, @r#"
            BaseConfig {
                renderer: None,
                server: Some(
                    "imap.example.com",
                ),
                port: None,
                username: Some(
                    "user@example.com",
                ),
                password: Some(
                    "password123",
                ),
                password_command: None,
                mode: Some(
                    AutoTls,
                ),
                debug: true,
                dry_run: true,
            }
            "#);
        } else {
            assert_debug_snapshot!(config, @r#"
            BaseConfig {
                renderer: None,
                server: Some(
                    "imap.example.com",
                ),
                port: None,
                username: Some(
                    "user@example.com",
                ),
                password: Some(
                    "password123",
                ),
                password_command: None,
                mode: Some(
                    Plaintext,
                ),
                debug: true,
                dry_run: true,
            }
            "#);
        }
    }

    #[test]
    fn arg_overrides_file_config() {
        let config_content = r#"
        server = "imap.example.com"
        username = "user@example.com"
        password = "password123"
        debug = false
        dry-run = false
        mode = "none"
    "#;

        let (_temp_dir, config_path) = write_temp_config(config_content);
        let args = Generic {
            config: Some(config_path),
            server: Some("override.example.com".to_owned()),
            username: Some("override_user@example.com".to_owned()),
            password: Some("override_password".to_owned()),
            dry_run: true,
            mode: Some(
                if cfg!(feature = "__tls") {
                    "tls"
                } else {
                    "plaintext"
                }
                .parse()
                .unwrap(),
            ),
            ..Default::default()
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();
        if cfg!(feature = "__tls") {
            assert_debug_snapshot!(config, @r#"
            BaseConfig {
                renderer: None,
                server: Some(
                    "override.example.com",
                ),
                port: None,
                username: Some(
                    "override_user@example.com",
                ),
                password: Some(
                    "override_password",
                ),
                password_command: None,
                mode: Some(
                    Tls,
                ),
                debug: false,
                dry_run: true,
            }
            "#);
        } else {
            assert_debug_snapshot!(config, @r#"
            BaseConfig {
                renderer: None,
                server: Some(
                    "override.example.com",
                ),
                port: None,
                username: Some(
                    "override_user@example.com",
                ),
                password: Some(
                    "override_password",
                ),
                password_command: None,
                mode: Some(
                    Plaintext,
                ),
                debug: false,
                dry_run: true,
            }
            "#);
        }
    }
}
