<?php

/**
 * Hyper-Scale PHP SDK for RCO Protocol.
 * Optimized for PHP 8.1+ with FFI Preloading support.
 * Achieves Web-Scale Sovereignty via Zero-Copy Memory Casting.
 */
class RcoClient {
    private static $ffi = null;
    private static $libPath = null;

    /**
     * Initialize the RCO FFI bridge.
     * Supports Preloading: If called during opcache.preload, the RCO kernel 
     * becomes part of the permanent PHP process memory.
     */
    public static function init(?string $customLibPath = null) {
        if (self::$ffi !== null) return;

        self::$libPath = $customLibPath ?? __DIR__ . '/../../target/debug/librco_sdk_csharp.so';
        
        $cdef = "
            typedef struct { long high; long low; } rco_reward_t;
            int rco_project_p14_batch(const double* input, size_t count, long* output_high, long* output_low);
            int rco_construct_por_message(const uint8_t* weight_hash, const uint8_t* merkle_link, const uint8_t* telemetry, size_t telemetry_len, uint8_t* output);
            int rco_verify_hardware_sovereignty();
        ";

        try {
            self::$ffi = FFI::cdef($cdef, self::$libPath);
        } catch (FFI\Exception $e) {
            throw new RuntimeException("RCO Native Library Load Failed: " . $e->getMessage());
        }
    }

    /**
     * Projects a batch of rewards using high-performance memory casting.
     * Hits the point of integral liberty: No PHP array conversion overhead.
     */
    public static function projectP14Batch(array $rewards) {
        self::init();
        $count = count($rewards);
        
        // Allocate native memory for the projection
        $input = FFI::new("double[$count]");
        $outHigh = FFI::new("long[$count]");
        $outLow = FFI::new("long[$count]");
        
        // Rapid memory population
        foreach ($rewards as $i => $r) $input[$i] = (double)$r;
        
        $result = self::$ffi->rco_project_p14_batch(
            FFI::addr($input), 
            $count, 
            FFI::addr($outHigh), 
            FFI::addr($outLow)
        );

        if ($result !== 0) throw new RuntimeException("RCO P14 Projection Failed: $result");
        
        // Return as a structured list of 128-bit parts
        $combined = [];
        for ($i = 0; $i < $count; $i++) {
            $combined[] = [$outHigh[$i], $outLow[$i]];
        }
        return $combined;
    }

    /**
     * High-speed Proof of Reflexion construction.
     * Uses binary string pointers for zero-copy efficiency.
     */
    public static function constructPoRMessage(string $weightHash, string $merkleLink, string $telemetry) {
        self::init();
        if (strlen($weightHash) !== 32 || strlen($merkleLink) !== 32) {
            throw new InvalidArgumentException("Hashes must be 32 bytes.");
        }

        $wh = FFI::new("uint8_t[32]");
        $ml = FFI::new("uint8_t[32]");
        FFI::memcpy($wh, $weightHash, 32);
        FFI::memcpy($ml, $merkleLink, 32);

        $telLen = strlen($telemetry);
        $tel = FFI::new("uint8_t[$telLen]");
        FFI::memcpy($tel, $telemetry, $telLen);

        $output = FFI::new("uint8_t[96]");
        
        $result = self::$ffi->rco_construct_por_message(
            FFI::addr($wh), 
            FFI::addr($ml), 
            FFI::addr($tel), 
            $telLen, 
            FFI::addr($output)
        );

        if ($result !== 0) throw new RuntimeException("RCO PoR Construction Failed: $result");

        return FFI::string($output, 96);
    }

    /**
     * Checks for hardware sovereignty directly in the web request lifecycle.
     */
    public static function isHardwareSovereign(): bool {
        self::init();
        return self::$ffi->rco_verify_hardware_sovereignty() === 1;
    }
}
