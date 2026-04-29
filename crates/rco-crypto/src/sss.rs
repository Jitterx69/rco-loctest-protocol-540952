//! Shamir's Secret Sharing (SSS)
//!
//! Provides secret splitting and Lagrange interpolation for `bls12_381::Scalar`.

use bls12_381::Scalar;
use ff::Field;
use rand_core::RngCore;

/// Evaluates a polynomial at `x`. The polynomial is defined by its coefficients.
/// `coeffs[0]` is the constant term.
pub fn evaluate_polynomial(coeffs: &[Scalar], x: &Scalar) -> Scalar {
    let mut result = Scalar::zero();
    let mut x_pow = Scalar::one();
    for c in coeffs {
        result += c * &x_pow;
        x_pow *= x;
    }
    result
}

/// Splits a secret into `n` shares, requiring `t` shares to reconstruct.
/// Returns a vector of `(x, y)` pairs, where `x` is the participant ID (1 to n) and `y` is the share.
pub fn split_secret<R: RngCore>(secret: Scalar, t: usize, n: usize, rng: &mut R) -> Vec<(Scalar, Scalar)> {
    assert!(t > 0 && t <= n);
    
    // Create random polynomial of degree t-1
    let mut coeffs = Vec::with_capacity(t);
    coeffs.push(secret);
    for _ in 1..t {
        coeffs.push(Scalar::random(&mut *rng));
    }

    let mut shares = Vec::with_capacity(n);
    for i in 1..=n {
        let x = Scalar::from(i as u64);
        let y = evaluate_polynomial(&coeffs, &x);
        shares.push((x, y));
    }
    shares
}

/// Reconstructs the secret from at least `t` shares using Lagrange interpolation.
pub fn reconstruct_secret(shares: &[(Scalar, Scalar)]) -> Scalar {
    let mut secret = Scalar::zero();

    for (i, (x_i, y_i)) in shares.iter().enumerate() {
        let mut num = Scalar::one();
        let mut den = Scalar::one();

        for (j, (x_j, _)) in shares.iter().enumerate() {
            if i != j {
                // num *= (0 - x_j) = -x_j
                num *= -x_j;
                // den *= (x_i - x_j)
                let diff = x_i - x_j;
                den *= diff;
            }
        }

        let basis = num * den.invert().unwrap();
        secret += y_i * basis;
    }
    secret
}
use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use std::time::Instant;

    #[test]
    fn test_split_and_reconstruct() {
        let mut rng = thread_rng();
        let secret = Scalar::random(&mut rng);
        let n = 12;
        let t = 8;

        let shares = split_secret(secret, t, n, &mut rng);
        assert_eq!(shares.len(), n);

        // Take first t shares
        let subset = &shares[0..t];
        let reconstructed = reconstruct_secret(subset);
        assert_eq!(secret, reconstructed);

        // Take last t shares
        let subset2 = &shares[n-t..n];
        let reconstructed2 = reconstruct_secret(subset2);
        assert_eq!(secret, reconstructed2);
    }

    #[test]
    fn bench_srrs_reconstruction() {
        // SRRS Benchmark (M-401): <15ms reconstruction latency for t=8, n=12
        let mut rng = thread_rng();
        let secret = Scalar::random(&mut rng);
        let n = 12;
        let t = 8;

        let shares = split_secret(secret, t, n, &mut rng);
        let subset = &shares[0..t];

        let start = Instant::now();
        let reconstructed = reconstruct_secret(subset);
        let elapsed = start.elapsed();

        assert_eq!(secret, reconstructed);
        println!("SRRS Reconstruction latency: {:?}", elapsed);
        assert!(elapsed.as_millis() < 15, "Reconstruction took too long!");
    }
}
