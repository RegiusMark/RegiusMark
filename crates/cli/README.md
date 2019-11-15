# Regius Mark

Regius Mark is a cryptocurrency that is backed by physical gold assets. A single
token is backed by one physical gram of gold. Blockchain technology is used to
provide an immutable and cryptographically verified ledger. The system is
centralized allowing for global scalability that would otherwise be foregone in
a decentralized system.

[Website](https://regiusmark.io) |
[Whitepaper](https://regiusmark.io/whitepaper)

## Overview

Command-line interface for interacting with the blockchain. A wallet is provided
amongst other utilities.

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
$ cargo run --bin regiusmark -- --help
```

Start the Regius Mark CLI wallet:
```
$ cargo run --bin regiusmark -- wallet
```

See available options for the Regius Mark CLI wallet:
```
$ cargo run --bin regiusmark -- wallet --help
```
