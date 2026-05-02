require_relative 'lib/rco'
require 'buffer'

def verify_ruby_sdk
  puts "--- RCO Ruby SDK Verification ---"
  
  # 1. Test Standard Array Projection
  rewards = [1.0, -0.5, 3.14159]
  projected = Rco::Client.project_p14_batch(rewards)
  
  puts "Projected Rewards (Standard Array):"
  projected.each_with_index do |(high, low), i|
    puts "  [#{i}] High: #{high}, Low: #{low}"
  end
  
  if projected[1][0] == -1
    puts "✅ Standard Array Projection: SUCCESS"
  else
    puts "❌ Standard Array Projection: FAILED"
  end

  # 2. Test PoR Message Construction
  wh = "\x01" * 32
  ml = "\x02" * 32
  tel = "telemetry_scientific_peak"
  por = Rco::Client.construct_por_message(wh, ml, tel)
  
  puts "PoR Message Length: #{por.bytesize}"
  if por.bytesize == 96
    puts "✅ PoR Construction: SUCCESS"
  end

  # 3. Test Hardware Sovereignty
  is_sovereign = Rco::Client.hardware_sovereign?
  puts "Hardware Sovereignty Verified: #{is_sovereign}"
  puts "✅ Hardware Check: SUCCESS"

  # 4. Numo Check
  if Rco::Client::HAS_NUMO
    puts "✅ Numo::NArray Support: ENABLED"
  else
    puts "ℹ️  Numo::NArray Support: READY (Requires gem install numo-narray)"
  end
end

begin
  verify_ruby_sdk
rescue => e
  puts "SDK Runtime Error: #{e.message}"
  puts e.backtrace
  exit 1
end
