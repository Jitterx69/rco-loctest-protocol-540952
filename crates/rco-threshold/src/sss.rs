//! Shamir's Secret Sharing (SSS) over BLS12-381
//!
//! Provides polynomial interpolation for distributed key management.

use bls12_381::Scalar;
use ff::Field;
use rand::RngCore;

/// Represents a secret share.
#[derive(Clone, Debug)]
pub struct Share {
    /// X-coordinate (node index)
    pub id: u64,
    /// Y-coordinate (share value)
    pub value: Scalar,
}

/// Generates $n$ shares for a secret with threshold $t$.
pub fn generate_shares<R: RngCore>(secret: Scalar, t: usize, n: usize, mut rng: R) -> Vec<Share> {
    let mut coeffs = vec![secret];
    for _ in 1..t {
        coeffs.push(Scalar::random(&mut rng));
    }

    (1..=n as u64)
        .map(|id| {
            let x = Scalar::from(id);
            let mut value = Scalar::zero();
            let mut x_pow = Scalar::one();
            for coeff in &coeffs {
                value += coeff * x_pow;
                x_pow *= x;
            }
            Share { id, value }
        })
        .collect()
}

/// Reconstructs the secret from $t$ shares.
pub fn reconstruct_secret(shares: &[Share]) -> Scalar {
    let mut secret = Scalar::zero();

    for (i, share_i) in shares.iter().enumerate() {
        let x_i = Scalar::from(share_i.id);
        let mut num = Scalar::one();
        let mut den = Scalar::one();

        for (j, share_j) in shares.iter().enumerate() {
            if i == j {
                continue;
            }
            let x_j = Scalar::from(share_j.id);
            num *= x_j;
            den *= x_j - x_i;
        }

        let lambda_i = num * den.invert().unwrap();
        secret += share_i.value * lambda_i;
    }

    secret
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_sss_reconstruction() {
        let mut rng = thread_rng();
        let secret = Scalar::random(&mut rng);
        let t = 3;
        let n = 5;

        let shares = generate_shares(secret, t, n, &mut rng);
        
        // Reconstruction with exactly t shares
        let reconstructed = reconstruct_secret(&shares[0..t]);
        assert_eq!(secret, reconstructed);

        // Reconstruction with different t shares
        let reconstructed2 = reconstruct_secret(&shares[2..5]);
        assert_eq!(secret, reconstructed2);
    }
}
