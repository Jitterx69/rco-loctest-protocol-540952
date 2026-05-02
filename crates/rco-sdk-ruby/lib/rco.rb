require 'ffi'

module Rco
  extend FFI::Library
  
  # Load the native library (using the C# FFI build which provides the C-ABI)
  ffi_lib File.expand_path('../../target/debug/librco_sdk_csharp.so', __dir__)

  # Define the C-FFI signatures
  attach_function :rco_project_p14_batch, [:pointer, :size_t, :pointer, :pointer], :int
  attach_function :rco_construct_por_message, [:pointer, :pointer, :pointer, :size_t, :pointer], :int
  attach_function :rco_verify_hardware_sovereignty, [], :int

  class Client
    # Integration with Numo::NArray if available
    HAS_NUMO = begin
      require 'numo/narray'
      true
    rescue LoadError
      false
    end

    def self.project_p14_batch(rewards)
      if HAS_NUMO && rewards.is_a?(Numo::NArray)
        project_numo(rewards)
      else
        project_standard(rewards)
      end
    end

    private

    def self.project_numo(rewards)
      count = rewards.size
      
      # Ensure data is contiguous DFloat (double)
      dfloat_rewards = rewards.is_a?(Numo::DFloat) ? rewards : Numo::DFloat.cast(rewards)
      
      # Create Numo output arrays for zero-copy high/low parts
      out_high = Numo::Int64.new(count)
      out_low = Numo::Int64.new(count)
      
      # Extract raw pointers from Numo
      input_ptr = FFI::Pointer.new(:double, dfloat_rewards.get_data_addr)
      high_ptr = FFI::Pointer.new(:int64, out_high.get_data_addr)
      low_ptr = FFI::Pointer.new(:int64, out_low.get_data_addr)
      
      result = Rco.rco_project_p14_batch(input_ptr, count, high_ptr, low_ptr)
      raise "P14 Projection Failed: #{result}" if result != 0
      
      { high: out_high, low: out_low }
    end

    def self.project_standard(rewards)
      count = rewards.size
      input = FFI::MemoryPointer.new(:double, count)
      input.put_array_of_double(0, rewards)
      
      out_high = FFI::MemoryPointer.new(:int64, count)
      out_low = FFI::MemoryPointer.new(:int64, count)
      
      result = Rco.rco_project_p14_batch(input, count, out_high, out_low)
      raise "P14 Projection Failed: #{result}" if result != 0
      
      highs = out_high.get_array_of_int64(0, count)
      lows = out_low.get_array_of_int64(0, count)
      
      highs.zip(lows)
    end

    public

    def self.construct_por_message(weight_hash, merkle_link, telemetry)
      raise "Hashes must be 32 bytes" if weight_hash.bytesize != 32 || merkle_link.bytesize != 32
      
      wh_ptr = FFI::MemoryPointer.new(:uint8, 32)
      wh_ptr.put_bytes(0, weight_hash)
      
      ml_ptr = FFI::MemoryPointer.new(:uint8, 32)
      ml_ptr.put_bytes(0, merkle_link)
      
      tel_ptr = FFI::MemoryPointer.new(:uint8, telemetry.bytesize)
      tel_ptr.put_bytes(0, telemetry)
      
      out_ptr = FFI::MemoryPointer.new(:uint8, 96)
      
      result = Rco.rco_construct_por_message(wh_ptr, ml_ptr, tel_ptr, telemetry.bytesize, out_ptr)
      raise "PoR Construction Failed: #{result}" if result != 0
      
      out_ptr.get_bytes(0, 96)
    end

    def self.hardware_sovereign?
      Rco.rco_verify_hardware_sovereignty == 1
    end
  end
end
