//! Identity FFI — Sovereign Identity Export/Import
//!
//! Exposes the identity migration logic to Julia.

use std::os::raw::{c_int, c_uint};
use rco_enclave::identity_export::{IdentityExportController, SovereignIdentityPackage};
use rco_enclave::closure::SelfAttestingRoot;
use crate::error_codes::RcoStatus;

/// Exports a Sovereign Identity Root to a byte buffer.
///
/// # Safety
/// Caller must provide valid pointers for `root_hash` and `signature`.
/// `out_payload` must be at least 96 bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_identity_export(
    root_hash: *const u8,
    signature: *const u8,
    threshold: c_uint,
    out_payload: *mut u8,
    out_identity_id: *mut u8,
) -> c_int {
    if root_hash.is_null() || signature.is_null() || out_payload.is_null() || out_identity_id.is_null() {
        return RcoStatus::NullPointer as c_int;
    }

    let mut hash = [0u8; 32];
    let mut sig = [0u8; 64];
    
    unsafe {
        std::ptr::copy_nonoverlapping(root_hash, hash.as_mut_ptr(), 32);
        std::ptr::copy_nonoverlapping(signature, sig.as_mut_ptr(), 64);
    }

    let root = SelfAttestingRoot {
        root_hash: hash,
        signature: sig,
        timestamp: 0,
    };

    let controller = IdentityExportController::new(root);
    let package = controller.export_sovereign_identity(threshold);

    unsafe {
        std::ptr::copy_nonoverlapping(package.payload.as_ptr(), out_payload, 96);
        std::ptr::copy_nonoverlapping(package.identity_id.as_ptr(), out_identity_id, 32);
    }

    RcoStatus::Ok as c_int
}

/// Reassembles a Sovereign Identity Root from a byte buffer.
///
/// # Safety
/// Caller must provide valid pointers for `payload` (96 bytes) and `identity_id`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rco_identity_reassemble(
    payload: *const u8,
    identity_id: *const u8,
    threshold: c_uint,
    out_root_hash: *mut u8,
    out_signature: *mut u8,
) -> c_int {
    if payload.is_null() || identity_id.is_null() || out_root_hash.is_null() || out_signature.is_null() {
        return RcoStatus::NullPointer as c_int;
    }

    let mut id_id = [0u8; 32];
    unsafe { std::ptr::copy_nonoverlapping(identity_id, id_id.as_mut_ptr(), 32); }

    let mut payload_vec = vec![0u8; 96];
    unsafe { std::ptr::copy_nonoverlapping(payload, payload_vec.as_mut_ptr(), 96); }

    let package = SovereignIdentityPackage {
        payload: payload_vec,
        identity_id: id_id,
        threshold,
    };

    let reassembled = IdentityExportController::reassemble_identity(&package);

    unsafe {
        std::ptr::copy_nonoverlapping(reassembled.root_hash.as_ptr(), out_root_hash, 32);
        std::ptr::copy_nonoverlapping(reassembled.signature.as_ptr(), out_signature, 64);
    }

    RcoStatus::Ok as c_int
}
