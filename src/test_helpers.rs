#![expect(clippy::expect_used, reason = "test helper")]
use std::{
    collections::VecDeque,
    io::{BufRead as _, BufReader, Write as _},
    net::{TcpListener, TcpStream},
    thread,
};

use regex::Regex;

pub enum ExpectCommand {
    Static(String),
    Regex(Regex),
}

/// A scripted IMAP exchange: untagged response lines + final tagged response.
pub struct MockExchange {
    /// Untagged lines sent before the tagged response (each must include `\r\n`).
    pub untagged: Vec<String>,
    /// Tagged response suffix, e.g. `"OK completed"` or `"NO Mailbox already exist"`.
    pub tagged: String,
    /// If set, the mock server asserts the client sent this exact command
    /// (everything after the IMAP tag, trimmed).
    pub command: Option<ExpectCommand>,
}

impl MockExchange {
    /// Successful exchange: tagged `OK completed` after the untagged lines.
    pub fn ok(untagged: Vec<String>) -> Self {
        Self {
            untagged,
            tagged: "OK completed".to_owned(),
            command: None,
        }
    }

    /// Failed exchange: tagged `NO <reason>`, no untagged lines.
    pub fn no(reason: impl Into<String>) -> Self {
        Self {
            untagged: vec![],
            tagged: format!("NO {}", reason.into()),
            command: None,
        }
    }

    /// Assert that the client sends this exact command (everything after the IMAP tag).
    pub fn expect_command(mut self, cmd: impl Into<String>) -> Self {
        self.command = Some(ExpectCommand::Static(cmd.into()));
        self
    }

    #[track_caller]
    pub fn expect_command_re<T>(mut self, re: T) -> Self
    where
        T: TryInto<Regex>,
        <T as std::convert::TryInto<regex::Regex>>::Error: std::fmt::Debug,
    {
        self.command = Some(ExpectCommand::Regex(re.try_into().expect("should compile")));
        self
    }
}

/// A single-connection mock IMAP TCP server for tests.
///
/// Handles `CAPABILITY`, `LOGIN`, and `LOGOUT` automatically.
/// All other commands are answered from the provided script in order.
pub struct MockServer {
    pub port: u16,
    handle: thread::JoinHandle<()>,
}

impl MockServer {
    /// Start the server on a random local port.
    ///
    /// `extra_caps`: additional capabilities beyond `IMAP4rev1 UIDPLUS` (e.g. `&["MOVE"]`).
    /// `script`: one `MockExchange` per non-handshake IMAP command.
    pub fn start(extra_caps: &'static [&'static str], script: Vec<MockExchange>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind to local port");
        let port = listener.local_addr().expect("get local port").port();
        let handle = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept connection");
            run_session(stream, extra_caps, script);
        });
        Self { port, handle }
    }

    pub fn join(self) {
        self.handle.join().expect("mock server thread panicked");
    }
}

#[track_caller]
fn run_session(stream: TcpStream, extra_caps: &[&str], script: Vec<MockExchange>) {
    let mut script: VecDeque<MockExchange> = script.into();
    let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
    let mut writer = stream;
    let mut exchange_index: usize = 0;

    writer
        .write_all(b"* OK IMAP4rev1 mock server ready\r\n")
        .expect("write greeting");

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).expect("read line") == 0 {
            break;
        }
        let tag = line.split_whitespace().next().unwrap_or("A0").to_owned();
        let cmd = line
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .to_ascii_uppercase();

        match cmd.as_str() {
            "CAPABILITY" => {
                let caps = if extra_caps.is_empty() {
                    "IMAP4rev1 UIDPLUS".to_owned()
                } else {
                    format!("IMAP4rev1 UIDPLUS {}", extra_caps.join(" "))
                };
                writer
                    .write_all(
                        format!("* CAPABILITY {caps}\r\n{tag} OK CAPABILITY completed\r\n")
                            .as_bytes(),
                    )
                    .expect("write capability");
            },
            "LOGIN" => {
                writer
                    .write_all(format!("{tag} OK LOGIN completed\r\n").as_bytes())
                    .expect("write login");
            },
            "LOGOUT" => {
                writer
                    .write_all(
                        format!("* BYE logging out\r\n{tag} OK LOGOUT completed\r\n").as_bytes(),
                    )
                    .expect("write logout");
                break;
            },
            _ => {
                let exchange = script
                    .pop_front()
                    .unwrap_or_else(|| MockExchange::ok(vec![]));
                #[expect(clippy::string_slice, reason = "ok")]
                // #[allow(clippy::print_stderr, reason = "ok")]
                match exchange.command {
                    Some(ExpectCommand::Static(expected)) => {
                        let actual = line[tag.len()..].trim();
                        assert_eq!(
                            actual,
                            expected.as_str(),
                            "command mismatch at exchange #{exchange_index}: expected {expected:?}, got {actual:?}"
                        );
                    },
                    Some(ExpectCommand::Regex(re)) => {
                        let actual = line[tag.len()..].trim();
                        assert!(
                            re.is_match(actual),
                            "command mismatch at exchange #{exchange_index}: expected {re:?}, got {actual:?}"
                        );
                    },
                    None => {
                        todo!("MOCK #{exchange_index}: {}", line[tag.len()..].trim());
                    },
                }
                exchange_index += 1;
                for resp in &exchange.untagged {
                    writer.write_all(resp.as_bytes()).expect("write untagged");
                }
                writer
                    .write_all(format!("{tag} {}\r\n", exchange.tagged).as_bytes())
                    .expect("write tagged");
            },
        }
    }
}

/// Create a minimal `BaseConfig` pointing at 127.0.0.1 with test credentials.
pub fn test_base() -> crate::libs::base_config::BaseConfig {
    crate::libs::base_config::BaseConfig::new(&crate::libs::args::Generic {
        server: Some("127.0.0.1".to_owned()),
        username: Some("test".to_owned()),
        password: Some("test".to_owned()),
        ..Default::default()
    })
    .expect("test base config")
}

/// Build a `BODY[HEADER.FIELDS ("MESSAGE-ID")]` FETCH response line for one message.
///
/// `seq` is the sequence number, `uid` is the UID, `msg_id` is e.g. `"<foo@bar.com>"`.
pub fn header_fetch_line(seq: u32, uid: u32, msg_id: &str) -> String {
    let header = format!("Message-ID: {msg_id}\r\n\r\n");
    let len = header.len();
    format!(
        "* {seq} FETCH (UID {uid} BODY[HEADER.FIELDS (\"MESSAGE-ID\")] {{{len}}}\r\n{header})\r\n"
    )
}
