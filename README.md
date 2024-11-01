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
- `-D` - `--dry-run` - Don't change anything on the server.

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
