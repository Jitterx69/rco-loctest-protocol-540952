import ctypes
import os

# RCO Python Binding (Ψ-V5.1.0)
# Sovereign Inversion Technical Reference

class RCO:
    def __init__(self, lib_path=None):
        if lib_path is None:
            # Point to the new Ψ-V5.1.0 distribution path
            lib_path = os.path.join(os.path.dirname(__file__), "../../dist/lib/librco_core.so")
        
        if not os.path.exists(lib_path):
            raise FileNotFoundError(f"RCO Sovereign Core (librco_core.so) not found at: {lib_path}")
            
        self.lib = ctypes.CDLL(lib_path)
        
        # Define Argument and Result types for Ψ-V5.1.0 ABI
        self.lib.rco_manifold_init.argtypes = [ctypes.c_uint64, ctypes.POINTER(ctypes.c_void_p)]
        self.lib.rco_manifold_init.restype = ctypes.c_int32

        self.lib.rco_manifold_project.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_double), ctypes.POINTER(ctypes.c_int64), ctypes.POINTER(ctypes.c_int64), ctypes.c_size_t]
        self.lib.rco_manifold_project.restype = ctypes.c_int32

        self.lib.rco_manifold_audit.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_ubyte * 32)]
        self.lib.rco_manifold_audit.restype = ctypes.c_int32

        self.lib.rco_verify_zk_proof.argtypes = [ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t, ctypes.POINTER(ctypes.c_ubyte)]
        self.lib.rco_verify_zk_proof.restype = ctypes.c_bool

        self.lib.rco_manifold_destroy.argtypes = [ctypes.c_void_p]
        self.lib.rco_manifold_destroy.restype = None

        self.manifold = None

    def init_manifold(self, node_id: int):
        handle = ctypes.c_void_p()
        status = self.lib.rco_manifold_init(ctypes.c_uint64(node_id), ctypes.byref(handle))
        if status != 0:
            raise Exception(f"RCO Initialization Error: {status}")
        self.manifold = handle

    def project(self, rewards: list):
        if self.manifold is None:
            raise Exception("Manifold not initialized.")
        
        length = len(rewards)
        c_rewards = (ctypes.c_double * length)(*rewards)
        high_res = (ctypes.c_int64 * length)()
        low_res = (ctypes.c_int64 * length)()
        
        status = self.lib.rco_manifold_project(self.manifold, c_rewards, high_res, low_res, ctypes.c_size_t(length))
        if status != 0:
            raise Exception(f"RCO Projection Error: {status}")
            
        return [(high_res[i] << 64) | (low_res[i] & 0xFFFFFFFFFFFFFFFF) for i in range(length)]

    def audit(self) -> bytes:
        if self.manifold is None:
            raise Exception("Manifold not initialized.")
        
        gih_buffer = (ctypes.c_ubyte * 32)()
        status = self.lib.rco_manifold_audit(self.manifold, ctypes.byref(gih_buffer))
        if status != 0:
            raise Exception(f"RCO Audit Error: {status}")
            
        return bytes(gih_buffer)

    def verify_proof(self, proof: bytes, public_inputs: bytes) -> bool:
        c_proof = (ctypes.c_ubyte * len(proof)).from_buffer_copy(proof)
        c_inputs = (ctypes.c_ubyte * len(public_inputs)).from_buffer_copy(public_inputs)
        return self.lib.rco_verify_zk_proof(c_proof, len(proof), c_inputs)

    def __del__(self):
        if self.manifold:
            self.lib.rco_manifold_destroy(self.manifold)

# Singleton Instance
_instance = None
def get_rco(node_id=1):
    global _instance
    if _instance is None:
        _instance = RCO()
        _instance.init_manifold(node_id)
    return _instance
