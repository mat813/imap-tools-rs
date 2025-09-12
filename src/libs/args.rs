use crate::libs::mode::Mode;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug, Clone, Default)]
pub struct Generic {
    /// Path to the configuration file.
    #[arg(short = 'c', long, default_value = ".imap-tools.toml")]
    pub config: Option<PathBuf>,

    /// The server to connect to.
    #[arg(short = 's', long)]
    pub server: Option<String>,

    /// The username to use for the connection.
    #[arg(short = 'u', long)]
    pub username: Option<String>,

    /// The password to use for the connection.
    #[arg(short = 'p', long)]
    pub password: Option<String>,

    /// The command to use to get the password.
    #[arg(short = 'P', long)]
    pub password_command: Option<String>,

    /// Display all the IMAP commands sent and received.
    #[arg(short = 'd', long)]
    pub debug: bool,

    #[cfg(any(feature = "rustls", feature = "openssl"))]
    /// Select the TLS mode
    ///
    /// `auto_tls`: Automatically detect what connection mode should be used.
    ///   This will use TLS if the port is 993, and otherwise STARTTLS if available.
    ///   If no TLS communication mechanism is available, the connection will fail.
    ///
    /// `auto`: Automatically detect what connection mode should be used.
    ///   This will use TLS if the port is 993, and otherwise STARTTLS if available.
    ///   It will fallback to a plaintext connection if no TLS option can be used.
    ///
    /// `plaintext`: A plain unencrypted TCP connection
    ///
    /// `tls`: An encrypted TLS connection (needs feature rustls or openssl)
    ///
    /// `start_tls`: An eventually-encrypted (i.e., STARTTLS) connection (needs feature rustls or openssl)
    #[arg(short = 'm', long)]
    pub mode: Option<Mode>,

    #[cfg(not(any(feature = "rustls", feature = "openssl")))]
    /// TLS is disabled, recompile with either feature rustls or openssl.
    ///
    /// The only accepted value is `plaintext`, which is no encryption
    #[arg(short = 'm', long)]
    pub mode: Option<Mode>,

    /// Do not actually do any changes to the server.
    #[arg(short = 'n', long)]
    pub dry_run: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

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
    fn debug_flag() {
        let generic = get_generic_from_args(["test", "-d"]);
        assert!(generic.debug);
    }

    #[test]
    fn dry_run_flag() {
        let generic = get_generic_from_args(["test", "-n"]);
        assert!(generic.dry_run);
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
            "-d",
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
        assert!(generic.debug);
        assert!(generic.dry_run);
    }
}
