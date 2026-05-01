//! Bencode Grammar (G_RCO):
//!
//! <element>    ::= <string> | <integer> | <list> | <dictionary>
//! <string>     ::= <len> ":" <bytes>
//! <integer>    ::= "i" <number> "e"
//! <list>       ::= "l" <element>+ "e"
//! <dictionary> ::= "d" (<string> <element>)+ "e"
//!
//! Dictionary keys MUST be in strict lexicographic order (byte-by-byte).
//! No leading zeros. No negative zero. Max recursion depth: 16.

use alloc::vec::Vec;

/// A single Bencode value — the nodes of the `G_RCO` grammar.
///
/// This enum represents the parsed form of a Bencoded payload. It is used
/// for both encoding (constructing values to serialize) and decoding
/// (parsing a binary stream into structured data).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BencodeValue {
    /// A byte string: `<len>:<bytes>`.
    ///
    /// In the RCO protocol, all dictionary keys are byte strings.
    /// String comparison for key ordering uses raw byte values.
    Bytes(Vec<u8>),

    /// A signed integer: `i<number>e`.
    ///
    /// Invariants:
    /// - No leading zeros: `i03e` is invalid.
    /// - No negative zero: `i-0e` is invalid.
    /// - P14-projected rewards are encoded as integers.
    Integer(i128),

    /// An ordered list of elements: `l<element>+e`.
    List(Vec<BencodeValue>),

    /// An ordered dictionary: `d(<string><element>)+e`.
    ///
    /// Invariant: keys MUST be sorted in strict lexicographic order.
    /// Duplicate keys trigger `RcoError::DuplicateKey` (F-11).
    Dict(Vec<(Vec<u8>, BencodeValue)>),
}

impl BencodeValue {
    /// Creates a new byte string value from a string slice.
    #[must_use]
    pub fn string(s: &str) -> Self {
        Self::Bytes(s.as_bytes().to_vec())
    }

    /// Creates a new byte string value from raw bytes.
    #[must_use]
    pub fn bytes(b: &[u8]) -> Self {
        Self::Bytes(b.to_vec())
    }

    /// Creates a new integer value.
    #[must_use]
    pub const fn integer(n: i128) -> Self {
        Self::Integer(n)
    }

    /// Creates a new empty list.
    #[must_use]
    pub fn list() -> Self {
        Self::List(Vec::new())
    }

    /// Creates a new empty dictionary.
    #[must_use]
    pub fn dict() -> Self {
        Self::Dict(Vec::new())
    }

    /// If this is a `Dict`, insert a key-value pair maintaining lexicographic order.
    ///
    /// Returns `Err(())` if this is not a `Dict`.
    pub fn insert(&mut self, key: &[u8], value: BencodeValue) -> Result<(), ()> {
        match self {
            Self::Dict(entries) => {
                // Binary search for insertion point to maintain sorted order
                let pos = entries
                    .binary_search_by(|(k, _)| k.as_slice().cmp(key))
                    .unwrap_or_else(|p| p);

                // Check for duplicate key
                if pos < entries.len() && entries[pos].0.as_slice() == key {
                    return Err(()); // Duplicate key — F-11
                }

                entries.insert(pos, (key.to_vec(), value));
                Ok(())
            }
            _ => Err(()),
        }
    }

    /// If this is a `List`, push a value to the end.
    pub fn push(&mut self, value: BencodeValue) -> Result<(), ()> {
        match self {
            Self::List(items) => {
                items.push(value);
                Ok(())
            }
            _ => Err(()),
        }
    }

    /// Returns the encoded size in bytes (without actually encoding).
    ///
    /// Useful for pre-allocating arena buffers.
    #[must_use]
    pub fn encoded_size(&self) -> usize {
        match self {
            Self::Bytes(b) => {
                // "<len>:<bytes>"
                let len_str = int_to_string_len(b.len() as i128);
                len_str + 1 + b.len()
            }
            Self::Integer(n) => {
                // "i<number>e"
                2 + int_to_string_len(*n)
            }
            Self::List(items) => {
                // "l" + items + "e"
                2 + items.iter().map(Self::encoded_size).sum::<usize>()
            }
            Self::Dict(entries) => {
                // "d" + entries + "e"
                2 + entries
                    .iter()
                    .map(|(k, v)| {
                        let key_val = Self::Bytes(k.clone());
                        key_val.encoded_size() + v.encoded_size()
                    })
                    .sum::<usize>()
            }
        }
    }
}

/// Compute the number of ASCII characters needed to represent an integer.
fn int_to_string_len(n: i128) -> usize {
    if n == 0 {
        return 1;
    }
    let mut len = 0;
    let mut val = n;
    if val < 0 {
        len += 1; // minus sign
        val = val.wrapping_neg();
    }
    while val > 0 {
        len += 1;
        val /= 10;
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dict_insert_maintains_order() {
        let mut dict = BencodeValue::dict();
        dict.insert(b"c", BencodeValue::integer(3)).unwrap();
        dict.insert(b"a", BencodeValue::integer(1)).unwrap();
        dict.insert(b"b", BencodeValue::integer(2)).unwrap();

        if let BencodeValue::Dict(entries) = &dict {
            assert_eq!(entries[0].0, b"a");
            assert_eq!(entries[1].0, b"b");
            assert_eq!(entries[2].0, b"c");
        } else {
            panic!("Expected Dict");
        }
    }

    #[test]
    fn test_dict_insert_rejects_duplicate() {
        let mut dict = BencodeValue::dict();
        dict.insert(b"key", BencodeValue::integer(1)).unwrap();
        assert!(dict.insert(b"key", BencodeValue::integer(2)).is_err());
    }

    #[test]
    fn test_encoded_size_integer() {
        assert_eq!(BencodeValue::integer(0).encoded_size(), 3); // "i0e"
        assert_eq!(BencodeValue::integer(42).encoded_size(), 4); // "i42e"
        assert_eq!(BencodeValue::integer(-1).encoded_size(), 4); // "i-1e"
    }

    #[test]
    fn test_encoded_size_string() {
        assert_eq!(BencodeValue::string("abc").encoded_size(), 5); // "3:abc"
        assert_eq!(BencodeValue::string("").encoded_size(), 2); // "0:"
    }
}
