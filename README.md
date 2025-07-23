# IMAP Tools

## Installation

```shell
$ cargo install imap-tools
```

## Usage

Most tools have the same generic arguments :

- `-c` - `--config` - The configuration file to use.

Next, these arguments can be either set on the command line or in the config file.

- `-u` - `--username` - The username to use for login.
- `-p` -  `--password` - The password to use for login.
- `-P` - `--password_command` - A command to use to get the password.
- `-s` - `--server` - The imap server, defaults to localhost.
- `-d` - `--debug` - Dump all the imap dialogue.
- `-n` - `--dry-run` - Don't change anything on the server.

So a configuration file may start with:

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

There are four fields that allow a finer selection of mailboxes.
They are `include`, `exclude`, `include_re` and `exclude_re`.
The first two will include and exclude based on basic string matching.
The later two will need to be valid [Regex patterns](https://docs.rs/regex/latest/), and will also include mailboxes and exclude them.

Some tools like clean have an `extra` parameter added globally, and/or to each filter, description is provided when this happens.
If a filter does not have the `extra` parameter set, the global one gets used instead.
For example if you have these filters with extras:

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

They would end up as if this had been written as:

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

This tool will list the mailboxes that would be processed by other tools.
It helps you figure out your filters without actually doing anything.

### finddup

This tool will go over all the mailboxes from an imap server, find messages with duplicate message ids, and remove duplicates.

### clean

This tool will go over the mailboxes and cleanup old messages according to simples rules.

The rules are added to a filter using the extra parameter.
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

The reference and name are the same as for other tools. `extra` is a map of mailbox size in MB to days of messages that should be kept.
In this example, messages are kept up to 105 days, unless the mailbox is larger than 5MB, and oldest message is newer than 105 days, then messages are kept up to 85 days, unless the mailbox is more than 10MB and the oldest message is less than 55 days old.

### archive

This tool will "archive", aka move, old emails from mailboxes into archive mailboxes.
Its extra parameter contains a number of days and a string format for the archive mailbox location.
The string format can contain is passed through [strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers) so all its specifiers can be used.
The strftime calls are made based on the date on which each email was received. (Called the internaldate in IMAP terms).
The string format can also contain `%%MBX` (yes, with two %) that will be replaced with the full mailbox name.
For example:

```toml
[[filters]]
  reference = ""
  name = "*"

  [filters.extra]
    days   = 200
    format = "Archive/%Y/%%MBX"
```

Will move messages older than 200 days into the archive mailbox.
If the mailbox is `INBOX/bob`, and the email is from 2024, it will be moved into `Archive/2024/INBOX/bob`.

Another example could be:

```toml
[[filters]]
  reference = ""
  name = "*"

  [filters.extra]
    days   = 200
    format = "%%MBX/old/%Y/%m"
```

If the mailbox is `INBOX/bob`, and the email is from september 2024, it will be moved into `INBOX/bob/old/2024/09`.

### imap

Permit somewhat "raw" imap commands

#### list

This is comparable to the list command, but it does not need a config file.

