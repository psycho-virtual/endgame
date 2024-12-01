// src/consensus/density.rs

use super::Consensus;
use crate::accumulator::{reed_solomon::ReedSolomonAccumulator, Accumulator};
use crate::crypto::field::FieldElement;
use std::time::{SystemTime, UNIX_EPOCH};

pub const SLOT_DURATION: u64 = 1; // 1 second per slot for demo
const WINDOW_SIZE: u64 = 50;  // Number of blocks to consider for density

#[derive(Clone)]
pub struct Block {
    pub parent_hash: [u8; 32],
    pub height: u64,
    pub timestamp: u64,
    pub state_proof: RSProof,
    pub accumulator: ReedSolomonAccumulator,
}

pub struct DensityConsensus {
    window_size: u64,
    slot_duration: u64,
}

impl DensityConsensus {
    pub fn new() -> Self {
        Self {
            window_size: WINDOW_SIZE,
            slot_duration: SLOT_DURATION,
        }
    }

    // Calculate the expected number of slots in a given time window
    fn expected_slots(&self, start_time: u64, end_time: u64) -> u64 {
        (end_time - start_time) / self.slot_duration
    }

    // Get current slot number
    fn current_slot(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / self.slot_duration
    }

    // Calculate density for a specific window
    fn window_density(&self, blocks: &[Block], start_slot: u64, end_slot: u64) -> f64 {
        let blocks_in_window: Vec<&Block> = blocks
            .iter()
            .filter(|b| {
                let block_slot = b.timestamp / self.slot_duration;
                block_slot >= start_slot && block_slot <= end_slot
            })
            .collect();

        let expected_blocks = self.expected_slots(
            start_slot * self.slot_duration,
            end_slot * self.slot_duration,
        );

        blocks_in_window.len() as f64 / expected_blocks as f64
    }
}

impl Consensus for DensityConsensus {
    type Block = Block;
    type State = Vec<FieldElement>;

    fn validate_block(&self, block: &Self::Block, state: &Self::State) -> bool {
        // Validate timestamp
        let current_slot = self.current_slot();
        let block_slot = block.timestamp / self.slot_duration;
        if block_slot > current_slot {
            return false;
        }

        // Validate state proof
        block.accumulator.verify(&block.state_proof)
    }

    fn choose_fork(&self, chain_a: &[Self::Block], chain_b: &[Self::Block]) -> &[Self::Block] {
        // For recent forks (within window_size), use simple length comparison
        if chain_a
            .last()
            .unwrap()
            .timestamp
            .abs_diff(chain_b.last().unwrap().timestamp)
            < self.window_size * self.slot_duration
        {
            return if chain_a.len() > chain_b.len() {
                chain_a
            } else {
                chain_b
            };
        }

        // For older forks, use density-based selection
        let density_a = self.calculate_density(chain_a);
        let density_b = self.calculate_density(chain_b);

        if density_a > density_b {
            chain_a
        } else {
            chain_b
        }
    }

    fn calculate_density(&self, blocks: &[Self::Block]) -> f64 {
        let num_windows = (blocks.len() as u64).max(1);
        let window_size = self.window_size;

        // Calculate average density across sliding windows
        let mut total_density = 0.0;

        for i in 0..num_windows {
            let start_block = &blocks[i as usize];
            let end_idx = ((i + window_size).min(blocks.len() as u64 - 1)) as usize;
            let end_block = &blocks[end_idx];

            let window_density = self.window_density(
                &blocks[i as usize..=end_idx],
                start_block.timestamp / self.slot_duration,
                end_block.timestamp / self.slot_duration,
            );

            total_density += window_density;
        }

        total_density / num_windows as f64
    }
}
