use napi_derive::napi;
use napi::bindgen_prelude::*;
use rco_p14::projection::project_p14_batch;
use rco_quorum::por::construct_por_message;
use group::GroupEncoding;
use rco_enclave::hardware::{detect_tee, TEEType};

/// Projects a batch of rewards into the P14 lattice using BigInt64Array.
/// This is the hyper-scale path: Zero-copy mapping to JS BigInt.
#[napi]
pub fn project_p14_batch_bigint(rewards: Float64Array, mut output_high: BigInt64Array, mut output_low: BigInt64Array) -> Result<()> {
    let len = rewards.len();
    if output_high.len() != len || output_low.len() != len {
        return Err(Error::new(Status::InvalidArg, "Buffer lengths must match".to_string()));
    }

    let mut results = vec![0i128; len];
    if let Err(_) = project_p14_batch(rewards.as_ref(), &mut results) {
        return Err(Error::new(Status::GenericFailure, "P14 Projection failed".to_string()));
    }

    for i in 0..len {
        output_high[i] = (results[i] >> 64) as i64;
        output_low[i] = (results[i] & 0xFFFFFFFFFFFFFFFF) as i64;
    }

    Ok(())
}

/// Asynchronous projection: Offloads the heavy P14 kernel to the libuv thread pool.
/// Essential for maintaining 60FPS/low-latency in JS applications.
#[napi(ts_return_type="Promise<void>")]
pub fn project_p14_batch_async(rewards: Float64Array, output_high: BigInt64Array, output_low: BigInt64Array) -> AsyncTask<P14Task> {
    AsyncTask::new(P14Task {
        rewards: rewards.to_vec(),
        len: rewards.len(),
        output_high,
        output_low,
    })
}

pub struct P14Task {
    rewards: Vec<f64>,
    len: usize,
    output_high: BigInt64Array,
    output_low: BigInt64Array,
}

#[napi]
impl Task for P14Task {
    type Output = Vec<i128>;
    type JsValue = ();

    fn compute(&mut self) -> Result<Self::Output> {
        let mut results = vec![0i128; self.len];
        project_p14_batch(&self.rewards, &mut results)
            .map_err(|_| Error::new(Status::GenericFailure, "Async P14 Projection failed"))?;
        Ok(results)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        for i in 0..self.len {
            self.output_high[i] = (output[i] >> 64) as i64;
            self.output_low[i] = (output[i] & 0xFFFFFFFFFFFFFFFF) as i64;
        }
        Ok(())
    }
}

#[napi]
pub fn construct_por_message_node(
    weight_hash: Uint8Array,
    merkle_link: Uint8Array,
    telemetry: Uint8Array,
) -> Uint8Array {
    if weight_hash.len() != 32 || merkle_link.len() != 32 {
        return Uint8Array::new(vec![]);
    }

    let mut wh = [0u8; 32];
    let mut ml = [0u8; 32];
    wh.copy_from_slice(weight_hash.as_ref());
    ml.copy_from_slice(merkle_link.as_ref());

    let point = construct_por_message(wh, ml, telemetry.as_ref());
    Uint8Array::new(point.to_bytes().as_ref().to_vec())
}

#[napi]
pub fn verify_hardware_sovereignty_node() -> bool {
    let tee = detect_tee();
    match tee {
        TEEType::SGX | TEEType::SEV => true,
        TEEType::Emulator => false,
    }
}
