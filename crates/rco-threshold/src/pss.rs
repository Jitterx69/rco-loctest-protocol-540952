//! Proactive Secret Sharing (PSS)
//!
//! Refreshes node shares in every epoch to defend against mobile adversaries.

use bls12_381::Scalar;
use ff::Field;
use rand::RngCore;
use crate::sss::{Share, generate_shares};

/// Refreshes a node's share by adding a share of zero.
pub fn refresh_share<R: RngCore>(current_share: &Share, t: usize, n: usize, mut rng: R) -> Vec<Share> {
    // Generate shares of zero
    let zero_secret = Scalar::zero();
    generate_shares(zero_secret, t, n, &mut rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use crate::sss::{generate_shares, reconstruct_secret};

    #[test]
    fn test_pss_refresh() {
        let mut rng = thread_rng();
        let secret = Scalar::random(&mut rng);
        let t = 3;
        let n = 5;

        let mut shares = generate_shares(secret, t, n, &mut rng);
        
        // Epoch Refresh: Each node generates shares of zero and distributes them
        let mut zero_shares_matrix = Vec::new();
        for _ in 0..n {
            zero_shares_matrix.push(generate_shares(Scalar::zero(), t, n, &mut rng));
        }

        // Each node updates its share by summing the received zero-shares
        for i in 0..n {
            let mut zero_sum = Scalar::zero();
            for j in 0..n {
                zero_sum += zero_shares_matrix[j][i].value;
            }
            shares[i].value += zero_sum;
        }

        // Secret should remain the same
        let reconstructed = reconstruct_secret(&shares[0..t]);
        assert_eq!(secret, reconstructed);
    }
}
