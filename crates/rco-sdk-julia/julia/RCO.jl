# RCO.jl — Julia SDK for the RCO Protocol
#
# Wraps the Rust FFI shared library (librco_sdk_julia.so) via ccall.
#
# Usage:
#   include("RCO.jl")
#   using .RCO
#   projected = RCO.project_p14(3.14159)
#   recovered = RCO.unproject_p14(projected)

module RCO

export project_p14, unproject_p14, project_p14_batch
export bencode_encode_integer, bencode_encode_string
export Chain, extend!, head_hash, keccak256
export IngestionPipeline, ingest!, next_index

# ── Library Path ──────────────────────────────────────────────────────
const LIBRCO = joinpath(@__DIR__, "..", "..", "..", "target", "release", "librco_sdk_julia")

# ── Status Code Checking ─────────────────────────────────────────────
struct RcoError <: Exception
    status::Int32
    msg::String
end

function check_status(status::Int32, context::String)
    status == 0 && return
    msgs = Dict(
        1 => "RewardNaN", 2 => "RewardOverflow",
        10 => "EncodeFailed", 11 => "DecodeFailed", 12 => "DuplicateKey",
        20 => "LinkageGap", 21 => "HashMismatch",
        100 => "NullPointer", 101 => "BufferTooSmall", 102 => "InvalidArgument",
    )
    throw(RcoError(status, "$context: $(get(msgs, Int(status), \"Unknown error $status\"))"))
end

# ── P14 Projection ────────────────────────────────────────────────────

"""
    project_p14(reward::Float64) -> Int128

Projects a floating-point reward into P14 integer space.
Guarantees bit-identical results across all architectures.
"""
function project_p14(reward::Float64)::Int128
    out = Ref{Int128}(0)
    status = ccall((:rco_p14_project, LIBRCO), Int32,
                   (Float64, Ptr{Int128}), reward, out)
    check_status(status, "project_p14")
    return out[]
end

"""
    unproject_p14(projected::Int128) -> Float64

Recovers an approximate Float64 from a P14-projected value.
WARNING: Lossy — use only for display, never for re-projection.
"""
function unproject_p14(projected::Int128)::Float64
    return ccall((:rco_p14_unproject, LIBRCO), Float64, (Int128,), projected)
end

"""
    project_p14_batch(rewards::Vector{Float64}) -> Vector{Int128}

Projects a batch of rewards. Throws on first invalid value.
"""
function project_p14_batch(rewards::Vector{Float64})::Vector{Int128}
    n = length(rewards)
    out = Vector{Int128}(undef, n)
    err_idx = Ref{Csize_t}(0)
    status = ccall((:rco_p14_project_batch, LIBRCO), Int32,
                   (Ptr{Float64}, Csize_t, Ptr{Int128}, Ptr{Csize_t}),
                   rewards, n, out, err_idx)
    if status != 0
        throw(RcoError(status, "project_p14_batch: error at index $(err_idx[]+1)"))
    end
    return out
end

# ── Bencode Encoding ──────────────────────────────────────────────────

"""
    bencode_encode_integer(value::Int128) -> Vector{UInt8}

Encodes an integer to canonical Bencode format.
"""
function bencode_encode_integer(value::Int128)::Vector{UInt8}
    buf_size = ccall((:rco_bencode_integer_size, LIBRCO), Csize_t, (Int128,), value)
    buf = Vector{UInt8}(undef, buf_size + 8)  # small margin
    written = Ref{Csize_t}(0)
    status = ccall((:rco_bencode_encode_integer, LIBRCO), Int32,
                   (Int128, Ptr{UInt8}, Csize_t, Ptr{Csize_t}),
                   value, buf, length(buf), written)
    check_status(status, "bencode_encode_integer")
    return buf[1:written[]]
end

"""
    bencode_encode_string(data::Vector{UInt8}) -> Vector{UInt8}

Encodes a byte string to canonical Bencode format.
"""
function bencode_encode_string(data::Vector{UInt8})::Vector{UInt8}
    buf = Vector{UInt8}(undef, length(data) + 32)
    written = Ref{Csize_t}(0)
    status = ccall((:rco_bencode_encode_string, LIBRCO), Int32,
                   (Ptr{UInt8}, Csize_t, Ptr{UInt8}, Csize_t, Ptr{Csize_t}),
                   data, length(data), buf, length(buf), written)
    check_status(status, "bencode_encode_string")
    return buf[1:written[]]
end

# ── RML Chain ─────────────────────────────────────────────────────────

