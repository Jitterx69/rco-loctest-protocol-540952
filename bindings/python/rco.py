import ctypes
import os

# RCO Python Binding (v5.0)
# No external dependencies required (uses standard ctypes)

class RCO:
    def __init__(self, lib_path=None):
        if lib_path is None:
            # Default to relative path in the rco-v5 bundle
            lib_path = os.path.join(os.path.dirname(__file__), "../../dist/librco.so")
        
        if not os.path.exists(lib_path):
            raise FileNotFoundError(f"RCO Engine (librco.so) not found at: {lib_path}")
            
        self.lib = ctypes.CDLL(lib_path)
        
        # Define Argument and Result types
        self.lib.rco_p14_project.argtypes = [ctypes.c_double, ctypes.POINTER(ctypes.c_int64), ctypes.POINTER(ctypes.c_int64)]
        self.lib.rco_p14_project.restype = ctypes.c_int32

    def project_p14(self, reward: float) -> int:
        low = ctypes.c_int64(0)
        high = ctypes.c_int64(0)
        status = self.lib.rco_p14_project(ctypes.c_double(reward), ctypes.byref(low), ctypes.byref(high))
        if status != 0:
            raise Exception(f"RCO Error: {status}")
        # Combine low and high into a single python int
        return (high.value << 64) | (low.value & 0xFFFFFFFFFFFFFFFF)

# Singleton Instance
_instance = None
def get_rco():
    global _instance
    if _instance is None:
        _instance = RCO()
    return _instance

def project_p14(reward: float):
    return get_rco().project_p14(reward)
