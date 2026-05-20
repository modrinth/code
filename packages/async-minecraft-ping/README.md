> [!NOTE]
>
> This is a vendored version of [`async-minecraft-ping`](https://github.com/jsvana/async-minecraft-ping) with modifications for our own purposes.
>
> This directory is licensed under the same license as the original project.

# async-minecraft-ping

[![crates.io](https://img.shields.io/crates/v/async-minecraft-ping)][crate]
[![docs.rs](https://docs.rs/async-minecraft-ping/badge.svg)][docs]
[![CI](https://github.com/jsvana/async-minecraft-ping/actions/workflows/ci.yml/badge.svg)](https://github.com/jsvana/async-minecraft-ping/actions/workflows/ci.yml)
![crates.io](https://img.shields.io/crates/l/async-minecraft-ping/0.1.0)

An async [ServerListPing](https://wiki.vg/Server_List_Ping) client implementation in Rust.

## Usage

See [the example](./examples/status.rs).

```rust
let config = ConnectionConfig::build("mc.example.com")
    .with_port(25565)
    .with_timeout(Duration::from_secs(5));

let connection = config.connect().await?;
let status = connection.status().await?;

println!(
    "{} of {} player(s) online",
    status.status.players.online, status.status.players.max
);
```

## Features

### SRV Record Lookup

Enable the `srv` feature to support automatic SRV record resolution for Minecraft servers:

```toml
[dependencies]
async-minecraft-ping = { version = "0.8", features = ["srv"] }
```

```rust
let config = ConnectionConfig::build("skyblock.net")
    .with_srv_lookup()  // Resolves _minecraft._tcp.skyblock.net
    .connect()
    .await?;
```

When SRV lookup is enabled, the library queries `_minecraft._tcp.<address>` for an SRV record. If found, it uses the target host and port from the record. If not found, it falls back to the original address and port.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[crate]: https://crates.io/crates/async-minecraft-ping
[docs]: https://docs.rs/async-minecraft-ping
