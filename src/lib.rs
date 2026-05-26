//! `NetId64` — runtime-bound 64-bit identifier.
//!
//! Layout (big-endian semantics):
//!
//! ```text
//! ┌────────┬─────────────┬──────────────────────────────┐
//! │ KIND:8 │   NODE:16   │         COUNTER:40           │
//! └────────┴─────────────┴──────────────────────────────┘
//! ```
//!
//! `NetId64` is not a universal identifier like a UUID. It is optimized
//! for runtime-bound identifiers: meaningful inside the lifetime of the
//! node, process, server, shard, or transport session that minted it.
//!
//! The whole id is a `u64`; the field accessors are `const fn` so the
//! compiler can fold them at call sites that have a constant id.
//!
//! The wire form is the big-endian byte representation of the inner `u64`.

use core::fmt;
use core::str::FromStr;

const KIND_SHIFT: u32 = 56;
const NODE_SHIFT: u32 = 40;
const NODE_MASK: u64 = 0xFFFF;
const COUNTER_MASK: u64 = 0xFF_FFFF_FFFF;

/// Runtime-bound 64-bit identifier with `[KIND:8][NODE:16][COUNTER:40]` layout.
///
/// `Copy + Clone + PartialEq + Eq + Hash + Ord` — small value,
/// identity-by-bits, usable as a map key.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NetId64(pub u64);

impl NetId64 {
    /// The all-zeros id. Useful as a sentinel; not a valid mint.
    pub const ZERO: Self = Self(0);

    /// Construct from raw u64.
    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// The raw u64 representation.
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// The KIND byte — which type this id refers to.
    pub const fn kind(self) -> u8 {
        (self.0 >> KIND_SHIFT) as u8
    }

    /// The NODE part — which node, shard, partition, or runtime minted this id.
    pub const fn node(self) -> u16 {
        ((self.0 >> NODE_SHIFT) & NODE_MASK) as u16
    }

    /// The COUNTER part — caller-defined local sequence value.
    pub const fn counter(self) -> u64 {
        self.0 & COUNTER_MASK
    }

    /// Build an id from the three fields. The counter is truncated to 40 bits.
    pub const fn make(kind: u8, node: u16, counter: u64) -> Self {
        Self(
            ((kind as u64) << KIND_SHIFT)
                | ((node as u64) << NODE_SHIFT)
                | (counter & COUNTER_MASK),
        )
    }

    /// Big-endian wire bytes.
    pub const fn to_be_bytes(self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    /// Reconstruct from big-endian wire bytes.
    pub const fn from_be_bytes(bytes: [u8; 8]) -> Self {
        Self(u64::from_be_bytes(bytes))
    }
}

// ─────────────────────────────────────────────────────────────────────
// Display / Debug / Parse
// ─────────────────────────────────────────────────────────────────────

impl fmt::Display for NetId64 {
    /// Canonical decimal form: `"kind:node:counter"`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.kind(), self.node(), self.counter())
    }
}

impl fmt::Debug for NetId64 {
    /// Debug shows both the triple and the raw hex — handy in logs.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NetId64({} | 0x{:016X})", self, self.0)
    }
}

/// Parse error for `NetId64::from_str`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseNetId64Error;

impl fmt::Display for ParseNetId64Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid NetId64: expected \"kind:node:counter\" or \"0x...\"")
    }
}

impl std::error::Error for ParseNetId64Error {}

impl FromStr for NetId64 {
    type Err = ParseNetId64Error;

    /// Accepts both forms:
    /// - `"7:42:999"` — decimal triple
    /// - `"0xDEADBEEF..."` — raw hex u64
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            return u64::from_str_radix(hex, 16)
                .map(Self)
                .map_err(|_| ParseNetId64Error);
        }
        let mut parts = s.split(':');
        let kind: u8 = parts
            .next()
            .ok_or(ParseNetId64Error)?
            .parse()
            .map_err(|_| ParseNetId64Error)?;
        let node: u16 = parts
            .next()
            .ok_or(ParseNetId64Error)?
            .parse()
            .map_err(|_| ParseNetId64Error)?;
        let counter: u64 = parts
            .next()
            .ok_or(ParseNetId64Error)?
            .parse()
            .map_err(|_| ParseNetId64Error)?;
        if parts.next().is_some() {
            return Err(ParseNetId64Error);
        }
        Ok(Self::make(kind, node, counter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_roundtrip() {
        let id = NetId64::make(1, 0x1234, 0xABCDE);
        assert_eq!(id.kind(), 1);
        assert_eq!(id.node(), 0x1234);
        assert_eq!(id.counter(), 0xABCDE);
        assert_eq!(NetId64::from_be_bytes(id.to_be_bytes()).raw(), id.raw());
    }

    #[test]
    fn counter_truncates_to_40_bits() {
        // anything in the top 24 bits of `counter` is dropped
        let id = NetId64::make(0, 0, 0xFFFF_FFFF_FFFF_FFFF);
        assert_eq!(id.counter(), 0xFF_FFFF_FFFF);
    }

    #[test]
    fn display_and_parse_triple() {
        let id: NetId64 = "7:42:999".parse().unwrap();
        assert_eq!(id.kind(), 7);
        assert_eq!(id.node(), 42);
        assert_eq!(id.counter(), 999);
        assert_eq!(id.to_string(), "7:42:999");
    }

    #[test]
    fn parse_hex_form() {
        let id: NetId64 = "0xFF00_0000_0000_0000".replace('_', "").parse().unwrap();
        assert_eq!(id.kind(), 0xFF);
        assert_eq!(id.node(), 0);
        assert_eq!(id.counter(), 0);
    }

    #[test]
    fn debug_format_carries_hex() {
        let id = NetId64::make(1, 2, 3);
        let dbg = format!("{:?}", id);
        assert!(dbg.contains("1:2:3"));
        assert!(dbg.contains("0x"));
    }

    #[test]
    fn parse_rejects_extras() {
        assert!("1:2:3:4".parse::<NetId64>().is_err());
        assert!("not-an-id".parse::<NetId64>().is_err());
        assert!(":2:3".parse::<NetId64>().is_err());
    }

    #[test]
    fn zero_constant() {
        assert_eq!(NetId64::ZERO.raw(), 0);
        assert_eq!(NetId64::ZERO.kind(), 0);
        assert_eq!(NetId64::ZERO.node(), 0);
        assert_eq!(NetId64::ZERO.counter(), 0);
    }
}