"""
    Chain(genesis_data::Vector{UInt8})

Creates a new RML chain from genesis block data.
"""
mutable struct Chain
    handle::Ptr{Nothing}

    function Chain(genesis_data::Vector{UInt8})
        handle = Ref{Ptr{Nothing}}(C_NULL)
        status = ccall((:rco_chain_create, LIBRCO), Int32,
                       (Ptr{UInt8}, Csize_t, Ptr{Ptr{Nothing}}),
                       genesis_data, length(genesis_data), handle)
        check_status(status, "Chain()")
        chain = new(handle[])
        finalizer(chain) do c
            ccall((:rco_chain_destroy, LIBRCO), Cvoid, (Ptr{Nothing},), c.handle)
        end
        return chain
    end
end

"""
    extend!(chain::Chain, batch_index::UInt64, batch_data::Vector{UInt8}) -> Vector{UInt8}

Extends the chain with a new batch. Returns the 32-byte anchor hash.
"""
function extend!(chain::Chain, batch_index::UInt64, batch_data::Vector{UInt8})::Vector{UInt8}
    anchor = Vector{UInt8}(undef, 32)
    status = ccall((:rco_chain_extend, LIBRCO), Int32,
                   (Ptr{Nothing}, UInt64, Ptr{UInt8}, Csize_t, Ptr{UInt8}),
                   chain.handle, batch_index, batch_data, length(batch_data), anchor)
    check_status(status, "extend!")
    return anchor
end

"""
    head_hash(chain::Chain) -> Vector{UInt8}

Returns the current head hash of the chain.
"""
function head_hash(chain::Chain)::Vector{UInt8}
    hash = Vector{UInt8}(undef, 32)
    status = ccall((:rco_chain_head_hash, LIBRCO), Int32,
                   (Ptr{Nothing}, Ptr{UInt8}), chain.handle, hash)
    check_status(status, "head_hash")
    return hash
end

"""
    keccak256(data::Vector{UInt8}) -> Vector{UInt8}

Computes Keccak-256 hash of arbitrary data.
"""
function keccak256(data::Vector{UInt8})::Vector{UInt8}
    hash = Vector{UInt8}(undef, 32)
    status = ccall((:rco_keccak256, LIBRCO), Int32,
                   (Ptr{UInt8}, Csize_t, Ptr{UInt8}),
                   data, length(data), hash)
    check_status(status, "keccak256")
    return hash
end

# ── Ingestion Pipeline ───────────────────────────────────────────────

"""
    IngestionPipeline(wal_path::String, genesis_hash::Vector{UInt8})

Opens an atomic ingestion gateway with WAL persistence.
"""
mutable struct IngestionPipeline
    handle::Ptr{Nothing}

    function IngestionPipeline(wal_path::String, genesis_hash::Vector{UInt8})
        handle = Ref{Ptr{Nothing}}(C_NULL)
        status = ccall((:rco_ingestion_open, LIBRCO), Int32,
                       (Cstring, Ptr{UInt8}, Ptr{Ptr{Nothing}}),
                       wal_path, genesis_hash, handle)
        check_status(status, "IngestionPipeline()")
        pipeline = new(handle[])
        finalizer(pipeline) do p
            ccall((:rco_ingestion_destroy, LIBRCO), Cvoid, (Ptr{Nothing},), p.handle)
        end
        return pipeline
    end
end

"""
    ingest!(pipeline::IngestionPipeline, batch_index::UInt64, batch_data::Vector{UInt8}) -> Vector{UInt8}

Ingests a batch through the atomic 2PC pipeline. Returns the 32-byte anchor.
"""
function ingest!(pipeline::IngestionPipeline, batch_index::UInt64, batch_data::Vector{UInt8})::Vector{UInt8}
    anchor = Vector{UInt8}(undef, 32)
    status = ccall((:rco_ingestion_ingest, LIBRCO), Int32,
                   (Ptr{Nothing}, UInt64, Ptr{UInt8}, Csize_t, Ptr{UInt8}),
                   pipeline.handle, batch_index, batch_data, length(batch_data), anchor)
    check_status(status, "ingest!")
    return anchor
end

"""
    next_index(pipeline::IngestionPipeline) -> UInt64

Returns the next expected batch index.
"""
function next_index(pipeline::IngestionPipeline)::UInt64
    return ccall((:rco_ingestion_next_index, LIBRCO), UInt64, (Ptr{Nothing},), pipeline.handle)
end

end # module RCO
