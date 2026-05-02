#pragma once

#include "../../dist/include/rco-core.h"
#include <stdexcept>
#include <string>
#include <vector>
#include <array>
#include <cstdint>

/**
 * RCO C++ Binding (Ψ-V5.1.0)
 * Header-only RAII wrapper for Sovereign Core integration.
 */

namespace rco {

    class Manifold {
    public:
        explicit Manifold(uint64_t node_id) {
            int32_t status = rco_manifold_init(node_id, &handle_);
            if (status != 0) {
                throw std::runtime_error("RCO Initialization Error: " + std::to_string(status));
            }
        }

        ~Manifold() {
            if (handle_) {
                rco_manifold_destroy(handle_);
            }
        }

        // Disable copy
        Manifold(const Manifold&) = delete;
        Manifold& operator=(const Manifold&) = delete;

        // Batch Projection
        std::vector<__int128> project(const std::vector<double>& rewards) {
            size_t len = rewards.size();
            std::vector<int64_t> high_res(len);
            std::vector<int64_t> low_res(len);
            
            int32_t status = rco_manifold_project(handle_, rewards.data(), high_res.data(), low_res.data(), len);
            if (status != 0) {
                throw std::runtime_error("RCO Projection Error: " + std::to_string(status));
            }
            
            std::vector<__int128> results(len);
            for (size_t i = 0; i < len; ++i) {
                __int128 val = high_res[i];
                results[i] = (val << 64) | (unsigned __int128)(uint64_t)low_res[i];
            }
            return results;
        }

        // Sentinel Audit (GIH)
        std::array<uint8_t, 32> audit() {
            std::array<uint8_t, 32> gih;
            int32_t status = rco_manifold_audit(handle_, gih.data());
            if (status != 0) {
                throw std::runtime_error("RCO Audit Error: " + std::to_string(status));
            }
            return gih;
        }

        // ZK Verification
        static bool verify_proof(const std::vector<uint8_t>& proof, const std::vector<uint8_t>& public_inputs) {
            return rco_verify_zk_proof(proof.data(), proof.size(), public_inputs.data());
        }

    private:
        rco_manifold_t handle_ = nullptr;
    };

} // namespace rco
