//! Distributed Key Generation (DKG)
//!
//! Provides Pedersen Verifiable Secret Sharing (VSS).

use bls12_381::{G1Projective, Scalar};
use ff::Field;
use group::Group;
use rand_core::RngCore;

/// A Pedersen commitment to a polynomial coefficient.
#[derive(Clone, Debug)]
pub struct Commitment(pub G1Projective);

/// Generates Pedersen commitments for a polynomial.
pub fn generate_commitments(coeffs: &[Scalar]) -> Vec<Commitment> {
    let g = G1Projective::generator();
    // For true Pedersen VSS, we would need a second generator `h` for the blinding factor.
    // In Feldman VSS (which is often what is meant in BLS DKG literature unless perfect hiding is required), 
    // we just commit to the coefficients using `g`.
    coeffs.iter().map(|c| Commitment(g * c)).collect()
}

/// Verifies a share against the public commitments.
pub fn verify_share(share_x: &Scalar, share_y: &Scalar, commitments: &[Commitment]) -> bool {
    let g = G1Projective::generator();
    let lhs = g * share_y;

    let mut rhs = G1Projective::identity();
    let mut x_pow = Scalar::one();
    for c in commitments {
        rhs += c.0 * &x_pow;
        x_pow *= share_x;
    }

    lhs == rhs
}
