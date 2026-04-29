//! # Canonical Bencode Decoder
//!
//! Parses a Bencoded binary stream into `BencodeValue` AST nodes.
//!
//! ## Security Properties
//!
//! - **Depth-Limited**: Recursion is hard-capped at `BENCODE_MAX_DEPTH` (16)
//!   to prevent stack-overflow attacks (F-31).
//! - **Strict Grammar**: Rejects all non-canonical representations:
//!   - Leading zeros in integers (`i03e` → error)
//!   - Negative zero (`i-0e` → error)
//!   - Duplicate dictionary keys → `DuplicateKey` (F-11)
//!   - Out-of-order dictionary keys → `KeyOrderViolation`
//! - **No Allocation Amplification**: Buffer allocation is bounded by input size.

use crate::grammar::BencodeValue;
use rco_types::BENCODE_MAX_DEPTH;
use rco_types::error::RcoError;

use alloc::vec::Vec;

/// Strict Bencode decoder enforcing `G_RCO` invariants.
pub struct BencodeDecoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BencodeDecoder<'a> {
    /// Creates a new decoder over the given byte slice.
    #[must_use]
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Decodes a single `BencodeValue` from the input.
    ///
    /// After successful decoding, the internal cursor points past the
    /// decoded value. Call `remaining()` to check for trailing data.
    ///
    /// # Errors
    ///
    /// Returns an `RcoError` if the input violates `G_RCO`.
    pub fn decode(&mut self) -> Result<BencodeValue, RcoError> {
        self.decode_recursive(0)
    }

    /// Returns the number of bytes remaining after the last decode.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// Returns the current byte offset.
    #[must_use]
    pub fn offset(&self) -> usize {
        self.pos
    }

    /// Internal recursive decoder with depth tracking.
    fn decode_recursive(&mut self, depth: usize) -> Result<BencodeValue, RcoError> {
        if depth > BENCODE_MAX_DEPTH {
            return Err(RcoError::RecursionDepthExceeded { depth });
        }

        let byte = self.peek()?;
        match byte {
            b'i' => self.decode_integer(),
            b'l' => self.decode_list(depth),
            b'd' => self.decode_dict(depth),
            b'0'..=b'9' => self.decode_bytes(),
            _ => Err(RcoError::InvalidGrammar {
                offset: self.pos,
                detail: "unexpected byte at start of value",
            }),
        }
    }

    /// Decodes a byte string: `<len>:<bytes>`.
    fn decode_bytes(&mut self) -> Result<BencodeValue, RcoError> {
        let len = self.read_ascii_usize()?;

        // Expect ':'
        let colon = self.next_byte()?;
        if colon != b':' {
            return Err(RcoError::InvalidGrammar {
                offset: self.pos - 1,
                detail: "expected ':' after string length",
            });
        }

        // Read exactly `len` bytes
        if self.pos + len > self.data.len() {
            return Err(RcoError::InvalidGrammar {
                offset: self.pos,
                detail: "string length exceeds remaining input",
            });
        }

        let bytes = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;
        Ok(BencodeValue::Bytes(bytes))
    }

    /// Decodes an integer: `i<number>e`.
    ///
    /// Enforces:
    /// - No leading zeros (except `i0e`)
    /// - No negative zero (`i-0e`)
    fn decode_integer(&mut self) -> Result<BencodeValue, RcoError> {
        let start = self.pos;

        // Consume 'i'
        self.expect_byte(b'i')?;

        let negative = self.peek()? == b'-';
        if negative {
            self.pos += 1;
        }

        // Read digits
        let digit_start = self.pos;
        while self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let digit_end = self.pos;

        if digit_start == digit_end {
            return Err(RcoError::InvalidInteger { offset: start });
        }

        // Check for leading zeros
        let digits = &self.data[digit_start..digit_end];
        if digits.len() > 1 && digits[0] == b'0' {
            return Err(RcoError::InvalidInteger { offset: start });
        }

        // Check for negative zero
        if negative && digits == b"0" {
            return Err(RcoError::InvalidInteger { offset: start });
        }

        // Consume 'e'
        self.expect_byte(b'e')?;

        // Parse the integer
        let mut value: i128 = 0;
        for &d in digits {
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add(i128::from(d - b'0')))
                .ok_or(RcoError::InvalidInteger { offset: start })?;
        }

        if negative {
            value = value
                .checked_neg()
                .ok_or(RcoError::InvalidInteger { offset: start })?;
        }

        Ok(BencodeValue::Integer(value))
    }

    /// Decodes a list: `l<elements>e`.
    fn decode_list(&mut self, depth: usize) -> Result<BencodeValue, RcoError> {
        self.expect_byte(b'l')?;

        let mut items = Vec::new();
        while self.peek()? != b'e' {
            items.push(self.decode_recursive(depth + 1)?);
        }

        self.expect_byte(b'e')?;
        Ok(BencodeValue::List(items))
    }

    /// Decodes a dictionary: `d<key><value>...e`.
    ///
    /// Enforces strict lexicographic key ordering and duplicate key detection.
    fn decode_dict(&mut self, depth: usize) -> Result<BencodeValue, RcoError> {
        self.expect_byte(b'd')?;

        let mut entries = Vec::new();
        let mut last_key: Option<Vec<u8>> = None;

        while self.peek()? != b'e' {
            // Keys must be byte strings
            let key_value = self.decode_bytes()?;
            let key = match key_value {
                BencodeValue::Bytes(k) => k,
                _ => {
                    return Err(RcoError::InvalidGrammar {
                        offset: self.pos,
                        detail: "dictionary key must be a byte string",
                    });
                }
            };

            // Enforce strict lexicographic ordering
            if let Some(ref prev) = last_key {
                match prev.as_slice().cmp(key.as_slice()) {
                    core::cmp::Ordering::Less => {} // Valid: prev < key
                    core::cmp::Ordering::Equal => return Err(RcoError::DuplicateKey),
                    core::cmp::Ordering::Greater => return Err(RcoError::KeyOrderViolation),
                }
            }

            let value = self.decode_recursive(depth + 1)?;
            last_key = Some(key.clone());
            entries.push((key, value));
        }

        self.expect_byte(b'e')?;
        Ok(BencodeValue::Dict(entries))
    }

    // ── Internal Helpers ────────────────────────────────────────────

    /// Peeks at the current byte without advancing.
    #[inline]
    fn peek(&self) -> Result<u8, RcoError> {
        self.data
            .get(self.pos)
            .copied()
            .ok_or(RcoError::InvalidGrammar {
                offset: self.pos,
                detail: "unexpected end of input",
            })
    }

    /// Reads the next byte and advances the cursor.
    #[inline]
    fn next_byte(&mut self) -> Result<u8, RcoError> {
        let b = self.peek()?;
        self.pos += 1;
        Ok(b)
    }

    /// Expects a specific byte, returning an error if it doesn't match.
    fn expect_byte(&mut self, expected: u8) -> Result<(), RcoError> {
        let b = self.next_byte()?;
        if b != expected {
            Err(RcoError::InvalidGrammar {
                offset: self.pos - 1,
                detail: "unexpected byte",
            })
        } else {
            Ok(())
        }
    }

    /// Reads an ASCII decimal unsigned integer (for string lengths).
    fn read_ascii_usize(&mut self) -> Result<usize, RcoError> {
        let start = self.pos;
        let mut value: usize = 0;
        let mut has_digits = false;

        while self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
            let digit = (self.data[self.pos] - b'0') as usize;
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add(digit))
                .ok_or(RcoError::InvalidGrammar {
                    offset: start,
                    detail: "string length overflow",
                })?;
            self.pos += 1;
            has_digits = true;
        }

        if !has_digits {
            return Err(RcoError::InvalidGrammar {
                offset: start,
                detail: "expected digit for string length",
            });
        }

        Ok(value)
    }
}

