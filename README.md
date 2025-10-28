# 🜲 NetId64

> _A prototype type for precise and elegant 64-bit network identity._

`NetId64` is a minimal, no-std-friendly Rust crate that defines a
**64-bit deterministic network identifier**:

[KIND:8][NODE:16][COUNTER:40]

Each bit has a purpose:

- **KIND (8 bits)** — logical entity type (user, message, node, etc.)
- **NODE (16 bits)** — origin node / partition
- **COUNTER (40 bits)** — local monotonic counter

### 🧩 Philosophy

This crate is part of a broader prototype of an idea:
> **transport-safe identity.**  
> Not random, not opaque, but shaped by its place in the network.
>
> It’s not yet another UUID — it’s a deterministic, modular identity
> that can live across QUIC frames, caches, or async boundaries.

Think of it as the **prototype species** of a coming type family:  
`NetId64` → `NetIdUuid` → `NetIdSnowflake` — all share the same wire semantics.

### ✳️ Features

- `no_std` compatible
- optional `serde` support
- constant-time encode/decode (`u64` big-endian)
- `Display`/`FromStr` for human-friendly forms (`"1:42:999"` or `"0x01AA00F3..."`)

### ⚙️ Example

```rust
use netid64::NetId64;

let id = NetId64::make(1, 7, 42);
println!("{}", id); // 1:7:42
assert_eq!(id.node(), 7);
assert_eq!(id.counter(), 42);



---

### 🌍 Manifesto

This crate carries no grand promise — only a direction:  
to design **identities that serve life**, not ownership.

Everything is for life.  
And if identity is a certificate,  
let it be the **most widely authorized** one —  
granted by existence itself.