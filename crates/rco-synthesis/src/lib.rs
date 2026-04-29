//! # RCO-Synthesis
//!
//! Stage-I Final Synthesis kernel. Unifies all protocol layers into a single 
//! telemetry-to-finality pipeline and enforces the Global Lineage Invariant (GLI).

#![warn(missing_docs)]

pub mod pipeline;
pub mod gli;
pub mod audit;
