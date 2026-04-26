#[cfg(all(feature = "native-tls", feature = "rustls"))]
compile_error!("features `openssl` and `rustls` are mutually exclusive — enable only one");

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Debug,
};

use async_imap::{Session, imap_proto::NameAttribute, types::Uid};
use derive_more::Display;
use exn::{OptionExt as _, Result, ResultExt as _, bail};
use futures::TryStreamExt as _;
use serde::Serialize;
use tokio::net::TcpStream;

use crate::libs::{
    base_config::BaseConfig, config::Config, filter::Filter, filters::Filters, mode::Mode,
};

/// Marker trait for streams usable with async-imap.
pub trait AsyncStream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + Debug {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + Debug> AsyncStream for T {}

/// Boxed async stream type alias.
pub type ImapStream = Box<dyn AsyncStream>;

#[derive(Debug, Display)]
/// Error type for IMAP operations.
pub struct ImapError(String);
impl std::error::Error for ImapError {}

/// The result of listing a mailbox, including optional command-specific extra data.
#[derive(Clone, Debug)]
pub struct ListResult<T>
where
    T: Clone + Debug,
{
    /// Optional extra data associated with the mailbox, from matching filter config.
    pub extra: Option<T>,
}

/// Wraps an async-imap Session with connection state and filter configuration.
#[derive(Debug)]
pub struct Imap<T>
where
    T: Clone + Debug + Serialize,
{
    /// The underlying async-imap session.
    pub session: Session<ImapStream>,

    /// Optional command-specific extra data from configuration.
    extra: Option<T>,

    /// Optional list of filters to apply when listing mailboxes.
    filters: Option<Filters<T>>,

    /// Cache of previously fetched capabilities to avoid redundant round trips.
    cached_capabilities: HashMap<String, bool>,

    /// Whether the session has been explicitly closed.
    closed: bool,
}

impl<T> Drop for Imap<T>
where
    T: Clone + Debug + Serialize,
{
    fn drop(&mut self) {
        if !self.closed {
            #[cfg(feature = "tracing")]
            tracing::warn!(
                "Imap dropped without explicit close(); connection torn down by TCP RST"
            );
        }
    }
}

impl<T> Imap<T>
where
    T: Clone + Debug + Serialize,
{
    /// Explicitly close the IMAP session by sending LOGOUT.
    ///
    /// # Errors
    /// Returns an error if the LOGOUT command fails.
    pub async fn close(mut self) -> Result<(), ImapError> {
        self.closed = true;
        self.session
            .logout()
            .await
            .or_raise(|| ImapError("imap logout failed".to_owned()))
    }

    /// Connect and login to the IMAP server described by `base`.
    ///
    /// # Errors
    /// Returns an error if the connection, TLS setup, or login fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(base), ret, err(level = "info"))
    )]
    pub async fn connect_base(base: &BaseConfig) -> Result<Self, ImapError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(?base);

        let server = base
            .server
            .as_ref()
            .ok_or_raise(|| ImapError("Missing server".to_owned()))?;

        let port = base.port.unwrap_or(143);
        let mode = base.mode.clone().unwrap_or_default();

        let tcp = TcpStream::connect((server.as_str(), port))
            .await
            .or_raise(|| ImapError(format!("failed to connect to {server} on port {port}")))?;

        let (stream, greeting_consumed): (ImapStream, bool) =
            build_stream(tcp, &mode, server, port)
                .await
                .or_raise(|| ImapError("TLS setup failed".to_owned()))?;

        let mut client = async_imap::Client::new(stream);

        if !greeting_consumed {
            client
                .read_response()
                .await
                .or_raise(|| ImapError("failed to read server greeting".to_owned()))?;
        }

        let username = base
            .username
            .as_ref()
            .ok_or_raise(|| ImapError("Missing username".to_owned()))?;
        let password = base
            .password()
            .or_raise(|| ImapError("Password error".to_owned()))?;

        let session = client
            .login(username, password)
            .await
            .map_err(|(err, _client)| err)
            .or_raise(|| ImapError("imap login failed".to_owned()))?;

        let mut ret = Self {
            session,
            extra: None,
            filters: None,
            cached_capabilities: HashMap::new(),
            closed: false,
        };

        if !ret.has_capability("UIDPLUS").await? {
            bail!(ImapError("The server does not support the UIDPLUS capability, and all our operations need UIDs for safety".to_owned()));
        }

        Ok(ret)
    }

    /// Test-only: connect to a specific port in plaintext mode (no TLS).
    /// Useful for connecting to a mock IMAP server.
    #[cfg(test)]
    pub async fn connect_base_on_port(base: &BaseConfig, port: u16) -> Result<Self, ImapError> {
        let mut test_base = base.clone();
        test_base.port = Some(port);
        #[expect(clippy::expect_used, reason = "known-valid literal")]
        {
            test_base.mode = Some("plaintext".parse().expect("plaintext is a valid mode"));
        }
        Self::connect_base(&test_base).await
    }

    /// Connect to the server and login with the given credentials.
    ///
    /// # Errors
    /// Many errors can happen
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(config), ret, err(level = "info"))
    )]
    pub async fn connect(config: &Config<T>) -> Result<Self, ImapError> {
        let mut ret = Self::connect_base(&config.base).await?;

        ret.extra.clone_from(&config.extra);
        ret.filters.clone_from(&config.filters);

        Ok(ret)
    }

    /// Check if the imap server has some capability.
    ///
    /// # Errors
    /// Imap errors can happen
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), ret, err(level = "info"))
    )]
    pub async fn has_capability<S: AsRef<str> + Debug>(
        &mut self,
        cap: S,
    ) -> Result<bool, ImapError> {
        if let Some(&cached_result) = self.cached_capabilities.get(cap.as_ref()) {
            return Ok(cached_result);
        }

        let has_capability = self
            .session
            .capabilities()
            .await
            .or_raise(|| ImapError("imap capabilities failed".to_owned()))?
            .has_str(cap.as_ref());

        self.cached_capabilities
            .insert(cap.as_ref().to_owned(), has_capability);

        Ok(has_capability)
    }

    /// Select a mailbox, flag the given UID sequence as `\Deleted`, then CLOSE
    /// (which expunges the flagged messages).
    ///
    /// # Errors
    /// Imap errors can happen
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), err(level = "info"))
    )]
    pub async fn delete_uids(&mut self, mailbox: &str, sequence: &str) -> Result<(), ImapError> {
        self.session
            .select(mailbox)
            .await
            .or_raise(|| ImapError(format!("imap select {mailbox:?} failed")))?;

        {
            let mut stream = self
                .session
                .uid_store(sequence, "+FLAGS (\\Deleted)")
                .await
                .or_raise(|| ImapError("imap uid store failed".to_owned()))?;
            while stream
                .try_next()
                .await
                .or_raise(|| ImapError("uid store stream error".to_owned()))?
                .is_some()
            {}
        }

        self.session
            .close()
            .await
            .or_raise(|| ImapError("imap close failed".to_owned()))?;

        Ok(())
    }

    /// Get a list of mailboxes given filters, returns a `BTreeMap` so it is
    /// sorted and stable.
    ///
    /// We use a map to be able to have generic filters at the beginning of the
    /// configuration that are overwritten by more specific filters afterwards.
    ///
    /// # Errors
    /// Many errors can happen
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(self), ret, err(level = "info"))
    )]
    pub async fn list(&mut self) -> Result<BTreeMap<String, ListResult<T>>, ImapError> {
        let mut mailboxes: BTreeMap<String, ListResult<T>> = BTreeMap::new();

        for filter in self
            .filters
            .clone()
            .unwrap_or_else(|| vec![Filter::default()])
        {
            let mut found = false;

            let names: Vec<_> = {
                let stream = self
                    .session
                    .list(filter.reference.as_deref(), filter.pattern.as_deref())
                    .await
                    .or_raise(|| ImapError(format!("imap list failed with {filter:?}")))?;
                stream
                    .try_collect()
                    .await
                    .or_raise(|| ImapError("imap list stream error".to_owned()))?
            };

            for mailbox in names
                .iter()
                .filter(|mbx| !mbx.attributes().contains(&NameAttribute::NoSelect))
                .filter(|mbx| {
                    filter
                        .include_re
                        .as_ref()
                        .is_none_or(|re| re.is_match(mbx.name()))
                })
                .filter(|mbx| {
                    filter
                        .exclude_re
                        .as_ref()
                        .is_none_or(|re| !re.is_match(mbx.name()))
                })
            {
                found = true;
                mailboxes.insert(mailbox.name().to_owned(), ListResult {
                    extra: filter.extra.clone().or_else(|| self.extra.clone()),
                });
            }

            if !found {
                bail!(ImapError(format!(
                    "This filter did not return anything {filter:?}"
                )));
            }
        }

        Ok(mailboxes)
    }
}

