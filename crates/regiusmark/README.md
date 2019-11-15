# Regius Mark

Regius Mark is a cryptocurrency that is backed by physical gold assets. A single
token is backed by one physical gram of gold. Blockchain technology is used to
provide an immutable and cryptographically verified ledger. The system is
centralized allowing for global scalability that would otherwise be foregone in
a decentralized system.

[Website](https://regiusmark.io) |
[Whitepaper](https://regiusmark.io/whitepaper)

## Overview

Regius Mark's core library and blockchain implementation.

The library API provides:

- Building transactions
- Building network messages
- Creating scripts and converting them to P2SH addresses used for receiving
- Generating keys and optionally converting them to a default P2SH address
- Backend storage for blocks and indexing
- Validating blocks and transactions

This library does not provide a network client implementation remaining agnostic
to any networking library.

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