/// Convenience function: decode a Bencoded byte slice into a `BencodeValue`.
pub fn decode(data: &[u8]) -> Result<BencodeValue, RcoError> {
    let mut decoder = BencodeDecoder::new(data);
    let value = decoder.decode()?;

    // Ensure no trailing data
    if decoder.remaining() > 0 {
        return Err(RcoError::InvalidGrammar {
            offset: decoder.offset(),
            detail: "trailing data after value",
        });
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::encode_to_vec;

    // ── Roundtrip Tests (Encode → Decode → Re-encode) ───────────────

    #[test]
    fn test_roundtrip_integer() {
        let original = BencodeValue::integer(42);
        let encoded = encode_to_vec(&original).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_roundtrip_negative_integer() {
        let original = BencodeValue::integer(-99);
        let encoded = encode_to_vec(&original).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_roundtrip_zero() {
        let original = BencodeValue::integer(0);
        let encoded = encode_to_vec(&original).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_roundtrip_string() {
        let original = BencodeValue::string("hello world");
        let encoded = encode_to_vec(&original).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_roundtrip_empty_string() {
        let original = BencodeValue::string("");
        let encoded = encode_to_vec(&original).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_roundtrip_list() {
        let original = BencodeValue::List(alloc::vec![
            BencodeValue::integer(1),
            BencodeValue::string("two"),
            BencodeValue::integer(3),
        ]);
        let encoded = encode_to_vec(&original).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_roundtrip_dict() {
        let mut original = BencodeValue::dict();
        original.insert(b"alpha", BencodeValue::integer(1)).unwrap();
        original
            .insert(b"beta", BencodeValue::string("two"))
            .unwrap();
        original.insert(b"gamma", BencodeValue::integer(3)).unwrap();

        let encoded = encode_to_vec(&original).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_roundtrip_nested_dict() {
        let mut inner = BencodeValue::dict();
        inner.insert(b"x", BencodeValue::integer(10)).unwrap();

        let mut outer = BencodeValue::dict();
        outer.insert(b"inner", inner).unwrap();
        outer.insert(b"val", BencodeValue::string("test")).unwrap();

        let encoded = encode_to_vec(&outer).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(outer, decoded);
    }

    #[test]
    fn test_roundtrip_p14_reward() {
        // Simulates a P14-projected reward: π × 10^14
        let reward = 314_159_265_358_979_3_i128;
        let original = BencodeValue::integer(reward);
        let encoded = encode_to_vec(&original).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    // ── Strict Grammar Rejection ────────────────────────────────────

    #[test]
    fn test_reject_leading_zeros() {
        let result = decode(b"i03e");
        assert!(matches!(result, Err(RcoError::InvalidInteger { .. })));
    }

    #[test]
    fn test_reject_negative_zero() {
        let result = decode(b"i-0e");
        assert!(matches!(result, Err(RcoError::InvalidInteger { .. })));
    }

    #[test]
    fn test_reject_unordered_dict_keys() {
        // "d1:b1:x1:a1:ye" — keys "b" > "a" violates ordering
        let result = decode(b"d1:b1:x1:a1:ye");
        assert!(matches!(result, Err(RcoError::KeyOrderViolation)));
    }

    #[test]
    fn test_reject_duplicate_dict_keys() {
        // "d1:a1:x1:a1:ye" — duplicate key "a"
        let result = decode(b"d1:a1:x1:a1:ye");
        assert!(matches!(result, Err(RcoError::DuplicateKey)));
    }

    #[test]
    fn test_reject_trailing_data() {
        let result = decode(b"i42eextra");
        assert!(matches!(result, Err(RcoError::InvalidGrammar { .. })));
    }

    #[test]
    fn test_reject_empty_input() {
        let result = decode(b"");
        assert!(matches!(result, Err(RcoError::InvalidGrammar { .. })));
    }

    // ── Determinism: Re-encoding Produces Identical Bytes ───────────

    #[test]
    fn test_decode_reencode_deterministic() {
        let input = b"d5:alphai1e4:beta4:test5:gammai3ee";
        let decoded = decode(input).unwrap();
        let reencoded = encode_to_vec(&decoded).unwrap();
        assert_eq!(input.as_slice(), reencoded.as_slice());
    }
}
