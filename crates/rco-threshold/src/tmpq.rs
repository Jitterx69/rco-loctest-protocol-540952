//! Threshold Multi-Party Quorum (TMPQ)
//!
//! Aggregates BLS signatures for manifold anchors using threshold logic.

use bls12_381::{G1Projective, G2Projective, Scalar};
use group::Group;
use crate::sss::{Share};

/// A partial signature from a single node.
pub struct PartialSignature {
    pub node_id: u64,
    pub sig: G2Projective,
}

/// The TMPQ engine.
pub struct TMPQ {
    pub t: usize,
    pub n: usize,
}

impl TMPQ {
    /// Creates a new TMPQ engine.
    pub fn new(t: usize, n: usize) -> Self {
        Self { t, n }
    }

    /// Generates a partial signature for a message hash.
    pub fn sign_partial(&self, share: &Share, message_hash: G2Projective) -> PartialSignature {
        PartialSignature {
            node_id: share.id,
            sig: message_hash * share.value,
        }
    }

    /// Aggregates $t$ partial signatures into a global signature.
    pub fn aggregate(&self, partials: &[PartialSignature]) -> Option<G2Projective> {
        if partials.len() < self.t {
            return None;
        }

        let mut aggregate_sig = G2Projective::identity();

        for (i, p_i) in partials.iter().enumerate() {
            let x_i = Scalar::from(p_i.node_id);
            let mut num = Scalar::one();
            let mut den = Scalar::one();

            for (j, p_j) in partials.iter().enumerate() {
                if i == j {
                    continue;
                }
                let x_j = Scalar::from(p_j.node_id);
                num *= x_j;
                den *= x_j - x_i;
            }

            let lambda_i = num * den.invert().unwrap();
            aggregate_sig += p_i.sig * lambda_i;
        }

        Some(aggregate_sig)
    }

    /// Verifies the aggregate signature against the master public key.
    pub fn verify(&self, aggregate_sig: G2Projective, public_key: G1Projective, message_hash: G2Projective) -> bool {
        // Pairing-based verification: e(G1, Sig) == e(PK, H)
        // In bls12_381 crate, we use pairings.
        use bls12_381::{pairing, G1Affine, G2Affine};
        
        let g1 = G1Affine::generator();
        let pk = G1Affine::from(public_key);
        let sig = G2Affine::from(aggregate_sig);
        let h = G2Affine::from(message_hash);

        pairing(&g1, &sig) == pairing(&pk, &h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use ff::Field;
    use crate::sss::generate_shares;

    #[test]
    fn test_threshold_signing() {
        let mut rng = thread_rng();
        let secret = Scalar::random(&mut rng);
        let public_key = G1Projective::generator() * secret;
        let message_hash = G2Projective::random(&mut rng);
        
        let t = 3;
        let n = 5;
        let tmpq = TMPQ::new(t, n);
        
        let shares = generate_shares(secret, t, n, &mut rng);
        let partials: Vec<PartialSignature> = shares[0..t]
            .iter()
            .map(|s| tmpq.sign_partial(s, message_hash))
            .collect();
            
        let aggregate_sig = tmpq.aggregate(&partials).unwrap();
        
        assert!(tmpq.verify(aggregate_sig, public_key, message_hash));
    }
}
