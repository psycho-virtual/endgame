// src/consensus/mod.rs

pub mod density;

use crate::crypto::field::FieldElement;

pub trait Consensus {
    type Block;
    type State;

    fn validate_block(&self, block: &Self::Block, state: &Self::State) -> bool;
    fn choose_fork<'a>(
        &self,
        chain_a: &'a [Self::Block],
        chain_b: &'a [Self::Block],
    ) -> &'a [Self::Block];
    fn calculate_density(&self, blocks: &[Self::Block]) -> f64;
}
