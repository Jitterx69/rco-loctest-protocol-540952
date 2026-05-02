/**
 * RCO-v5-Psi (Ψ-V5.1.0) Universal Core Header
 * Absolute Technical Reference for Native Manifold Coordination
 */

#ifndef RCO_CORE_H
#define RCO_CORE_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Opaque handle to an RCO Manifold Instance.
 * Sovereign identity and memory state are managed within the TEE/Enclave.
 */
typedef struct rco_manifold_t rco_manifold_t;

/**
 * Result codes for the Sovereign Inversion.
 */
typedef enum {
    RCO_SUCCESS = 0,
    RCO_ERROR_HARDWARE_TAMPERED = 1,
    RCO_ERROR_LATTICE_DIVERGENCE = 2,
    RCO_ERROR_ZK_PROOF_INVALID = 3,
    RCO_ERROR_ENCLAVE_COMPROMISED = 4
} rco_result_t;

/**
 * Initializes a Sovereign RCO Manifold bound to the local TPM 2.0 device.
 * @param node_id The physical node ID for AIK derivation.
 * @param out_manifold Pointer to the manifold handle.
 */
rco_result_t rco_manifold_init(uint64_t node_id, rco_manifold_t** out_manifold);

/**
 * Performs a high-frequency P14 Lattice Projection.
 * Utilizes AVX-512/NEON assembly for sub-microsecond coordination.
 */
rco_result_t rco_manifold_project(rco_manifold_t* manifold, const double* rewards, int64_t* results_high, int64_t* results_low, size_t len);

/**
 * Generates a Sentinel Audit Report sealed against the TPM PCRs.
 * @param out_gih Pointer to a 32-byte buffer to receive the Global Invariance Hash.
 */
rco_result_t rco_manifold_audit(rco_manifold_t* manifold, uint8_t* out_gih);

/**
 * Verifies a Zero-Knowledge Invariance Proof for a peer node.
 */
bool rco_verify_zk_proof(const uint8_t* proof, size_t len, const uint8_t* public_inputs);

/**
 * Closes and seals the manifold instance.
 */
void rco_manifold_destroy(rco_manifold_t* manifold);

#ifdef __cplusplus
}
#endif

#endif // RCO_CORE_H
