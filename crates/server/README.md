# Regius Mark

Regius Mark is a cryptocurrency that is backed by physical gold assets. A single
token is backed by one physical gram of gold. Blockchain technology is used to
provide an immutable and cryptographically verified ledger. The system is
centralized allowing for global scalability that would otherwise be foregone in
a decentralized system.

[Website](https://regiusmark.io) |
[Whitepaper](https://regiusmark.io/whitepaper)

## Overview

Core server implementation providing RPC functionality for clients and producing
blocks in the network.

[![Build Status](https://travis-ci.com/RegiusMark/regiusmark.svg?branch=master)](https://travis-ci.com/RegiusMark/regiusmark)

## Supported Rust Versions

Regius Mark is built against the latest stable version of the compiler. Any
previous versions are not guaranteed to compile.

## Developing

When bugs are fixed, regression tests should be added. New features likewise
should have corresponding tests added to ensure correct behavior.

Run the test suite:
```
$ cargo test
```

The crate should build and tests should pass.

## Running

Make sure the tests pass before starting the server to ensure correct
functionality. See the [Developing](#Developing) section for running tests.

### Runtime environment

- `REGIUSMARK_HOME` - (optional) specifies the directory where data and
  configurations are stored.

### Launching

See available options:
```
$ cargo run --bin regiusmark-server -- --help
```

Start the Regius Mark server:
```
$ cargo run --bin regiusmark-server
```

The server requires a configuration file in the home folder called
`config.toml`. The config implementation can be found in
`src/bin/server/main.rs`.

Configuration keys:

- `minter_key` - (required) Minter key to use for block production
- `enable_stale_production` - (required) Produces blocks even if there are no
  transactions
- `bind_address` - (optional) - default is 127.0.0.1:7777) The bind address for
  the server to listen on
