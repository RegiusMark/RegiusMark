## Releases

All crates must have the same version number when creating a release. This
simplifies documenting any changes.

# Version 0.2.1 (2019-12-03)

Fixes a memory issue when retrieving block ranges by ensuring that the network
handler can't queue an infinite amount of messages.

- Configure the max message send queue in the WebSocket handler for back
  pressure control.

# Version 0.2.0 (2019-11-29)

This release improves the networking protocol and allows eliminating an
unnecessary round-trip RPC call when synchronizing blocks.

- Network protocol supports setting the block filter with an empty hash set to
  retrieve only block headers.
- Add ClearBlockFilter net API to the network protocol.
- Remove GetBlockHeader net API from the network protocol.
- Add GetFullBlock net API to allow filtering all blocks and allow retrieving
  full blocks when necessary.
- Add GetBlockRange net API to stream back a range of blocks to the client. This
  is more efficient than naively looping GetBlock requests to the server.
- Use a bounded sender when sending network messages for back pressure control.

### Breaking changes

- The network protocol message type constants have been changed.
- Clients must support streaming responses from the GetBlockRange network API.

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
