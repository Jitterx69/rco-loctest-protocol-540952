#!/usr/bin/env julia

using Pkg
# Pkg.activate(joinpath(@__DIR__, "..", "crates", "rco-sdk-julia"))
include(joinpath(@__DIR__, "..", "crates", "rco-sdk-julia", "julia", "RCO.jl"))
using .RCO
using Test

function test_sovereign_migration_cycle()
    println("Testing Sovereign Migration Cycle (Julia -> Rust -> Julia)...")
    
    # 1. Setup Original Root
    orig_root_hash = rand(UInt8, 32)
    orig_signature = rand(UInt8, 64)
    threshold = UInt32(5)
    
    # 2. Export Identity
    println("Exporting Identity...")
    payload, id_id = export_identity(orig_root_hash, orig_signature, threshold)
    
    @test length(payload) == 96
    @test length(id_id) == 32
    @test payload != [orig_root_hash; orig_signature] # Should be encrypted/shuffled
    
    # 3. Reassemble Identity
    println("Reassembling Identity...")
    res_root_hash, res_signature = reassemble_identity(payload, id_id, threshold)
    
    @test res_root_hash == orig_root_hash
    @test res_signature == orig_signature
    
    println("Migration Cycle Verified: Identity Integrity Maintained.")
end

test_sovereign_migration_cycle()