/// Wrap a raw `TcpStream` in the appropriate TLS layer (or leave as-is for plaintext),
/// returning the opaque `ImapStream` and whether the server greeting has already been consumed.
#[cfg_attr(
    not(feature = "__tls"),
    expect(clippy::unused_async, reason = "only needed when using tls")
)]
async fn build_stream(
    tcp: TcpStream,
    mode: &Mode,
    #[cfg_attr(
        not(feature = "__tls"),
        expect(unused_variables, reason = "only needed when using tls")
    )]
    server: &str,
    port: u16,
) -> Result<(ImapStream, bool), ImapError> {
    match *mode {
        Mode::Plaintext => Ok((Box::new(tcp), false)),
        #[cfg(feature = "__tls")]
        Mode::Tls => {
            let tls = wrap_tls(tcp, server).await?;
            Ok((tls, false))
        },
        #[cfg(feature = "__tls")]
        Mode::StartTls => {
            let mut plain_client = async_imap::Client::new(tcp);
            plain_client
                .read_response()
                .await
                .or_raise(|| ImapError("failed to read greeting before STARTTLS".to_owned()))?;
            plain_client
                .run_command_and_check_ok("STARTTLS", None)
                .await
                .or_raise(|| ImapError("STARTTLS command failed".to_owned()))?;
            let tcp_back: TcpStream = plain_client.into_inner();
            let tls = wrap_tls(tcp_back, server).await?;
            Ok((tls, true))
        },
        Mode::AutoTls => {
            #[cfg(feature = "__tls")]
            {
                if port == 993 {
                    let tls = wrap_tls(tcp, server).await?;
                    Ok((tls, false))
                } else {
                    // Treat as StartTls
                    let mut plain_client = async_imap::Client::new(tcp);
                    plain_client.read_response().await.or_raise(|| {
                        ImapError("failed to read greeting before STARTTLS".to_owned())
                    })?;
                    plain_client
                        .run_command_and_check_ok("STARTTLS", None)
                        .await
                        .or_raise(|| ImapError("STARTTLS command failed".to_owned()))?;
                    let tcp_back: TcpStream = plain_client.into_inner();
                    let tls = wrap_tls(tcp_back, server).await?;
                    Ok((tls, true))
                }
            }
            #[cfg(not(feature = "__tls"))]
            {
                let _ = port;
                Ok((Box::new(tcp), false))
            }
        },
        Mode::Auto => {
            #[cfg(feature = "__tls")]
            {
                if port == 993 {
                    let tls = wrap_tls(tcp, server).await?;
                    return Ok((tls, false));
                }
                // Non-993: read greeting then attempt STARTTLS; fall back to plaintext if not supported
                let mut plain_client = async_imap::Client::new(tcp);
                plain_client
                    .read_response()
                    .await
                    .or_raise(|| ImapError("failed to read server greeting".to_owned()))?;
                // Try STARTTLS; if the server returns NO/BAD, fall back to plaintext
                let starttls_ok = plain_client
                    .run_command_and_check_ok("STARTTLS", None)
                    .await
                    .is_ok();
                if starttls_ok {
                    let tcp_back: TcpStream = plain_client.into_inner();
                    let tls = wrap_tls(tcp_back, server).await?;
                    Ok((tls, true))
                } else {
                    // Server does not support STARTTLS; use plaintext
                    let stream: TcpStream = plain_client.into_inner();
                    Ok((Box::new(stream), true))
                }
            }
            #[cfg(not(feature = "__tls"))]
            {
                let _ = port;
                Ok((Box::new(tcp), false))
            }
        },
    }
}

