# IMAP Tools

## Installation

```shell
$ cargo install imap-tools
```

## Usage

Most tools have the same generic arguments:

- `-c` - `--config` - The configuration file to use.

You can set the next arguments on the command line or in the configuration file.

- `-u` - `--username` - The username for login.
- `-p` -  `--password` - The password for login.
- `-P` - `--password_command` - A command that prints the password.
- `-s` - `--server` - The IMAP server. The default is localhost.
- `-d` - `--debug` - Print all the IMAP dialogue.
- `-n` - `--dry-run` - Do not change anything on the server.

A configuration file can start like this:

```toml
password-command = "secret-tool lookup id example"
server           = "imap.example.com"
username         = "alice@example.com"

# This will match all mailboxes
[[filters]]
  reference = ""
  pattern   = "*"

# This will match the first level of mailboxes, and then only keep mailboxes
# whose full path contain strings in `include`.
[[filters]]
  reference = ""
  pattern   = "%"
  include   = ["INBOX", "Sent"]
```

Four fields give a finer selection of mailboxes.
They are `include`, `exclude`, `include_re` and `exclude_re`.
`include` and `exclude` match basic strings.
`include_re` and `exclude_re` need valid [Regex patterns](https://docs.rs/regex/latest/).
Each field adds mailboxes to the selection (`include`, `include_re`) or excludes them from it (`exclude`, `exclude_re`).

Some tools, for example clean, have an `extra` parameter.
You can set `extra` globally, in each filter, or in both.
The documentation of each tool describes its `extra` parameter.
If a filter does not set `extra`, it uses the global `extra`.
In this example, the filters have these `extra` values:

```toml
extra = "foo"

[[filters]]
  reference = ""
  name = "*"
  extra = "bar"

[[filters]]
  reference = "foo"
  name = "*"

[[filters]]
  reference = "bar"
  name = "*"
```

The result is the same as this configuration:

```toml
[[filters]]
  reference = ""
  name = "*"
  extra = "bar"

[[filters]]
  reference = "foo"
  name = "*"
  extra = "foo"

[[filters]]
  reference = "bar"
  name = "*"
  extra = "foo"
```

### Example usage

```shell
$ imap-tools list -c config.toml
```

## Tools

### list

This tool lists the mailboxes that the other tools process.
Use it to test your filters. It does not change anything.

### finddup

This tool reads all the mailboxes of an IMAP server and finds messages that have the same message id.
It deletes the duplicates.

### clean

This tool reads the mailboxes and deletes old messages. It uses simple rules.

Put the rules in the `extra` parameter of a filter.
For example:

```toml
[[filters]]
  reference = ""
  name = "*"

  [filters.extra]
    0MB = 105
    5MB = 85
    10MB = 55
```

The reference and name are the same as for the other tools.
`extra` is a map. Each key is a mailbox size in MB. Each value is the number of days to keep messages.
In this example, the tool keeps messages for 105 days.
If the mailbox is larger than 5MB and the oldest message is newer than 105 days, the tool keeps messages for 85 days.
If the mailbox is larger than 10MB and the oldest message is newer than 55 days, the tool keeps messages for 55 days.

### archive

This tool archives old messages. It moves them from mailboxes into archive mailboxes.
Its `extra` parameter contains a number of days and a string format for the location of the archive mailbox.
The tool passes the string format through [strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers). You can use all its specifiers.
The strftime calls use the date on which the server received each message. IMAP calls this date the internaldate.
The string format can also contain `%%MBX` (with two %). The tool replaces it with the full mailbox name.
For example:

```toml
[[filters]]
  reference = ""
  name = "*"

  [filters.extra]
    days   = 200
    format = "Archive/%Y/%%MBX"
```

This configuration moves messages that are older than 200 days into the archive mailbox.
If the mailbox is `INBOX/bob` and the message is from 2024, the tool moves it into `Archive/2024/INBOX/bob`.

Here is another example:

```toml
[[filters]]
  reference = ""
  name = "*"

  [filters.extra]
    days   = 200
    format = "%%MBX/old/%Y/%m"
```

If the mailbox is `INBOX/bob` and the message is from September 2024, the tool moves it into `INBOX/bob/old/2024/09`.

### Authentication

The `auth` field selects the SASL mechanism. Set it in the configuration file or with the `--auth` command-line flag.
The default is `login` (the standard IMAP LOGIN command).

| `auth` value    | Mechanism     | Credential needed           | Notes                                    |
|-----------------|---------------|-----------------------------|------------------------------------------|
| `login`         | IMAP LOGIN    | `password` / `password-command` | Default                            |
| `plain`         | SASL PLAIN    | `password` / `password-command` | Requires TLS to avoid cleartext        |
| `cram-md5`      | SASL CRAM-MD5 | `password` / `password-command` | Challenge/response, no TLS required    |
| `scram-sha-1`   | SASL SCRAM-SHA-1 | `password` / `password-command` | Mutual-auth, RFC 5802              |
| `scram-sha-256` | SASL SCRAM-SHA-256 | `password` / `password-command` | Preferred over SHA-1, RFC 7677   |
| `xoauth2`       | SASL XOAUTH2  | `oauth2-command`            | Bearer token, for Gmail / Office 365     |

Example that uses SCRAM-SHA-256:

```toml
auth             = "scram-sha-256"
server           = "imap.example.com"
username         = "user@example.com"
password-command = "pass show imap/example"
```

### `OAuth2` (Gmail / Office 365)

Gmail and Office 365 do not accept plain-password IMAP login.
Set `auth = "xoauth2"` and `oauth2-command`. The **stdout** of `oauth2-command` must be a valid bearer access token.

```toml
auth           = "xoauth2"
server         = "imap.gmail.com"
port           = 993
username       = "user@gmail.com"
oauth2-command = "oauth2l fetch --output_format=bare https://mail.google.com/"
```

imap-tools does **not** refresh or cache tokens. Use an external tool for this:

- **Gmail** — [`oauth2l`](https://github.com/google/oauth2l) or [`mutt_oauth2.py`](https://gitlab.com/muttmua/mutt/-/blob/master/contrib/oauth2/mutt_oauth2.py).
  Get a refresh token once. Follow the setup guide of the tool. Then use the command above.
- **Office 365** — [`mutt_oauth2.py`](https://gitlab.com/muttmua/mutt/-/blob/master/contrib/oauth2/mutt_oauth2.py) with `--provider microsoft`.

The tool gets a new access token on every run. If the token expired, the IMAP server returns an authentication error.

### imap

These commands send "raw" IMAP commands.

#### list

This command is like the list tool, but it does not need a configuration file.

#### create

This command creates a mailbox.

#### delete

This command deletes a mailbox.
