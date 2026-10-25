use std::{
    fmt::Write as _,
    path::PathBuf,
    process::{Command, ExitStatus},
};

use exn::{OptionExt as _, Result, ResultExt as _, bail};
use rust_i18n::t;
use serde::Deserialize;
use shell_words::split;

use crate::libs::{args::Generic, auth::AuthMethod, mode::Mode, render::RendererArg};

#[derive(Clone, Copy, Debug, derive_more::Display)]
pub enum CommandType {
    #[display("password")]
    Password,
    #[display("oauth2")]
    Oauth2,
}

#[derive(Debug, derive_more::Display)]
pub enum BaseConfigError {
    #[cfg_attr(not(test), display("{}", t!("error.shared.parse_file", file = file : {:?})))]
    #[cfg_attr(test, display("Parsing config file"))]
    ParseFile { file: PathBuf },
    #[display("{}", t!("error.base_config.no_server"))]
    NoServer,
    #[display("{}", t!("error.base_config.no_username"))]
    NoUsername,
    #[display("{}", t!("error.base_config.no_password"))]
    NoPassword,
    #[display("{}", t!("error.base_config.oauth2_password"))]
    Oauth2Password,
    #[display("{}", t!("error.base_config.oauth2_no_command"))]
    Oauth2NoCommand,
    #[display(
        "{}",
        t!("error.base_config.parsing_command", command_type = command_type, command = command)
    )]
    ParsingCommand {
        command_type: CommandType,
        command: String,
    },
    #[display("{}", t!("error.base_config.command_empty", command_type = command_type))]
    CommandEmpty { command_type: CommandType },
    #[display("{}", t!("error.base_config.command_exec", command_type = command_type))]
    CommandExec { command_type: CommandType },
    #[display(
        "{}",
        t!("error.base_config.command_fail", command_type = command_type, command = command : {:?}, status = status, stdout = stdout, stderr = stderr)
    )]
    CommandFail {
        command_type: CommandType,
        command: String,
        status: ExitStatus,
        stdout: String,
        stderr: String,
    },
    #[display("{}", t!("error.base_config.password_command_output", command_type = command_type))]
    PasswordCommandOutput { command_type: CommandType },
}
impl std::error::Error for BaseConfigError {}

#[derive(Clone, Debug, Default, Deserialize)]
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

    #[serde(default, rename = "debug")]
    pub _debug: bool,

    #[serde(default)]
    pub dry_run: bool,

    #[serde(default)]
    pub auth: Option<AuthMethod>,

    pub(self) oauth2_command: Option<String>,
}

#[derive(Debug, derive_more::Display)]
pub struct SerdeAnyWrapper(pub serde_any::Error);
impl std::error::Error for SerdeAnyWrapper {}

