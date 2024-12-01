// src/accumulator/mod.rs

pub mod reed_solomon;

use crate::crypto::field::FieldElement;

pub trait Accumulator {
    type Proof;
    type State;

    fn new() -> Self;
    fn accumulate(&mut self, state: Self::State) -> Self::Proof;
    fn verify(&self, proof: &Self::Proof) -> bool;
    fn fold(&mut self, other: &Self) -> Self::Proof;
}
