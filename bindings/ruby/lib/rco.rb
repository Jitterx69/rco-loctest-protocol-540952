require 'ffi'

# RCO Ruby SDK (Ψ-V5.1.0)
# Sovereign Inversion Technical Reference

module Rco
  extend FFI::Library
  
  # Load the Sovereign Core library from the dist hierarchy
  LIB_PATH = File.expand_path('../../../dist/lib/librco_core.so', __dir__)
  if File.exist?(LIB_PATH)
    ffi_lib LIB_PATH
  else
    warn "WARNING: RCO Sovereign Core not found at #{LIB_PATH}"
  end

  # Define the Ψ-V5.1.0 ABI signatures
  attach_function :rco_manifold_init, [:uint64, :pointer], :int
  attach_function :rco_manifold_project, [:pointer, :pointer, :pointer, :pointer, :size_t], :int
  attach_function :rco_manifold_audit, [:pointer, :pointer], :int
  attach_function :rco_verify_zk_proof, [:pointer, :size_t, :pointer], :bool
  attach_function :rco_manifold_destroy, [:pointer], :void

  class Manifold
    def initialize(node_id = 1)
      @handle_ptr = FFI::MemoryPointer.new(:pointer)
      status = Rco.rco_manifold_init(node_id, @handle_ptr)
      raise "RCO Manifold Initialization Failed: #{status}" if status != 0
      @handle = @handle_ptr.read_pointer
      
      # RAII: Ensure native resource disposal
      ObjectSpace.define_finalizer(self, self.class.finalize(@handle))
    end

    def self.finalize(handle)
      proc { Rco.rco_manifold_destroy(handle) if handle }
    end

    def project(rewards)
      count = rewards.size
      input = FFI::MemoryPointer.new(:double, count)
      input.put_array_of_double(0, rewards.map(&:to_f))
      
      high_res = FFI::MemoryPointer.new(:int64, count)
      low_res = FFI::MemoryPointer.new(:int64, count)
      
      status = Rco.rco_manifold_project(@handle, input, high_res, low_res, count)
      raise "RCO Projection Failed: #{status}" if status != 0
      
      highs = high_res.get_array_of_int64(0, count)
      lows = low_res.get_array_of_int64(0, count)
      
      # Reassemble into 128-bit coordinates (Ruby handles large ints natively)
      highs.zip(lows).map { |h, l| (h << 64) | (l & 0xFFFFFFFFFFFFFFFF) }
    end

    def audit
      out_ptr = FFI::MemoryPointer.new(:uint8, 32)
      status = Rco.rco_manifold_audit(@handle, out_ptr)
      raise "RCO Audit Failed: #{status}" if status != 0
      out_ptr.read_bytes(32)
    end

    def self.verify_zk_proof(proof, public_inputs)
      p_ptr = FFI::MemoryPointer.new(:uint8, proof.bytesize)
      p_ptr.put_bytes(0, proof)
      
      pi_ptr = FFI::MemoryPointer.new(:uint8, public_inputs.bytesize)
      pi_ptr.put_bytes(0, public_inputs)
      
      Rco.rco_verify_zk_proof(p_ptr, proof.bytesize, pi_ptr)
    end
  end
end
