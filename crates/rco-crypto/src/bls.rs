//! BLS12-381 Threshold Signatures
//!
//! Provides non-interactive signature aggregation.

use bls12_381::{G1Projective, G2Projective, Scalar};
use group::{Curve, Group};
use ff::Field;
use rand_core::RngCore;

/// A private key share for BLS signatures.
#[derive(Clone, Debug)]
pub struct PrivateShare(pub Scalar);

/// A public key share for BLS signatures.
#[derive(Clone, Debug)]
pub struct PublicShare(pub G1Projective);

/// A BLS signature.
#[derive(Clone, Debug)]
pub struct Signature(pub G2Projective);

impl PrivateShare {
    /// Generates a random private share.
    pub fn random<R: RngCore>(rng: &mut R) -> Self {
        Self(Scalar::random(rng))
    }

    /// Computes the corresponding public share.
    pub fn public_share(&self) -> PublicShare {
        PublicShare(G1Projective::generator() * self.0)
    }

    /// Signs a message hash.
    /// The hash must be mapped to the G2 curve.
    pub fn sign(&self, message_hash_point: &G2Projective) -> Signature {
        Signature(*message_hash_point * self.0)
    }
}

/// Aggregates multiple signatures into a single signature.
pub fn aggregate_signatures(signatures: &[Signature]) -> Signature {
    let mut agg = G2Projective::identity();
    for sig in signatures {
        agg += sig.0;
    }
    Signature(agg)
}

/// Aggregates multiple public shares into a single aggregate public key.
pub fn aggregate_public_shares(shares: &[PublicShare]) -> PublicShare {
    let mut agg = G1Projective::identity();
    for share in shares {
        agg += share.0;
    }
    PublicShare(agg)
}
