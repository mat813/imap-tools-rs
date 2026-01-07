use crate::libs::{
    args::Generic,
    base_config::{BaseConfig, SerdeAnyWrapper},
    filters::Filters,
};
use derive_more::Display;
use exn::{Result, ResultExt as _};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Display)]
pub struct ConfigError(&'static str);

impl std::error::Error for ConfigError {}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config<T>
where
    T: Clone + Debug + Serialize,
{
    #[serde(flatten)]
    pub base: BaseConfig,

    pub extra: Option<T>,

    pub filters: Option<Filters<T>>,
}

impl<T> Default for Config<T>
where
    T: Clone + Debug + Serialize,
{
    fn default() -> Self {
        Self {
            base: BaseConfig::default(),
            extra: None,
            filters: None,
        }
    }
}

impl<T> Config<T>
where
    T: Clone + Debug + Serialize + for<'de> Deserialize<'de>,
{
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(args), ret, err(level = "info"))
    )]
    /// Creates from a file and arguments
    /// # Errors
    /// Many errors can happen
    pub fn new(args: &Generic) -> Result<Self, ConfigError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(?args);

        let mut config = if let Some(ref config) = args.config {
            serde_any::from_file(config)
                .map_err(SerdeAnyWrapper)
                .or_raise(|| ConfigError("config file parsing failed"))?
        } else {
            Self::default()
        };

        config.base = config
            .base
            .apply_args(args)
            .or_raise(|| ConfigError("base"))?;

        Ok(config)
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

        let config: Config<()> = Config::new(&args).unwrap();

        if cfg!(any(feature = "rustls", feature = "openssl")) {
            assert_debug_snapshot!(config, @r#"
            Config {
                base: BaseConfig {
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
                },
                extra: None,
                filters: None,
            }
            "#);
        } else {
            assert_debug_snapshot!(config, @r#"
            Config {
                base: BaseConfig {
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
                },
                extra: None,
                filters: None,
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

        let result: Result<Config<()>, ConfigError> = Config::new(&args);
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

        let result: Result<Config<()>, ConfigError> = Config::new(&args);
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

        let config: Config<()> = Config::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let password = config.base.password().unwrap();
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

        let config: Config<()> = Config::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let result = config.base.password();
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

        let config: Config<()> = Config::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let result = config.base.password();
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

        let config: Config<()> = Config::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let result = config.base.password();
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

        let config: Config<()> = Config::new(&args).unwrap();

        // Mock the command execution with a fake password output
        let password = config.base.password().unwrap();
        assert_snapshot!(password.trim(), @"secret_password");
    }

    #[test]
    fn password_error_when_missing() {
        let args = Generic {
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            ..Default::default()
        };

        let config: Result<Config<()>, ConfigError> = Config::new(&args);

        assert!(config.is_err());
        assert_debug_snapshot!(config, @r#"
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

        let config: Result<Config<()>, ConfigError> = Config::new(&args);
        assert!(config.is_err());
        assert_debug_snapshot!(config, @r#"
        Err(
            "config file parsing failed: TomlDeserialize(Error { inner: ErrorInner { kind: NewlineInString, line: Some(1), col: 34, message: \"\", key: [] } })",
        )
        "#);
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

        let config: Config<()> = Config::new(&args).unwrap();
        if cfg!(any(feature = "rustls", feature = "openssl")) {
            assert_debug_snapshot!(config, @r#"
            Config {
                base: BaseConfig {
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
                },
                extra: None,
                filters: None,
            }
            "#);
        } else {
            assert_debug_snapshot!(config, @r#"
            Config {
                base: BaseConfig {
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
                },
                extra: None,
                filters: None,
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

        let config: Config<()> = Config::new(&args).unwrap();
        if cfg!(any(feature = "rustls", feature = "openssl")) {
            assert_debug_snapshot!(config, @r#"
            Config {
                base: BaseConfig {
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
                },
                extra: None,
                filters: None,
            }
            "#);
        } else {
            assert_debug_snapshot!(config, @r#"
            Config {
                base: BaseConfig {
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
                },
                extra: None,
                filters: None,
            }
            "#);
        }
    }
}