/// Wrap a `TcpStream` in a TLS layer using the OpenSSL backend.
#[cfg(feature = "native-tls")]
async fn wrap_tls(tcp: TcpStream, server: &str) -> Result<ImapStream, ImapError> {
    let connector = native_tls::TlsConnector::new()
        .or_raise(|| ImapError("native TLS connector creation failed".to_owned()))?;
    let connector = tokio_native_tls::TlsConnector::from(connector);
    let tls = connector
        .connect(server, tcp)
        .await
        .or_raise(|| ImapError(format!("TLS handshake with {server} failed")))?;
    Ok(Box::new(tls))
}

/// Wrap a `TcpStream` in a TLS layer using the rustls backend.
#[cfg(feature = "rustls")]
async fn wrap_tls(tcp: TcpStream, server: &str) -> Result<ImapStream, ImapError> {
    use std::sync::Arc;

    use tokio_rustls::rustls;

    let cert_result = rustls_native_certs::load_native_certs();
    if !cert_result.errors.is_empty() {
        return Err(ImapError(format!(
            "failed to load native certs: {:?}",
            cert_result.errors
        )))
        .or_raise(|| ImapError("failed to load native certs".to_owned()));
    }
    let mut roots = rustls::RootCertStore::empty();
    for cert in cert_result.certs {
        roots
            .add(cert)
            .or_raise(|| ImapError("failed to add cert".to_owned()))?;
    }
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let connector = tokio_rustls::TlsConnector::from(Arc::new(config));
    let dns = rustls::pki_types::ServerName::try_from(server.to_owned())
        .or_raise(|| ImapError(format!("invalid server name: {server}")))?;
    let tls = connector
        .connect(dns, tcp)
        .await
        .or_raise(|| ImapError(format!("TLS handshake with {server} failed")))?;
    Ok(Box::new(tls))
}

