#pragma once

#include "../../dist/librco.h"
#include <stdexcept>
#include <string>

/**
 * RCO C++ Binding (v5.0)
 * Header-only wrapper for easy integration.
 */

namespace rco {

    class Engine {
    public:
        static __int128 project_p14(double reward) {
            int64_t low = 0;
            int64_t high = 0;
            int32_t status = rco_p14_project(reward, &low, &high);
            
            if (status != 0) {
                throw std::runtime_error("RCO Error: " + std::to_string(status));
            }
            
            __int128 result = high;
            result = (result << 64) | (unsigned __int128)(uint64_t)low;
            return result;
        }
    };

} // namespace rco
