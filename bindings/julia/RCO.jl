# RCO.jl — Julia SDK for the RCO Protocol (Ψ-V5.1.0)
#
# Wraps the Sovereign Core shared library (librco_core.so) via ccall.
#
# Usage:
#   using .RCO
#   m = Manifold(1) # Init with node_id 1
#   projected = project(m, [1.0, 2.0, 3.14])

module RCO

using Libdl

export Manifold, project, audit, verify_zk_proof

# ── Library Path ──────────────────────────────────────────────────────
const LIB_NAME = "librco_core"
const LIBRCO = Libdl.find_library(LIB_NAME, [joinpath(@__DIR__, "..", "..", "dist", "lib")])
if LIBRCO == ""
    error("Sovereign Core ($LIB_NAME) not found in dist/lib. Please check the Ψ-V5.1.0 distribution.")
end

# ── Status Code Checking ─────────────────────────────────────────────
struct RcoError <: Exception
    status::Int32
end

function check_status(status::Int32, context::String)
    status == 0 && return
    throw(RcoError(status))
end

# ── Manifold Wrapper ──────────────────────────────────────────────────
mutable struct Manifold
    handle::Ptr{Nothing}

    function Manifold(node_id::UInt64)
        handle_ref = Ref{Ptr{Nothing}}(C_NULL)
        status = ccall((:rco_manifold_init, LIBRCO), Int32,
                       (UInt64, Ptr{Ptr{Nothing}}),
                       node_id, handle_ref)
        check_status(status, "Manifold Initialization")
        
        m = new(handle_ref[])
        finalizer(m) do x
            if x.handle != C_NULL
                ccall((:rco_manifold_destroy, LIBRCO), Cvoid, (Ptr{Nothing},), x.handle)
            end
        end
        return m
    end
end

# ── Simplicial Projection ──────────────────────────────────────────────
function project(m::Manifold, rewards::Vector{Float64})::Vector{Int128}
    len = length(rewards)
    high_res = Vector{Int64}(undef, len)
    low_res = Vector{Int64}(undef, len)
    
    status = ccall((:rco_manifold_project, LIBRCO), Int32,
                   (Ptr{Nothing}, Ptr{Float64}, Ptr{Int64}, Ptr{Int64}, Csize_t),
                   m.handle, rewards, high_res, low_res, len)
    check_status(status, "Manifold Projection")
    
    # Reassemble Int128 coordinates
    return [Int128(high_res[i]) << 64 | (Int128(low_res[i]) & 0xFFFFFFFFFFFFFFFF) for i in 1:len]
end

# ── Sentinel Auditing ──────────────────────────────────────────────────
function audit(m::Manifold)::Vector{UInt8}
    gih = Vector{UInt8}(undef, 32)
    status = ccall((:rco_manifold_audit, LIBRCO), Int32,
                   (Ptr{Nothing}, Ptr{UInt8}), m.handle, gih)
    check_status(status, "Manifold Audit")
    return gih
end

# ── ZK-Verification ───────────────────────────────────────────────────
function verify_zk_proof(proof::Vector{UInt8}, public_inputs::Vector{UInt8})::Bool
    return ccall((:rco_verify_zk_proof, LIBRCO), Bool,
                 (Ptr{UInt8}, Csize_t, Ptr{UInt8}),
                 proof, length(proof), public_inputs)
end

end # module RCO
