# netid64

[![Rust Crate](https://img.shields.io/crates/v/netid64?label=Rust%20Crate)](https://crates.io/crates/netid64)
[![API](https://img.shields.io/docsrs/netid64?label=API)](https://docs.rs/netid64)

Provides `NetId64`: "valid for life", as long as the runtime that
minted it runs.

It is a fixed-layout `u64` identifier intended for protocols and
runtime substrates that need a compact, inspectable id.

`NetId64` is not a universal identifier like a UUID. It is optimized for
runtime-bound identifiers: the identifier is meaningful inside the
lifetime of the node, process, server, or transport session that minted
it. When that runtime is gone, the identifiers it produced are gone with
it.

```text
┌────────┬─────────────┬──────────────────────────────┐
│ KIND:8 │   NODE:16   │         COUNTER:40           │
└────────┴─────────────┴──────────────────────────────┘

raw = (kind << 56) | (node << 40) | (counter & 0xFF_FFFF_FFFF)
wire = raw.to_be_bytes()
text = "kind:node:counter"
```

## Fields

```text
KIND
  8-bit caller-defined type, frame family, or entity class.

NODE
  16-bit origin node, fleet member, shard, or partition.

COUNTER
  40-bit local sequence value. Larger inputs are truncated by `make`.
```

## API

```rust
use netid64::NetId64;

let id = NetId64::make(1, 7, 42);

assert_eq!(id.raw(), 0x0100_0700_0000_002A);
assert_eq!(id.kind(), 1);
assert_eq!(id.node(), 7);
assert_eq!(id.counter(), 42);
```

```rust
use netid64::NetId64;

let id = NetId64::from_raw(0x0100_0700_0000_002A);
let bytes = id.to_be_bytes();

assert_eq!(NetId64::from_be_bytes(bytes), id);
```

## Usage

`NetId64` is useful anywhere the id should carry enough structure to be
decoded at the boundary where it is observed:

- shared-memory rings, where the counter can point at the slot/version
  that produced the value;
- binary protocols, where the id must fit in a fixed-width frame header;
- local process fleets, where `NODE` distinguishes the writer;
- routing, cache, event, or metric substrates that need stable type
  families through `KIND`;
- tests and debugging tools, where `"kind:node:counter"` is easier to
  inspect than an opaque random id.

Current related projects:

- [iadev09/orbit](https://github.com/iadev09/orbit) uses `NetId64` as
  the identifier format for fleet-aware shared-memory ring frames.
- [iadev09/nwd1](https://github.com/iadev09/nwd1) is the adjacent binary
  framing work where a fixed-width id can live directly in frame
  metadata.
- [iadev09/nwd1-quic](https://github.com/iadev09/nwd1-quic) carries the
  same framing model over QUIC transport.
- [iadev09/nwd1-webtransport](https://github.com/iadev09/nwd1-webtransport)
  extends the transport direction toward browser/server WebTransport
  boundaries.

## Text Format

```rust
use netid64::NetId64;

let id: NetId64 = "1:7:42".parse().unwrap();

assert_eq!(id.to_string(), "1:7:42");
assert_eq!(format!("{id:?}"), "NetId64(1:7:42 | 0x010007000000002A)");
```

Raw hex `u64` input is also accepted:

```rust
use netid64::NetId64;

let id: NetId64 = "0x010007000000002A".parse().unwrap();

assert_eq!(id.kind(), 1);
assert_eq!(id.node(), 7);
assert_eq!(id.counter(), 42);
```

## Counter Truncation

```rust
use netid64::NetId64;

let id = NetId64::make(0, 0, u64::MAX);

assert_eq!(id.counter(), 0xFF_FFFF_FFFF);
```

## Public Items

```text
NetId64
ParseNetId64Error
```
