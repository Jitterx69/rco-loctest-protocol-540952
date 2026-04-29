//! # Canonical Bencode Encoder
//!
//! Serializes `BencodeValue` into the canonical binary form defined by `G_RCO`.
//!
//! ## Architecture
//!
//! The encoder writes into a caller-provided `&mut [u8]` buffer (arena).
//! No heap allocation occurs during encoding — the buffer is filled
//! monotonically from index 0 upward.
//!
//! ## Invariant Enforcement
//!
//! - Dictionary keys are validated for strict lexicographic order at encode time.
//! - Integer encoding rejects leading zeros and negative zero.
//! - Recursion depth is tracked and hard-capped at `BENCODE_MAX_DEPTH` (16).
//!
//! ## Cross-Platform Guarantee (TC-01)
//!
//! The encoder produces bit-identical output regardless of platform,
//! endianness, or Rust compiler version, because:
//! 1. All operations are pure byte manipulation — no floats, no locale.
//! 2. Integer-to-ASCII conversion uses a deterministic div/mod loop.
//! 3. Dictionary key order is enforced at the type level (`BencodeValue::insert`).

use crate::grammar::BencodeValue;
use crate::sort;
use rco_types::BENCODE_MAX_DEPTH;
use rco_types::error::RcoError;

/// Arena-style Bencode encoder that writes into a pre-allocated buffer.
///
/// # Usage
///
/// ```ignore
/// let mut buf = [0u8; 4096];
/// let mut encoder = BencodeEncoder::new(&mut buf);
/// let n = encoder.encode(&value)?;
/// let encoded = &buf[..n];
/// ```
pub struct BencodeEncoder<'a> {
    buf: &'a mut [u8],
    cursor: usize,
}

impl<'a> BencodeEncoder<'a> {
    /// Creates a new encoder writing into the given buffer.
    #[must_use]
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, cursor: 0 }
    }

    /// Encodes a `BencodeValue` into the buffer.
    ///
    /// Returns the number of bytes written on success.
    ///
    /// # Errors
    ///
    /// - `RcoError::BufferTooSmall` if the buffer cannot hold the output.
    /// - `RcoError::RecursionDepthExceeded` if dict/list nesting exceeds 16.
    /// - `RcoError::KeyOrderViolation` if dictionary keys are unordered.
    /// - `RcoError::DuplicateKey` if duplicate keys are detected.
    pub fn encode(&mut self, value: &BencodeValue) -> Result<usize, RcoError> {
        self.cursor = 0;
        self.encode_recursive(value, 0)?;
        Ok(self.cursor)
    }

    /// Internal recursive encoder with depth tracking.
    fn encode_recursive(&mut self, value: &BencodeValue, depth: usize) -> Result<(), RcoError> {
        if depth > BENCODE_MAX_DEPTH {
            return Err(RcoError::RecursionDepthExceeded { depth });
        }

        match value {
            BencodeValue::Bytes(b) => self.encode_bytes(b),
            BencodeValue::Integer(n) => self.encode_integer(*n),
            BencodeValue::List(items) => self.encode_list(items, depth),
            BencodeValue::Dict(entries) => self.encode_dict(entries, depth),
        }
    }

    /// Encodes a byte string: `<len>:<bytes>`.
    fn encode_bytes(&mut self, data: &[u8]) -> Result<(), RcoError> {
        // Write the length prefix
        self.write_integer_ascii(data.len() as i128, false)?;
        self.write_byte(b':')?;
        self.write_bytes(data)?;
        Ok(())
    }

    /// Encodes a signed integer: `i<number>e`.
    ///
    /// Invariants enforced:
    /// - No leading zeros: only `i0e` is valid for zero.
    /// - Negative zero is impossible (i128 has no negative zero).
    fn encode_integer(&mut self, n: i128) -> Result<(), RcoError> {
        self.write_byte(b'i')?;
        self.write_integer_ascii(n, true)?;
        self.write_byte(b'e')?;
        Ok(())
    }

    /// Encodes a list: `l<elements>e`.
    fn encode_list(&mut self, items: &[BencodeValue], depth: usize) -> Result<(), RcoError> {
        self.write_byte(b'l')?;
        for item in items {
            self.encode_recursive(item, depth + 1)?;
        }
        self.write_byte(b'e')?;
        Ok(())
    }

    /// Encodes a dictionary: `d<key><value>...e`.
    ///
    /// Validates lexicographic key ordering before emitting.
    fn encode_dict(
        &mut self,
        entries: &[(alloc::vec::Vec<u8>, BencodeValue)],
        depth: usize,
    ) -> Result<(), RcoError> {
        // Validate key ordering
        let keys: alloc::vec::Vec<&[u8]> = entries.iter().map(|(k, _)| k.as_slice()).collect();
        sort::validate_key_order(&keys)?;

        self.write_byte(b'd')?;
        for (key, value) in entries {
            // Keys are always byte strings
            self.encode_bytes(key)?;
            self.encode_recursive(value, depth + 1)?;
        }
        self.write_byte(b'e')?;
        Ok(())
    }

    /// Writes a single byte to the buffer.
    #[inline(always)]
    fn write_byte(&mut self, b: u8) -> Result<(), RcoError> {
        if self.cursor >= self.buf.len() {
            return Err(RcoError::BufferTooSmall {
                required: self.cursor + 1,
                available: self.buf.len(),
            });
        }
        self.buf[self.cursor] = b;
        self.cursor += 1;
        Ok(())
    }

    /// Writes a byte slice to the buffer.
    #[inline]
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), RcoError> {
        let end = self.cursor + data.len();
        if end > self.buf.len() {
            return Err(RcoError::BufferTooSmall {
                required: end,
                available: self.buf.len(),
            });
        }
        self.buf[self.cursor..end].copy_from_slice(data);
        self.cursor = end;
        Ok(())
    }

    /// Writes an integer in ASCII decimal representation.
    ///
    /// `allow_negative`: if false, treats the value as unsigned (for string lengths).
    fn write_integer_ascii(&mut self, n: i128, allow_negative: bool) -> Result<(), RcoError> {
        if n == 0 {
            return self.write_byte(b'0');
        }

        let mut val = n;
        if val < 0 {
            if allow_negative {
                self.write_byte(b'-')?;
                // Handle i128::MIN carefully
                val = val.wrapping_neg();
            } else {
                // String lengths can't be negative — this is a bug
                return Err(RcoError::InvalidGrammar {
                    offset: self.cursor,
                    detail: "negative string length",
                });
            }
        }

        // Write digits in reverse order, then swap
        let start = self.cursor;
        let mut tmp = val;
        while tmp > 0 {
            let digit = (tmp % 10) as u8;
            self.write_byte(b'0' + digit)?;
            tmp /= 10;
        }
        let end = self.cursor;

        // Reverse the digits in-place
        let digits = &mut self.buf[start..end];
        digits.reverse();

        Ok(())
    }
}

