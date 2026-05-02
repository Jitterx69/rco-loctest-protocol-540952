use libc::{size_t, c_double, uint8_t, int64_t};
use rco_p14::projection::project_p14_batch;
use rco_quorum::por::construct_por_message;
use group::GroupEncoding;
use rco_enclave::hardware::{detect_tee, TEEType};

#[unsafe(no_mangle)]
pub extern "C" fn rco_project_p14_batch(
    input: *const c_double,
    count: size_t,
    output_high: *mut int64_t,
    output_low: *mut int64_t,
) -> i32 {
    if input.is_null() || output_high.is_null() || output_low.is_null() {
        return -1;
    }

    let input_slice = unsafe { std::slice::from_raw_parts(input, count) };
    let mut results = vec![0i128; count];
    
    if let Err(_) = project_p14_batch(input_slice, &mut results) {
        return -2;
    }

    let out_h = unsafe { std::slice::from_raw_parts_mut(output_high, count) };
    let out_l = unsafe { std::slice::from_raw_parts_mut(output_low, count) };

    for i in 0..count {
        out_h[i] = (results[i] >> 64) as i64;
        out_l[i] = (results[i] & 0xFFFFFFFFFFFFFFFF) as i64;
    }

    0 // Success
}

#[unsafe(no_mangle)]
pub extern "C" fn rco_construct_por_message(
    weight_hash: *const uint8_t,
    merkle_link: *const uint8_t,
    telemetry: *const uint8_t,
    telemetry_len: size_t,
    output: *mut uint8_t, // Buffer must be 96 bytes
) -> i32 {
    if weight_hash.is_null() || merkle_link.is_null() || telemetry.is_null() || output.is_null() {
        return -1;
    }

    let wh = unsafe { std::slice::from_raw_parts(weight_hash, 32) };
    let ml = unsafe { std::slice::from_raw_parts(merkle_link, 32) };
    let tel = unsafe { std::slice::from_raw_parts(telemetry, telemetry_len) };

    let mut wh_arr = [0u8; 32];
    let mut ml_arr = [0u8; 32];
    wh_arr.copy_from_slice(wh);
    ml_arr.copy_from_slice(ml);

    let point = construct_por_message(wh_arr, ml_arr, tel);
    let bytes = point.to_bytes();
    
    let out_slice = unsafe { std::slice::from_raw_parts_mut(output, 96) };
    out_slice.copy_from_slice(bytes.as_ref());

    0 // Success
}

#[unsafe(no_mangle)]
pub extern "C" fn rco_verify_hardware_sovereignty() -> i32 {
    let tee = detect_tee();
    match tee {
        TEEType::SGX | TEEType::SEV => 1, // Sovereign
        TEEType::Emulator => 0,          // Simulated
    }
}
