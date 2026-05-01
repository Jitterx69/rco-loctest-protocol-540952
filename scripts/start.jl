#!/usr/bin/env julia

# RCO Bootstrapper
# Purpose: Starts the RCO node.

using Pkg
# Ensure dependencies are available
if !haskey(Pkg.project().dependencies, "JSON")
    println("Installing dependencies...")
    Pkg.add("JSON")
end

include(joinpath(@__DIR__, "..", "crates", "rco-sdk-julia", "julia", "RCO.jl"))
using .RCO
using JSON

function start_node(manifest_path::String)
    println("--- RCO Starting ---")
    
    if !isfile(manifest_path)
        error("Manifest not found: $manifest_path")
    end
    manifest = JSON.parsefile(manifest_path)
    println("Version: $(manifest["version"])")
    
    # Mock data for initial start
    payload = Vector{UInt8}(undef, 96)
    id = Vector{UInt8}(undef, 32)
    fill!(payload, 0xAF); fill!(id, 0x42)
    
    println("Initializing...")
    try
        root, sig = reassemble_identity(payload, id, UInt32(3))
        println("Identity active: $(bytes2hex(root[1:8]))...")
    catch e
        println("Init failed.")
        rethrow(e)
    end

    println("--- RCO is now running ---")
end

manifest_file = joinpath(@__DIR__, "..", "dist", "manifest.json")
start_node(manifest_file)
