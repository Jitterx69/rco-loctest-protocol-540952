//! # RCO-Threshold
//!
//! Distributed Threshold Sovereignty kernel. Implements BLS12-381 SSS, 
//! Distributed Key Generation (DKG), and Proactive Secret Sharing (PSS).

#![warn(missing_docs)]

pub mod dkg;
pub mod tmpq;
pub mod pss;
pub mod sss;
