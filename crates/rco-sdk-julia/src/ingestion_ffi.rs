//! # Ingestion FFI — Atomic Ingestion Gateway for Julia
//!
//! Exposes the `IngestionPipeline` across the C-ABI boundary.

use crate::error_codes::RcoStatus;
use rco_ingestion::pipeline::{IngestionPipeline, IngestionResult};
use rco_ingestion::backpressure::BackpressureConfig;
use rco_types::HASH_SIZE;
use std::path::Path;
use std::ffi::CStr;
use std::os::raw::c_char;

/// Opaque handle to an IngestionPipeline (Box'd pointer).
pub type IngestionHandle = *mut IngestionPipeline;

/// Opens an ingestion pipeline.
///
/// # Safety
/// - `wal_path` must be a valid, null-terminated C string.
/// - `genesis_hash` must point to 32 readable bytes.
/// - `out_handle` must be a valid non-null pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_ingestion_open(
    wal_path: *const c_char,
    genesis_hash: *const u8,
    out_handle: *mut IngestionHandle,
) -> i32 {
    if wal_path.is_null() || genesis_hash.is_null() || out_handle.is_null() {
        return RcoStatus::NullPointer as i32;
    }

    // SAFETY: Caller guarantees wal_path is a valid C string.
    let path_str = match unsafe { CStr::from_ptr(wal_path).to_str() } {
        Ok(s) => s,
        Err(_) => return RcoStatus::InvalidArgument as i32,
    };

    let path = Path::new(path_str);
    
    // SAFETY: Caller guarantees genesis_hash points to 32 bytes.
    let mut hash = [0u8; HASH_SIZE];
    unsafe { core::ptr::copy_nonoverlapping(genesis_hash, hash.as_mut_ptr(), HASH_SIZE); }

    match IngestionPipeline::open_default(path, hash) {
        Ok(pipeline) => {
            unsafe { *out_handle = Box::into_raw(Box::new(pipeline)); }
            RcoStatus::Ok as i32
        }
        Err(ref e) => RcoStatus::from_error(e) as i32,
    }
}

/// Ingests a batch.
///
/// # Safety
/// - `handle` must be a valid handle from `rco_ingestion_open`.
/// - `batch_data` must point to `batch_len` bytes.
/// - `out_anchor` must point to 32 writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_ingestion_ingest(
    handle: IngestionHandle,
    batch_index: u64,
    batch_data: *const u8,
    batch_len: usize,
    out_anchor: *mut u8,
) -> i32 {
    if handle.is_null() || batch_data.is_null() || out_anchor.is_null() {
        return RcoStatus::NullPointer as i32;
    }

    let pipeline = unsafe { &mut *handle };
    let data = unsafe { core::slice::from_raw_parts(batch_data, batch_len) };

    match pipeline.ingest(batch_index, data) {
        Ok(result) => {
            let out = unsafe { core::slice::from_raw_parts_mut(out_anchor, HASH_SIZE) };
            out.copy_from_slice(&result.anchor);
            RcoStatus::Ok as i32
        }
        Err(ref e) => RcoStatus::from_error(e) as i32,
    }
}

/// Destroys an ingestion pipeline handle.
///
/// # Safety
/// `handle` must be valid and not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_ingestion_destroy(handle: IngestionHandle) {
    if !handle.is_null() {
        let _ = unsafe { Box::from_raw(handle) };
    }
}

/// Returns the next expected batch index.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_ingestion_next_index(handle: IngestionHandle) -> u64 {
    if handle.is_null() { return 0; }
    let pipeline = unsafe { &*handle };
    pipeline.next_batch_index()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rco_merkle::chain::compute_hash;
    use tempfile::TempDir;
    use std::ffi::CString;

    #[test]
    fn test_ffi_ingestion_lifecycle() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("ffi_test.wal");
        let wal_path_c = CString::new(wal_path.to_str().unwrap()).unwrap();
        let genesis_hash = compute_hash(b"genesis_data");

        let mut handle: IngestionHandle = core::ptr::null_mut();
        
        // Open
        let status = unsafe {
            rco_ingestion_open(
                wal_path_c.as_ptr(),
                genesis_hash.as_ptr(),
                &mut handle,
            )
        };
        assert_eq!(status, 0);
        assert!(!handle.is_null());

        // Next index
        assert_eq!(unsafe { rco_ingestion_next_index(handle) }, 1);

        // Ingest
        let batch_data = b"i42e";
        let mut anchor = [0u8; 32];
        let status = unsafe {
            rco_ingestion_ingest(
                handle,
                1,
                batch_data.as_ptr(),
                batch_data.len(),
                anchor.as_mut_ptr(),
            )
        };
        assert_eq!(status, 0);
        assert_ne!(anchor, [0u8; 32]);
        assert_eq!(unsafe { rco_ingestion_next_index(handle) }, 2);

        // Destroy
        unsafe { rco_ingestion_destroy(handle); }
    }
}
