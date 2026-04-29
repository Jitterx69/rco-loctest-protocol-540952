//! # Chain FFI — RML Chain Operations for Julia

use crate::error_codes::RcoStatus;
use rco_merkle::chain::{RmlChain, compute_hash, compute_chained_hash};
use rco_types::HASH_SIZE;

/// Opaque handle to an RML chain (Box'd pointer).
pub type ChainHandle = *mut RmlChain;

/// Creates a new RML chain from a genesis block's encoded bytes.
///
/// # Safety
/// `genesis_data` must point to `genesis_len` valid bytes.
/// `out_handle` must be a valid non-null pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_chain_create(
    genesis_data: *const u8, genesis_len: usize,
    out_handle: *mut ChainHandle,
) -> i32 {
    if genesis_data.is_null() || out_handle.is_null() {
        return RcoStatus::NullPointer as i32;
    }
    let data = unsafe { core::slice::from_raw_parts(genesis_data, genesis_len) };
    let genesis_hash = compute_hash(data);
    let chain = Box::new(RmlChain::from_genesis(genesis_hash));
    unsafe { *out_handle = Box::into_raw(chain); }
    RcoStatus::Ok as i32
}

/// Extends the chain with a new batch.
///
/// # Safety
/// `handle` must be a valid chain handle from `rco_chain_create`.
/// `out_anchor` must point to 32 writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_chain_extend(
    handle: ChainHandle,
    batch_index: u64,
    encoded_batch: *const u8, batch_len: usize,
    out_anchor: *mut u8,
) -> i32 {
    if handle.is_null() || encoded_batch.is_null() || out_anchor.is_null() {
        return RcoStatus::NullPointer as i32;
    }
    let chain = unsafe { &mut *handle };
    let batch = unsafe { core::slice::from_raw_parts(encoded_batch, batch_len) };
    match chain.extend(batch_index, batch) {
        Ok(anchor) => {
            let out = unsafe { core::slice::from_raw_parts_mut(out_anchor, HASH_SIZE) };
            out.copy_from_slice(&anchor.hash);
            RcoStatus::Ok as i32
        }
        Err(ref e) => RcoStatus::from_error(e) as i32,
    }
}

/// Returns the current head hash of the chain.
///
/// # Safety
/// `handle` must be valid. `out_hash` must point to 32 writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_chain_head_hash(
    handle: ChainHandle,
    out_hash: *mut u8,
) -> i32 {
    if handle.is_null() || out_hash.is_null() {
        return RcoStatus::NullPointer as i32;
    }
    let chain = unsafe { &*handle };
    let out = unsafe { core::slice::from_raw_parts_mut(out_hash, HASH_SIZE) };
    out.copy_from_slice(chain.head_hash());
    RcoStatus::Ok as i32
}

/// Returns the chain length.
///
/// # Safety
/// `handle` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_chain_len(handle: ChainHandle) -> u64 {
    if handle.is_null() { return 0; }
    let chain = unsafe { &*handle };
    chain.len()
}

/// Destroys a chain handle, freeing its memory.
///
/// # Safety
/// `handle` must have been created by `rco_chain_create` and not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_chain_destroy(handle: ChainHandle) {
    if !handle.is_null() {
        let _ = unsafe { Box::from_raw(handle) };
    }
}

/// Computes Keccak-256 hash of arbitrary data.
///
/// # Safety
/// `data` must point to `data_len` bytes. `out_hash` must point to 32 bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_keccak256(
    data: *const u8, data_len: usize,
    out_hash: *mut u8,
) -> i32 {
    if data.is_null() || out_hash.is_null() {
        return RcoStatus::NullPointer as i32;
    }
    let input = unsafe { core::slice::from_raw_parts(data, data_len) };
    let hash = compute_hash(input);
    let out = unsafe { core::slice::from_raw_parts_mut(out_hash, HASH_SIZE) };
    out.copy_from_slice(&hash);
    RcoStatus::Ok as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_chain_lifecycle() {
        let mut handle: ChainHandle = core::ptr::null_mut();

        // Create
        let status = unsafe {
            rco_chain_create(b"genesis".as_ptr(), 7, &mut handle)
        };
        assert_eq!(status, 0);
        assert!(!handle.is_null());

        // Check length
        assert_eq!(unsafe { rco_chain_len(handle) }, 1);

        // Extend
        let mut anchor = [0u8; 32];
        let status = unsafe {
            rco_chain_extend(handle, 1, b"batch_1".as_ptr(), 7, anchor.as_mut_ptr())
        };
        assert_eq!(status, 0);
        assert_ne!(anchor, [0u8; 32]);
        assert_eq!(unsafe { rco_chain_len(handle) }, 2);

        // Head hash
        let mut head = [0u8; 32];
        let status = unsafe { rco_chain_head_hash(handle, head.as_mut_ptr()) };
        assert_eq!(status, 0);
        assert_eq!(head, anchor);

        // Destroy
        unsafe { rco_chain_destroy(handle); }
    }

    #[test]
    fn test_ffi_chain_rejects_non_sequential() {
        let mut handle: ChainHandle = core::ptr::null_mut();
        unsafe { rco_chain_create(b"gen".as_ptr(), 3, &mut handle); }

        let mut anchor = [0u8; 32];
        let status = unsafe {
            rco_chain_extend(handle, 5, b"bad".as_ptr(), 3, anchor.as_mut_ptr())
        };
        assert_eq!(status, RcoStatus::LinkageGap as i32);

        unsafe { rco_chain_destroy(handle); }
    }

    #[test]
    fn test_ffi_keccak256() {
        let mut hash = [0u8; 32];
        let status = unsafe {
            rco_keccak256(b"test".as_ptr(), 4, hash.as_mut_ptr())
        };
        assert_eq!(status, 0);
        assert_ne!(hash, [0u8; 32]);
    }

    #[test]
    fn test_ffi_chain_null_guards() {
        assert_eq!(unsafe { rco_chain_len(core::ptr::null_mut()) }, 0);

        let mut handle: ChainHandle = core::ptr::null_mut();
        let status = unsafe {
            rco_chain_create(core::ptr::null(), 0, &mut handle)
        };
        assert_eq!(status, RcoStatus::NullPointer as i32);
    }
}