/// Convert a set of `Uid`s into a collapsed IMAP sequence string.
///
/// For example, `{1, 2, 3, 7, 8}` becomes `"1:3,7:8"`.
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", skip(ids), ret)
)]
pub fn ids_list_to_collapsed_sequence(ids: &HashSet<Uid>) -> String {
    #[cfg(feature = "tracing")]
    tracing::trace!(?ids);

    debug_assert!(!ids.is_empty(), "ids must not be empty");

    // Collect and sort the IDs
    let mut sorted_ids: Vec<_> = ids.iter().copied().collect();
    sorted_ids.sort_unstable();

    // Collect ranges from the sorted list
    let mut result = Vec::new();
    let mut start = sorted_ids.first().copied();
    let mut end = start;

    for &id in sorted_ids.get(1..).unwrap_or_default() {
        match (end, start) {
            (Some(e), Some(_s)) if id == e + 1 => end = Some(id),
            _ => {
                // Push the previous range
                if let (Some(s), Some(e)) = (start, end) {
                    result.push(
                        if s == e {
                            s.to_string()
                        } else {
                            format!("{s}:{e}")
                        },
                    );
                }
                start = Some(id);
                end = start;
            },
        }
    }

    // Push the last range
    if let (Some(s), Some(e)) = (start, end) {
        result.push(
            if s == e {
                s.to_string()
            } else {
                format!("{s}:{e}")
            },
        );
    }

    result.join(",")
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use async_imap::types::Uid;

    use super::ids_list_to_collapsed_sequence;

    #[cfg_attr(not(debug_assertions), ignore = "testing debug_assert!")]
    #[test]
    #[should_panic(expected = "ids must not be empty")]
    fn empty_set() {
        let ids: HashSet<Uid> = HashSet::new();
        ids_list_to_collapsed_sequence(&ids);
    }

    #[test]
    fn single_id() {
        let mut ids = HashSet::new();
        ids.insert(5);
        assert_eq!(ids_list_to_collapsed_sequence(&ids), "5");
    }

    #[test]
    fn continuous_range() {
        let ids: HashSet<_> = [1, 2, 3, 4, 5].iter().copied().collect();
        assert_eq!(ids_list_to_collapsed_sequence(&ids), "1:5");
    }

    #[test]
    fn multiple_disjoint_ranges() {
        let ids: HashSet<_> = [1, 2, 3, 7, 8, 10, 11].iter().copied().collect();
        assert_eq!(ids_list_to_collapsed_sequence(&ids), "1:3,7:8,10:11");
    }

    #[test]
    fn mixed_ranges_and_single_ids() {
        let ids: HashSet<_> = [1, 3, 4, 6, 7, 10, 12].iter().copied().collect();
        assert_eq!(ids_list_to_collapsed_sequence(&ids), "1,3:4,6:7,10,12");
    }

    #[test]
    fn unsorted_input() {
        let ids: HashSet<_> = [10, 1, 4, 5, 12, 6, 22, 23, 24, 31]
            .iter()
            .copied()
            .collect();
        assert_eq!(ids_list_to_collapsed_sequence(&ids), "1,4:6,10,12,22:24,31");
    }
}
