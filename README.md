# Regius Mark
[![Build Status](https://travis-ci.com/RegiusMark/RegiusMark.svg?branch=master)](https://travis-ci.com/RegiusMark/RegiusMark)

https://regiusmark.io

## What is Regius Mark?

Regius Mark is a cryptocurrency that is backed by physical gold assets. The
digital token name is represented as MARK. A single token will be represented by
a gram of gold.

For more information see the [whitepaper](https://regiusmark.io/whitepaper).

## Development

This project is still under heavy development. APIs are currently evolving all
the time and not to be considered stable. It is possible to run a private test
net for personal use and development.

### Prerequisites

Ensure you have the following software installed:

- Rust compiler (1.36+)
- libsodium

### Getting started

Make sure the source code is locally available by either cloning the repository
or downloading it.

#### Runtime environment

- `REGIUSMARK_HOME` - specifies the directory where data and configurations are
  stored.

#### Running

Run the test suite:
```
$ cargo test
```

Launch CLI:
```
$ cargo run --bin regiusmark-cli
```

Launch server:
```
$ cargo run --bin regiusmark-server
```

The server requires a configuration file in the home folder called
`config.toml`

Configuration keys:

- `minter_key` - (required) Minter key to use for block production
- `bind_address` - (optional) - default is 127.0.0.1:7777) The bind address for
  the server to listen on
