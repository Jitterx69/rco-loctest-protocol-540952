<?php

/**
 * Hyper-Scale PHP SDK for RCO Protocol (Ψ-V5.1.0).
 * Optimized for PHP 8.1+ with FFI Preloading support.
 * Sovereign Inversion Technical Reference.
 */
class RcoClient {
    private static $ffi = null;
    private $manifold = null;

    /**
     * Initialize the RCO FFI bridge.
     */
    public static function init(?string $libPath = null) {
        if (self::$ffi !== null) return;

        $libPath = $libPath ?? __DIR__ . '/../../dist/lib/librco_core.so';
        
        if (!file_exists($libPath)) {
            throw new RuntimeException("RCO Sovereign Core not found at: $libPath");
        }

        $cdef = "
            typedef struct rco_manifold_t rco_manifold_t;
            int rco_manifold_init(uint64_t node_id, rco_manifold_t** out_manifold);
            int rco_manifold_project(rco_manifold_t* manifold, const double* rewards, int64_t* results_high, int64_t* results_low, size_t len);
            int rco_manifold_audit(rco_manifold_t* manifold, uint8_t* out_gih);
            bool rco_verify_zk_proof(const uint8_t* proof, size_t len, const uint8_t* public_inputs);
            void rco_manifold_destroy(rco_manifold_t* manifold);
        ";

        try {
            self::$ffi = FFI::cdef($cdef, $libPath);
        } catch (FFI\Exception $e) {
            throw new RuntimeException("RCO Native Library Load Failed: " . $e->getMessage());
        }
    }

    public function __construct(int $nodeId = 1) {
        self::init();
        $handlePtr = FFI::new("rco_manifold_t*");
        $status = self::$ffi->rco_manifold_init($nodeId, FFI::addr($handlePtr));
        if ($status !== 0) {
            throw new RuntimeException("RCO Manifold Initialization Failed: $status");
        }
        $this->manifold = $handlePtr;
    }

    public function project(array $rewards) {
        $count = count($rewards);
        $input = FFI::new("double[$count]");
        $outHigh = FFI::new("int64_t[$count]");
        $outLow = FFI::new("int64_t[$count]");
        
        foreach ($rewards as $i => $r) $input[$i] = (double)$r;
        
        $status = self::$ffi->rco_manifold_project(
            $this->manifold,
            FFI::addr($input), 
            FFI::addr($outHigh), 
            FFI::addr($outLow),
            $count
        );

        if ($status !== 0) throw new RuntimeException("RCO Projection Failed: $status");
        
        $combined = [];
        for ($i = 0; $i < $count; $i++) {
            // Reassemble coordinates (Simplified for PHP 64-bit)
            $combined[] = ["high" => $outHigh[$i], "low" => $outLow[$i]];
        }
        return $combined;
    }

    public function audit(): string {
        $gih = FFI::new("uint8_t[32]");
        $status = self::$ffi->rco_manifold_audit($this->manifold, FFI::addr($gih));
        if ($status !== 0) throw new RuntimeException("RCO Audit Failed: $status");
        return FFI::string($gih, 32);
    }

    public static function verifyZkProof(string $proof, string $publicInputs): bool {
        self::init();
        $proofLen = strlen($proof);
        $p = FFI::new("uint8_t[$proofLen]");
        FFI::memcpy($p, $proof, $proofLen);
        
        $piLen = strlen($publicInputs);
        $pi = FFI::new("uint8_t[$piLen]");
        FFI::memcpy($pi, $publicInputs, $piLen);

        return (bool)self::$ffi->rco_verify_zk_proof($p, $proofLen, $pi);
    }

    public function __destruct() {
        if ($this->manifold) {
            self::$ffi->rco_manifold_destroy($this->manifold);
        }
    }
}
