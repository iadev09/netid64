#![cfg_attr(not(feature = "std"), no_std)]
// All comments in English (per your rule).

use core::{fmt, str::FromStr};

/// 64-bit ID layout: [KIND:8][NODE:16][COUNTER:40] big-endian semantics.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NetId64(pub u64);

impl NetId64 {
    #[inline]
    pub const fn raw(self) -> u64 {
        self.0
    }
    #[inline]
    pub const fn kind(self) -> u8 {
        (self.0 >> 56) as u8
    }
    #[inline]
    pub const fn node(self) -> u16 {
        ((self.0 >> 40) & 0xFFFF) as u16
    }
    #[inline]
    pub const fn counter(self) -> u64 {
        self.0 & 0xFF_FFFF_FFFF
    }

    /// Construct from fields (counter truncated to 40 bits).
    #[inline]
    pub const fn make(kind: u8, node: u16, counter: u64) -> Self {
        Self(((kind as u64) << 56) | ((node as u64) << 40) | (counter & 0xFF_FFFF_FFFF))
    }

    /// Big-endian bytes of raw u64.
    #[inline]
    pub const fn to_be_bytes(self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    #[inline]
    pub const fn from_be_bytes(b: [u8; 8]) -> Self {
        Self(u64::from_be_bytes(b))
    }

    /// Canonical string: "k:node:ctr" in decimals (no leading zeros).
    pub fn to_triple(self) -> Triple {
        Triple {
            kind: self.kind(),
            node: self.node(),
            counter: self.counter(),
        }
    }
}

pub struct Triple {
    pub kind: u8,
    pub node: u16,
    pub counter: u64,
}

impl fmt::Display for NetId64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = self.to_triple();
        write!(f, "{}:{}:{}", t.kind, t.node, t.counter)
    }
}

impl fmt::Debug for NetId64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Debug carries both triple and hex raw for logs.
        write!(f, "NetId64({} | 0x{:016X})", self, self.0)
    }
}

impl FromStr for NetId64 {
    type Err = ParseError;

    /// Accepts "k:node:ctr" (decimal) or "0x..." (raw u64).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(hex) = s.strip_prefix("0x") {
            let v = u64::from_str_radix(hex, 16).map_err(|_| ParseError)?;
            return Ok(Self(v));
        }
        let mut it = s.split(':');
        let k = it
            .next()
            .ok_or(ParseError)?
            .parse::<u8>()
            .map_err(|_| ParseError)?;
        let n = it
            .next()
            .ok_or(ParseError)?
            .parse::<u16>()
            .map_err(|_| ParseError)?;
        let c = it
            .next()
            .ok_or(ParseError)?
            .parse::<u64>()
            .map_err(|_| ParseError)?;
        if it.next().is_some() {
            return Err(ParseError);
        }
        Ok(Self::make(k, n, c))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid NetId64")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}

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
    fn parse_and_display() {
        let id: NetId64 = "7:42:999".parse().unwrap();
        assert_eq!(id.kind(), 7);
        assert_eq!(id.node(), 42);
        assert_eq!(id.counter(), 999);
        assert_eq!(id.to_string(), "7:42:999");
        let hex = format!("{id:?}"); // Debug prints hex too
        assert!(hex.contains("0x"));
    }
}