/// Convenience function: encode a `BencodeValue` into a new `Vec<u8>`.
///
/// This allocates a buffer sized to `value.encoded_size()` and encodes into it.
/// For hot-path use, prefer `BencodeEncoder` with a pre-allocated arena.
pub fn encode_to_vec(value: &BencodeValue) -> Result<alloc::vec::Vec<u8>, RcoError> {
    let size = value.encoded_size();
    let mut buf = alloc::vec![0u8; size];
    let mut encoder = BencodeEncoder::new(&mut buf);
    let written = encoder.encode(value)?;
    buf.truncate(written);
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    fn encode(value: &BencodeValue) -> Vec<u8> {
        encode_to_vec(value).expect("encoding failed")
    }

    // ── Integer Encoding ────────────────────────────────────────────

    #[test]
    fn test_encode_integer_zero() {
        assert_eq!(encode(&BencodeValue::integer(0)), b"i0e");
    }

    #[test]
    fn test_encode_integer_positive() {
        assert_eq!(encode(&BencodeValue::integer(42)), b"i42e");
    }

    #[test]
    fn test_encode_integer_negative() {
        assert_eq!(encode(&BencodeValue::integer(-7)), b"i-7e");
    }

    #[test]
    fn test_encode_integer_large() {
        // P14 reward: 3.14159 × 10^14 = 314159000000000000
        let p14 = 314_159_000_000_000_000_i128;
        let encoded = encode(&BencodeValue::integer(p14));
        assert_eq!(encoded, b"i314159000000000000e");
    }

    #[test]
    fn test_encode_integer_i128_max() {
        let max = i128::MAX;
        let encoded = encode(&BencodeValue::integer(max));
        let expected = alloc::format!("i{max}e");
        assert_eq!(encoded, expected.as_bytes());
    }

    // ── String Encoding ─────────────────────────────────────────────

    #[test]
    fn test_encode_string_empty() {
        assert_eq!(encode(&BencodeValue::string("")), b"0:");
    }

    #[test]
    fn test_encode_string_simple() {
        assert_eq!(encode(&BencodeValue::string("spam")), b"4:spam");
    }

    #[test]
    fn test_encode_string_binary() {
        let data = &[0x00, 0xFF, 0x42];
        assert_eq!(
            encode(&BencodeValue::bytes(data)),
            &[b'3', b':', 0x00, 0xFF, 0x42]
        );
    }

    // ── List Encoding ───────────────────────────────────────────────

    #[test]
    fn test_encode_list_empty() {
        assert_eq!(encode(&BencodeValue::List(alloc::vec![])), b"le");
    }

    #[test]
    fn test_encode_list_mixed() {
        let list = BencodeValue::List(alloc::vec![
            BencodeValue::string("hello"),
            BencodeValue::integer(123),
        ]);
        assert_eq!(encode(&list), b"l5:helloi123ee");
    }

    // ── Dictionary Encoding ─────────────────────────────────────────

    #[test]
    fn test_encode_dict_sorted() {
        let mut dict = BencodeValue::dict();
        dict.insert(b"b", BencodeValue::integer(2)).unwrap();
        dict.insert(b"a", BencodeValue::integer(1)).unwrap();
        // Keys should be emitted in lexicographic order: a, b
        assert_eq!(encode(&dict), b"d1:ai1e1:bi2ee");
    }

    #[test]
    fn test_encode_dict_nested() {
        let mut inner = BencodeValue::dict();
        inner.insert(b"x", BencodeValue::integer(10)).unwrap();

        let mut outer = BencodeValue::dict();
        outer.insert(b"inner", inner).unwrap();
        outer.insert(b"val", BencodeValue::string("test")).unwrap();

        let encoded = encode(&outer);
        assert_eq!(encoded, b"d5:innerd1:xi10ee3:val4:teste");
    }

    // ── RCO-Specific: Telemetry Batch Encoding ──────────────────────

    #[test]
    fn test_encode_rco_batch() {
        // Simulates a minimal telemetry batch with P14-projected reward
        let mut batch = BencodeValue::dict();
        batch
            .insert(b"batch_index", BencodeValue::integer(42))
            .unwrap();
        batch
            .insert(b"reward", BencodeValue::integer(314_159_265_358_979_3_i128))
            .unwrap();
        batch
            .insert(
                b"run_uuid",
                BencodeValue::string("550e8400-e29b-41d4-a716-446655440000"),
            )
            .unwrap();

        let encoded = encode(&batch);

        // Verify it starts with 'd' and ends with 'e'
        assert_eq!(encoded[0], b'd');
        assert_eq!(*encoded.last().unwrap(), b'e');

        // Verify key ordering: batch_index < reward < run_uuid
        let s = alloc::string::String::from_utf8_lossy(&encoded);
        let bi_pos = s.find("11:batch_index").unwrap();
        let rw_pos = s.find("6:reward").unwrap();
        let ru_pos = s.find("8:run_uuid").unwrap();
        assert!(bi_pos < rw_pos);
        assert!(rw_pos < ru_pos);
    }

    // ── Error Cases ─────────────────────────────────────────────────

    #[test]
    fn test_encode_buffer_too_small() {
        let value = BencodeValue::string("hello");
        let mut buf = [0u8; 3]; // Too small for "5:hello"
        let mut encoder = BencodeEncoder::new(&mut buf);
        let result = encoder.encode(&value);
        assert!(matches!(result, Err(RcoError::BufferTooSmall { .. })));
    }

    #[test]
    fn test_encode_depth_limit() {
        // Build a deeply nested list exceeding depth 16
        let mut value = BencodeValue::integer(0);
        for _ in 0..18 {
            value = BencodeValue::List(alloc::vec![value]);
        }
        let mut buf = [0u8; 1024];
        let mut encoder = BencodeEncoder::new(&mut buf);
        let result = encoder.encode(&value);
        assert!(matches!(
            result,
            Err(RcoError::RecursionDepthExceeded { .. })
        ));
    }

    // ── Determinism: Repeated Encoding ──────────────────────────────

    #[test]
    fn test_encode_deterministic_repeated() {
        let mut dict = BencodeValue::dict();
        dict.insert(b"key", BencodeValue::integer(42)).unwrap();

        let a = encode(&dict);
        let b = encode(&dict);
        assert_eq!(a, b, "Repeated encoding must be bit-identical");
    }
}
