//! Gossip Protocol
//!
//! Non-deterministic dissemination of signature shares.

use cuckoofilter::CuckooFilter;
use std::collections::hash_map::DefaultHasher;
use rco_crypto::bls::Signature;

/// A Gossip Network Packet
#[derive(Clone, Debug)]
pub struct GossipPacket {
    /// Epoch or batch index this signature corresponds to
    pub batch_index: u64,
    /// The signature share
    pub share: Signature,
    /// The ID of the node that created this share
    pub node_id: u32,
}

/// The Gossip Engine
pub struct GossipEngine {
    filter: CuckooFilter<DefaultHasher>,
    local_node_id: u32,
}

impl GossipEngine {
    /// Initializes a new Gossip Engine
    pub fn new(local_node_id: u32) -> Self {
        Self {
            filter: CuckooFilter::new(),
            local_node_id,
        }
    }

    /// Processes an incoming packet. Returns true if it's new and should be processed.
    pub fn process_incoming(&mut self, packet: &GossipPacket) -> bool {
        // Create a unique byte representation of the packet for filtering
        let mut data = Vec::new();
        data.extend_from_slice(&packet.batch_index.to_le_bytes());
        data.extend_from_slice(&packet.node_id.to_le_bytes());
        // For simplicity, we just hash the basic metadata to deduplicate shares from the same node for the same batch

        if self.filter.contains(&data) {
            return false; // Already seen
        }

        self.filter.add(&data).unwrap();
        true
    }

    /// Selects peers to gossip to based on fanout
    pub fn select_peers(&self, peers: &[u32], fanout: usize) -> Vec<u32> {
        let mut rng = rand::thread_rng();
        use rand::seq::SliceRandom;
        
        let mut available: Vec<u32> = peers.iter().filter(|&&p| p != self.local_node_id).copied().collect();
        available.shuffle(&mut rng);
        available.truncate(fanout);
        available
    }
}
