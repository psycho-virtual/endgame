// src/lib.rs

pub mod accumulator;
pub mod consensus;
pub mod crypto;

// Re-export commonly used items
pub use accumulator::{reed_solomon::ReedSolomonAccumulator, Accumulator};
pub use consensus::{
    density::{Block, DensityConsensus},
    Consensus,
};
pub use crypto::field::FieldElement;
