//! # Bencode FFI — Deterministic Serialization for Julia

use crate::error_codes::RcoStatus;
use rco_bencode::encoder::BencodeEncoder;
use rco_bencode::grammar::BencodeValue;

/// Encodes a single i128 integer to Bencode format.
///
/// # Safety
/// `out_buf` must point to `buf_len` writable bytes. `out_written` must be non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_bencode_encode_integer(
    value: i128,
    out_buf: *mut u8,
    buf_len: usize,
    out_written: *mut usize,
) -> i32 {
    if out_buf.is_null() || out_written.is_null() {
        return RcoStatus::NullPointer as i32;
    }
    let buf = unsafe { core::slice::from_raw_parts_mut(out_buf, buf_len) };
    let node = BencodeValue::Integer(value);
    let mut encoder = BencodeEncoder::new(buf);
    match encoder.encode(&node) {
        Ok(written) => {
            unsafe { *out_written = written; }
            RcoStatus::Ok as i32
        }
        Err(ref e) => RcoStatus::from_error(e) as i32,
    }
}

/// Encodes a byte string to Bencode format.
///
/// # Safety
/// All pointers must be valid and non-null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_bencode_encode_string(
    data: *const u8, data_len: usize,
    out_buf: *mut u8, buf_len: usize,
    out_written: *mut usize,
) -> i32 {
    if data.is_null() || out_buf.is_null() || out_written.is_null() {
        return RcoStatus::NullPointer as i32;
    }
    let data_slice = unsafe { core::slice::from_raw_parts(data, data_len) };
    let buf = unsafe { core::slice::from_raw_parts_mut(out_buf, buf_len) };
    let node = BencodeValue::Bytes(data_slice.to_vec());
    let mut encoder = BencodeEncoder::new(buf);
    match encoder.encode(&node) {
        Ok(written) => {
            unsafe { *out_written = written; }
            RcoStatus::Ok as i32
        }
        Err(ref e) => RcoStatus::from_error(e) as i32,
    }
}

/// Returns Bencode-encoded size of an integer (for buffer pre-allocation).
#[unsafe(no_mangle)]
pub extern "C" fn rco_bencode_integer_size(value: i128) -> usize {
    BencodeValue::Integer(value).encoded_size()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_encode_integer() {
        let mut buf = [0u8; 64];
        let mut written: usize = 0;
        let status = unsafe {
            rco_bencode_encode_integer(42, buf.as_mut_ptr(), buf.len(), &mut written)
        };
        assert_eq!(status, 0);
        assert_eq!(&buf[..written], b"i42e");
    }

    #[test]
    fn test_ffi_encode_string() {
        let data = b"hello";
        let mut buf = [0u8; 64];
        let mut written: usize = 0;
        let status = unsafe {
            rco_bencode_encode_string(
                data.as_ptr(), data.len(),
                buf.as_mut_ptr(), buf.len(), &mut written,
            )
        };
        assert_eq!(status, 0);
        assert_eq!(&buf[..written], b"5:hello");
    }

    #[test]
    fn test_ffi_encode_null_output() {
        let mut written: usize = 0;
        let status = unsafe {
            rco_bencode_encode_integer(42, core::ptr::null_mut(), 64, &mut written)
        };
        assert_eq!(status, RcoStatus::NullPointer as i32);
    }

    #[test]
    fn test_ffi_integer_size() {
        assert_eq!(rco_bencode_integer_size(0), 3);  // "i0e"
        assert_eq!(rco_bencode_integer_size(42), 4); // "i42e"
    }
}
