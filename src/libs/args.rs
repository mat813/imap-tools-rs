use std::path::PathBuf;

use clap::Args;

use crate::libs::{mode::Mode, render::RendererArg};

#[derive(Args, Debug, Clone, Default)]
pub struct Generic {
    /// Path to the configuration file.
    #[arg(short = 'c', long, default_value = ".imap-tools.toml")]
    pub config: Option<PathBuf>,

    /// The server to connect to.
    #[arg(short = 's', long)]
    pub server: Option<String>,

    /// The port to connect to (default: 143).
    #[arg(long)]
    pub port: Option<u16>,

    /// The username to use for the connection.
    #[arg(short = 'u', long)]
    pub username: Option<String>,

    /// The password to use for the connection.
    #[arg(short = 'p', long)]
    pub password: Option<String>,

    /// The command to use to get the password.
    #[arg(short = 'P', long)]
    pub password_command: Option<String>,

    #[cfg_attr(
        any(feature = "rustls", feature = "openssl"),
        doc = "Select the TLS mode"
    )]
    #[cfg_attr(
        not(any(feature = "rustls", feature = "openssl")),
        doc = "TLS is disabled, recompile with either feature rustls or openssl."
    )]
    #[arg(short = 'm', long, value_enum)]
    pub mode: Option<Mode>,

    /// Which renderer to use.
    #[arg(long, env = "RENDERER", value_enum)]
    pub renderer: Option<RendererArg>,

    /// Do not actually do any changes to the server.
    #[arg(short = 'n', long)]
    pub dry_run: bool,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[derive(Parser, Debug, Clone)]
    struct Args {
        #[clap(flatten)]
        generic: Generic,
    }

    fn get_generic_from_args<I, T>(args: I) -> Generic
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Args::parse_from(args).generic
    }

    #[test]
    fn default_config_path() {
        // No arguments, config should use default value
        let generic = get_generic_from_args(["test"]);
        assert_eq!(generic.config, Some(PathBuf::from(".imap-tools.toml")));
    }

    #[test]
    fn custom_config_path() {
        let generic = get_generic_from_args(["test", "-c", "custom-config.toml"]);
        assert_eq!(generic.config, Some(PathBuf::from("custom-config.toml")));
    }

    #[test]
    fn server_option() {
        let generic = get_generic_from_args(["test", "-s", "imap.example.com"]);
        assert_eq!(generic.server, Some("imap.example.com".to_owned()));
    }

    #[test]
    fn username_option() {
        let generic = get_generic_from_args(["test", "-u", "user@example.com"]);
        assert_eq!(generic.username, Some("user@example.com".to_owned()));
    }

    #[test]
    fn password_option() {
        let generic = get_generic_from_args(["test", "-p", "secret_password"]);
        assert_eq!(generic.password, Some("secret_password".to_owned()));
    }

    #[test]
    fn password_command_option() {
        let generic = get_generic_from_args(["test", "-P", "echo secret_password"]);
        assert_eq!(
            generic.password_command,
            Some("echo secret_password".to_owned())
        );
    }

    #[test]
    fn dry_run_flag() {
        let generic = get_generic_from_args(["test", "-n"]);
        assert!(generic.dry_run);
    }

    #[test]
    fn port_option() {
        let generic = get_generic_from_args(["test", "--port", "993"]);
        assert_eq!(generic.port, Some(993));
    }

    #[test]
    fn mode_option() {
        let generic = get_generic_from_args(["test", "-m", "auto"]);
        assert!(generic.mode.is_some(), "mode should be set");
    }

    #[cfg(any(feature = "rustls", feature = "openssl"))]
    #[test]
    fn mode_tls_option() {
        let generic = get_generic_from_args(["test", "-m", "tls"]);
        assert!(generic.mode.is_some(), "mode should be set");
    }

    #[test]
    fn combined_options() {
        let generic = get_generic_from_args([
            "test",
            "-c",
            "custom-config.toml",
            "-s",
            "imap.example.com",
            "-u",
            "user@example.com",
            "-p",
            "secret_password",
            "-P",
            "echo secret_password",
            "-n",
        ]);

        assert_eq!(generic.config, Some(PathBuf::from("custom-config.toml")));
        assert_eq!(generic.server, Some("imap.example.com".to_owned()));
        assert_eq!(generic.username, Some("user@example.com".to_owned()));
        assert_eq!(generic.password, Some("secret_password".to_owned()));
        assert_eq!(
            generic.password_command,
            Some("echo secret_password".to_owned())
        );
        assert!(generic.dry_run);
    }
}
