use sha3::{Keccak256, Digest};

fn main() {
    let challenge = [0xCC; 32];
    let omega_rve = [0xAA; 32];
    
    let mut hasher = Keccak256::new();
    hasher.update(&challenge);
    hasher.update(&omega_rve);
    
    let result = hasher.finalize();
    println!("{:?}", result);
}