impl BaseConfig {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(args), err(level = "info"))
    )]
    /// Create the config from a file and arguments.
    /// # Errors
    /// Many errors can happen
    pub fn new(args: &Generic) -> Result<Self, BaseConfigError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(?args);

        let config = if let Some(ref config) = args.config {
            serde_any::from_file(config)
                .map_err(SerdeAnyWrapper)
                .or_raise(|| BaseConfigError::ParseFile {
                    file: config.clone(),
                })?
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
        self.server = args.server.clone().or(self.server);
        self.port = args.port.or(self.port);
        self.username = args.username.clone().or(self.username);
        self.password = args.password.clone().or(self.password);
        self.password_command = args.password_command.clone().or(self.password_command);
        self.dry_run |= args.dry_run;
        self.mode = Some(args.mode.clone().or(self.mode).unwrap_or_default());
        self.auth = args.auth.or(self.auth);
        self.oauth2_command = args.oauth2_command.clone().or(self.oauth2_command);
        self.renderer = args.renderer.or(self.renderer);

        if self.server.is_none() {
            bail!(BaseConfigError::NoServer);
        }

        if self.username.is_none() {
            bail!(BaseConfigError::NoUsername);
        }

        let auth = self.auth.unwrap_or_default();
        if auth.requires_password() {
            if self.password.is_none() && self.password_command.is_none() {
                bail!(BaseConfigError::NoPassword);
            }
        } else {
            // XOAuth2
            if self.password.is_some() || self.password_command.is_some() {
                bail!(BaseConfigError::Oauth2Password);
            }
            if self.oauth2_command.is_none() {
                bail!(BaseConfigError::Oauth2NoCommand);
            }
        }

        Ok(self)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), ret, err(level = "info"))
    )]
    /// Get the password from the literal value or from the command.
    /// # Errors
    /// Many errors can happen
    pub fn password(&self) -> Result<String, BaseConfigError> {
        #[expect(clippy::needless_borrowed_reference, reason = "ok")]
        match (&self.password, &self.password_command) {
            (&Some(ref pass), _) => Ok(pass.clone()),
            (_, &Some(ref command)) => run_command(command, CommandType::Password),
            _ => bail!(BaseConfigError::NoPassword),
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    /// Run `oauth2-command` and return its stdout as the `OAuth2` access token.
    ///
    /// # Errors
    /// Returns an error if the command is missing, does not parse or fails to run.
    /// It also returns an error if the output is not UTF-8.
    pub fn oauth2_token(&self) -> Result<String, BaseConfigError> {
        match self.oauth2_command {
            None => bail!(BaseConfigError::Oauth2NoCommand),
            Some(ref command) => run_command(command, CommandType::Oauth2),
        }
    }
}

/// Run `command` and return its stdout without the trailing newline.
fn run_command(command: &str, command_type: CommandType) -> Result<String, BaseConfigError> {
    let command_vec = split(command).or_raise(|| BaseConfigError::ParsingCommand {
        command_type,
        command: command.to_owned(),
    })?;
    let (exe, args) = command_vec
        .split_first()
        .ok_or_raise(|| BaseConfigError::CommandEmpty { command_type })?;
    let output = Command::new(exe)
        .args(args)
        .output()
        .or_raise(|| BaseConfigError::CommandExec { command_type })?;

    if !output.status.success() {
        bail!(BaseConfigError::CommandFail {
            command_type,
            command: command.to_owned(),
            status: output.status,
            stdout: indent_output("STDOUT: ", &output.stdout),
            stderr: indent_output("STDERR: ", &output.stderr),
        })
    }

    let mut out = String::from_utf8(output.stdout)
        .or_raise(|| BaseConfigError::PasswordCommandOutput { command_type })?;

    // Remove the trailing newline that most commands add
    if out.ends_with('\n') {
        out.pop();
        if out.ends_with('\r') {
            out.pop();
        }
    }
    Ok(out)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", skip(prefix, bytes))
)]
fn indent_output(prefix: &str, bytes: &[u8]) -> String {
    let s = String::from_utf8_lossy(bytes);
    let indent = " ".repeat(prefix.len());
    let mut lines = s.lines();
    lines.next().map_or_else(
        || format!("{prefix}{}", t!("output.empty")),
        |first| {
            let rest = lines.fold(String::new(), |mut output, l| {
                let _ = write!(output, "\n{indent}{l}");
                output
            });
            format!("{prefix}{first}{rest}")
        },
    )
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "test")]

    use std::{fs::File, io::Write as _};

    use insta::{assert_debug_snapshot, assert_snapshot};

    use super::*;

    // Helper that creates temporary config files with the given content.
    // It must also return the directory. Otherwise the directory goes out of scope
    // and the system deletes it.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(content), ret)
    )]
    fn write_temp_config(content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let temp_dir = tempfile::tempdir().expect("should work");
        let temp_file_path = temp_dir.path().join("config.toml");
        let mut file = File::create(&temp_file_path).expect("file creation should work");
        file.write_all(content.as_bytes())
            .expect("writing should work");
        (temp_dir, temp_file_path)
    }

    #[test]
    fn new_with_args_minimal_config() {
        // Create minimal args that have only the required fields
        let args = Generic {
            server: Some("imap.example.com".to_owned()),
            username: Some("user@example.com".to_owned()),
            password: Some("password123".to_owned()),
            ..Default::default()
        };

        let config: BaseConfig =
            BaseConfig::new(&args).expect("minimal config args should produce valid config");

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
                _debug: false,
                dry_run: false,
                auth: None,
                oauth2_command: None,
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
                _debug: false,
                dry_run: false,
                auth: None,
                oauth2_command: None,
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
            The server must be set, at src/libs/base_config.rs:142:13,
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
            The username must be set, at src/libs/base_config.rs:146:13,
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

        let config: BaseConfig =
            BaseConfig::new(&args).expect("config with password-command should be constructable");

        // Mock the command execution. The mock returns a fake password.
        let password = config
            .password()
            .expect("password-command should execute successfully");
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

        let config: BaseConfig = BaseConfig::new(&args)
            .expect("config with unparseable command should be constructable");

        // Mock the command execution. The mock returns a fake password.
        let result = config.password();
        assert!(result.is_err());
        assert_debug_snapshot!(result, @r#"
        Err(
            Parsing password command echo "secret_password, at src/libs/base_config.rs:202:38
            `-- missing closing quote, at src/libs/base_config.rs:202:38,
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

        let config: BaseConfig =
            BaseConfig::new(&args).expect("config with failing command should be constructable");

        // Mock the command execution. The mock returns a fake password.
        let result = config.password();
        assert!(result.is_err());
        assert_debug_snapshot!(result, @"
        Err(
            Executing password command, at src/libs/base_config.rs:212:10
            `-- No such file or directory (os error 2), at src/libs/base_config.rs:212:10,
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

        let config: BaseConfig =
            BaseConfig::new(&args).expect("config with empty command should be constructable");

        // Mock the command execution. The mock returns a fake password.
        let result = config.password();
        assert!(result.is_err());
        assert_debug_snapshot!(result, @"
        Err(
            password command is empty, at src/libs/base_config.rs:208:10,
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

        let config: BaseConfig =
            BaseConfig::new(&args).expect("config with static password should be valid");

        // Mock the command execution. The mock returns a fake password.
        let password = config
            .password()
            .expect("static password should be returned");
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
            The password or password command must be set, at src/libs/base_config.rs:152:17,
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
            Parsing config file, at src/libs/base_config.rs:115:18
            `-- TOML deserialize error: newline in string found at line 2, at src/libs/base_config.rs:115:18,
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

        let config: BaseConfig =
            BaseConfig::new(&args).expect("config from valid file should parse");
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
                _debug: true,
                dry_run: true,
                auth: None,
                oauth2_command: None,
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
                _debug: true,
                dry_run: true,
                auth: None,
                oauth2_command: None,
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
                .expect("'tls' and 'plaintext' are valid TlsMode strings"),
            ),
            ..Default::default()
        };

        let config: BaseConfig =
            BaseConfig::new(&args).expect("config with CLI arg overrides should be valid");
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
                _debug: false,
                dry_run: true,
                auth: None,
                oauth2_command: None,
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
                _debug: false,
                dry_run: true,
                auth: None,
                oauth2_command: None,
            }
            "#);
        }
    }
}
