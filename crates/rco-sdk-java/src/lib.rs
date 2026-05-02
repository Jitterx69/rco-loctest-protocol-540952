use jni::JNIEnv;
use jni::objects::{JClass, JDoubleArray, JLongArray, JByteArray, ReleaseMode};
use jni::sys::{jlongArray, jbyteArray, jboolean};
use rco_p14::projection::project_p14_batch;
use rco_quorum::por::construct_por_message;
use group::GroupEncoding;
use rco_enclave::hardware::{detect_tee, TEEType};

// --- Hardened JNI Interface (Default Package) ---

#[unsafe(no_mangle)]
pub extern "system" fn Java_RcoClient_projectP14Batch(
    mut env: JNIEnv,
    _class: JClass,
    input: JDoubleArray,
) -> jlongArray {
    if input.is_null() {
        return env.new_long_array(0).unwrap().into_raw();
    }

    let input_vec: Vec<f64> = unsafe {
        match env.get_array_elements(&input, ReleaseMode::NoCopyBack) {
            Ok(elements) => {
                let len = elements.len();
                let mut v = Vec::with_capacity(len);
                for i in 0..len {
                    v.push(elements[i]);
                }
                v
            }
            Err(_) => return env.new_long_array(0).unwrap().into_raw(),
        }
    };

    let mut output = vec![0i128; input_vec.len()];
    if let Err(_) = project_p14_batch(&input_vec, &mut output) {
        return env.new_long_array(0).unwrap().into_raw();
    }

    let result_len = (output.len() * 2) as i32;
    let result_array = match env.new_long_array(result_len) {
        Ok(arr) => arr,
        Err(_) => return env.new_long_array(0).unwrap().into_raw(),
    };
    
    let mut long_elements = Vec::with_capacity(output.len() * 2);
    for val in output {
        long_elements.push((val >> 64) as i64);
        long_elements.push((val & 0xFFFFFFFFFFFFFFFF) as i64);
    }

    if let Err(_) = env.set_long_array_region(&result_array, 0, &long_elements) {
         return env.new_long_array(0).unwrap().into_raw();
    }
    
    result_array.into_raw()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_RcoClient_constructPoRMessage(
    env: JNIEnv,
    _class: JClass,
    weight_hash: JByteArray,
    merkle_link: JByteArray,
    telemetry: JByteArray,
) -> jbyteArray {
    if weight_hash.is_null() || merkle_link.is_null() || telemetry.is_null() {
        return env.new_byte_array(0).unwrap().into_raw();
    }

    let wh_vec = env.convert_byte_array(&weight_hash).unwrap_or_default();
    let ml_vec = env.convert_byte_array(&merkle_link).unwrap_or_default();
    let tel_vec = env.convert_byte_array(&telemetry).unwrap_or_default();

    if wh_vec.len() != 32 || ml_vec.len() != 32 {
        return env.new_byte_array(0).unwrap().into_raw();
    }

    let mut wh = [0u8; 32];
    let mut ml = [0u8; 32];
    wh.copy_from_slice(&wh_vec);
    ml.copy_from_slice(&ml_vec);

    let point = construct_por_message(wh, ml, &tel_vec);
    let bytes = point.to_bytes();

    let i8_bytes: Vec<i8> = bytes.as_ref().iter().map(|&b| b as i8).collect();
    let result = match env.new_byte_array(i8_bytes.len() as i32) {
        Ok(arr) => arr,
        Err(_) => return env.new_byte_array(0).unwrap().into_raw(),
    };

    if let Err(_) = env.set_byte_array_region(&result, 0, &i8_bytes) {
        return env.new_byte_array(0).unwrap().into_raw();
    }

    result.into_raw()
}

// --- Project Panama (FFM) & C-Linkage Exports ---
// These allow Java 22+ to call the RCO kernel with ZERO overhead.

#[unsafe(no_mangle)]
pub extern "C" fn rco_panama_project_p14(
    input_ptr: *const f64,
    len: usize,
    output_ptr: *mut i64, // Output is 2x long per entry
) -> i32 {
    if input_ptr.is_null() || output_ptr.is_null() { return -1; }
    
    let input = unsafe { std::slice::from_raw_parts(input_ptr, len) };
    let mut results = vec![0i128; len];
    
    if let Err(_) = project_p14_batch(input, &mut results) { return -2; }
    
    let output = unsafe { std::slice::from_raw_parts_mut(output_ptr, len * 2) };
    for i in 0..len {
        output[i * 2] = (results[i] >> 64) as i64;
        output[i * 2 + 1] = (results[i] & 0xFFFFFFFFFFFFFFFF) as i64;
    }
    
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_RcoClient_verifyHardwareSovereignty(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    let tee = detect_tee();
    match tee {
        TEEType::SGX | TEEType::SEV => 1,
        TEEType::Emulator => 0,
    }
}
