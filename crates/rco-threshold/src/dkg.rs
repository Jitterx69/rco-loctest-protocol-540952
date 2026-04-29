//! Joint-Feldman Distributed Key Generation (JF-DKG)
//!
//! Nodes collectively generate a shared public key and private shares.

use bls12_381::{G1Projective, Scalar};
use group::Group;
use ff::Field;
use rand::RngCore;
use crate::sss::{Share, generate_shares};

/// State of a node during DKG.
pub struct DKGNode {
    pub id: u64,
    pub t: usize,
    pub n: usize,
    /// The secret contribution from this node
    pub secret_contribution: Scalar,
    /// Polynomial coefficients
    pub coeffs: Vec<Scalar>,
    /// Commitments to the polynomial coefficients
    pub commitments: Vec<G1Projective>,
}

impl DKGNode {
    /// Creates a new node for DKG.
    pub fn new<R: RngCore>(id: u64, t: usize, n: usize, mut rng: R) -> Self {
        let secret = Scalar::random(&mut rng);
        let mut coeffs = vec![secret];
        for _ in 1..t {
            coeffs.push(Scalar::random(&mut rng));
        }

        let commitments = coeffs.iter().map(|c| G1Projective::generator() * c).collect();
        let secret = coeffs[0];

        Self {
            id,
            t,
            n,
            secret_contribution: secret,
            coeffs,
            commitments,
        }
    }

    /// Generates shares of this node's secret for all other nodes using its stored coefficients.
    pub fn generate_my_shares(&self) -> Vec<Share> {
        (1..=self.n as u64)
            .map(|id| {
                let x = Scalar::from(id);
                let mut value = Scalar::zero();
                let mut x_pow = Scalar::one();
                for coeff in &self.coeffs {
                    value += coeff * x_pow;
                    x_pow *= x;
                }
                Share { id, value }
            })
            .collect()
    }

    /// Verifies a received share against the sender's commitments.
    pub fn verify_received_share(&self, share: &Share, commitments: &[G1Projective]) -> bool {
        let x = Scalar::from(share.id);
        let mut lhs = G1Projective::generator() * share.value;
        let mut rhs = G1Projective::identity();
        let mut x_pow = Scalar::one();

        for c in commitments {
            rhs += c * x_pow;
            x_pow *= x;
        }

        lhs == rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_dkg_share_verification() {
        let mut rng = thread_rng();
        let node1 = DKGNode::new(1, 3, 5, &mut rng);
        let node2 = DKGNode::new(2, 3, 5, &mut rng);

        let shares_from_1 = node1.generate_my_shares();
        let share_for_2 = &shares_from_1[1]; // share with id 2

        assert!(node2.verify_received_share(share_for_2, &node1.commitments));
    }
}
