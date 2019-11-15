## Releases

All crates must have the same version number when creating a release. This
simplifies documenting any changes.

# Unreleased

# Version 0.1.0 (2019-11-14)

This marks the first release of the project. The blockchain server supports
running an alpha network. Clients are able to connect to the server and be able
to interact with the blockchain using the CLI wallet.

Power users will be using the CLI crate to interact with the public network when
it launches. Developers can take a look at `crates/regiusmark` for creating
applications and `crates/server` for running a private alpha network locally for
testing.

### Crates released
- crates/cli
- crates/regiusmark
- crates/server
