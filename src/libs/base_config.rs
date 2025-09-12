use crate::libs::{args::Generic, mode::Mode};
use eyre::{bail, eyre, OptionExt as _, Result, WrapErr as _};
use serde::{Deserialize, Serialize};
use shell_words::split;
use std::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct BaseConfig {
    pub server: Option<String>,

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
                .map_err(|err| eyre!("config file parsing failed: {err:?}"))?
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

        if let Some(ref mode) = args.mode {
            self.mode = Some(mode.clone());
        } else if self.mode.is_none() {
            self.mode = Some(Mode::default());
        } else {
            // vide pour clippy
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
                let args = split(command)
                    .wrap_err_with(|| format!("parsing command failed: {command}"))?;
                let (exe, args) = args.split_first().ok_or_eyre("password command is empty")?;
                let output = Command::new(exe)
                    .args(args)
                    .output()
                    .wrap_err("password command exec failed")?;
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
            _ => bail!("The password or password command must be set"),
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test")]

    use super::*;
    use insta::{assert_debug_snapshot, assert_snapshot};
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
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password: Some("password123".to_owned()),
            debug: true,
            ..Default::default()
        };

        let config: BaseConfig = BaseConfig::new(&args).unwrap();

        if cfg!(any(feature = "rustls", feature = "openssl")) {
            assert_debug_snapshot!(config, @r#"
            BaseConfig {
                server: Some(
                    "imap.example.com",
                ),
                username: Some(
                    "user@example.com",
                ),
                password: Some(
                    "password123",
                ),
                password_command: None,
                mode: Some(
                    Mode(
                        AutoTls,
                    ),
                ),
                debug: true,
                dry_run: false,
            }
            "#);
        } else {
            assert_debug_snapshot!(config, @r#"
            BaseConfig {
                server: Some(
                    "imap.example.com",
                ),
                username: Some(
                    "user@example.com",
                ),
                password: Some(
                    "password123",
                ),
                password_command: None,
                mode: Some(
                    Mode(
                        Plaintext,
                    ),
                ),
                debug: true,
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

        let result: Result<BaseConfig> = BaseConfig::new(&args);
        assert!(result.is_err());
        assert_debug_snapshot!(result, @r#"
        Err(
            "The server must be set",
        )
        "#);
    }

    #[test]
    fn new_with_args_missing_username_error() {
        let args = Generic {
            server: Some("imap.example.com".to_owned()),
            password: Some("password123".to_owned()),
            ..Default::default()
        };

        let result: Result<BaseConfig> = BaseConfig::new(&args);
        assert!(result.is_err());
        assert_debug_snapshot!(result, @r#"
        Err(
            "The username must be set",
        )
        "#);
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
            Error {
                msg: "parsing command failed: echo \"secret_password",
                source: ParseError,
            },
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
        assert_debug_snapshot!(result, @r#"
        Err(
            Error {
                msg: "password command exec failed",
                source: Os {
                    code: 2,
                    kind: NotFound,
                    message: "No such file or directory",
                },
            },
        )
        "#);
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
        assert_debug_snapshot!(result, @r#"
        Err(
            "password command is empty",
        )
        "#);
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

        let config: Result<BaseConfig> = BaseConfig::new(&args);

        assert!(config.is_err());
        assert_debug_snapshot!( config, @r#"
        Err(
            "The password or password command must be set",
        )
        "#);
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

        let config: Result<BaseConfig> = BaseConfig::new(&args);
        assert!(config.is_err());
        assert_debug_snapshot!(
            config,
            @r#"
        Err(
            "config file parsing failed: TomlDeserialize(Error { inner: ErrorInner { kind: NewlineInString, line: Some(1), col: 34, message: \"\", key: [] } })",
        )
        "#
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
        if cfg!(any(feature = "rustls", feature = "openssl")) {
            assert_debug_snapshot!(config, @r#"
            BaseConfig {
                server: Some(
                    "imap.example.com",
                ),
                username: Some(
                    "user@example.com",
                ),
                password: Some(
                    "password123",
                ),
                password_command: None,
                mode: Some(
                    Mode(
                        AutoTls,
                    ),
                ),
                debug: true,
                dry_run: true,
            }
            "#);
        } else {
            assert_debug_snapshot!(config, @r#"
            BaseConfig {
                server: Some(
                    "imap.example.com",
                ),
                username: Some(
                    "user@example.com",
                ),
                password: Some(
                    "password123",
                ),
                password_command: None,
                mode: Some(
                    Mode(
                        Plaintext,
                    ),
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
            debug: true,
            dry_run: true,
            mode: Some(
                if cfg!(any(feature = "rustls", feature = "openssl")) {
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
        if cfg!(any(feature = "rustls", feature = "openssl")) {
            assert_debug_snapshot!(config, @r#"
            BaseConfig {
                server: Some(
                    "override.example.com",
                ),
                username: Some(
                    "override_user@example.com",
                ),
                password: Some(
                    "override_password",
                ),
                password_command: None,
                mode: Some(
                    Mode(
                        Tls,
                    ),
                ),
                debug: true,
                dry_run: true,
            }
            "#);
        } else {
            assert_debug_snapshot!(config, @r#"
            BaseConfig {
                server: Some(
                    "override.example.com",
                ),
                username: Some(
                    "override_user@example.com",
                ),
                password: Some(
                    "override_password",
                ),
                password_command: None,
                mode: Some(
                    Mode(
                        Plaintext,
                    ),
                ),
                debug: true,
                dry_run: true,
            }
            "#);
        }
    }
}
