// tests/integration_tests.rs

use endgame::crypto::field::FIELD_PRIME;
use endgame::{Accumulator, FieldElement, ReedSolomonAccumulator};

#[test]
fn test_field_arithmetic() {
    let a = FieldElement::new(123456789);
    let b = FieldElement::new(987654321);

    // Test addition
    let c = a + b;
    assert_eq!(
        (c.value() + FIELD_PRIME - a.value() - b.value()) % FIELD_PRIME,
        0
    );

    // Test multiplication and division
    let d = a * b;
    let e = d / b;
    assert_eq!(e, a);

    // Test powers
    let f = a.pow(3);
    let expected = a * a * a;
    assert_eq!(f, expected);

    // Test inverse
    let g = a.inverse().unwrap();
    assert_eq!(a * g, FieldElement::one());
}

#[test]
fn test_rs_accumulator() {
    let mut acc = ReedSolomonAccumulator::new();

    // Create some test state
    let state1: Vec<FieldElement> = (0..10).map(|i| FieldElement::new(i as u64)).collect();

    // Generate and verify proof
    let proof1 = acc.accumulate(state1.clone());
    assert!(acc.verify(&proof1));

    // Create another accumulator and fold
    let mut acc2 = ReedSolomonAccumulator::new();
    let state2: Vec<FieldElement> = (10..20).map(|i| FieldElement::new(i as u64)).collect();

    let proof2 = acc2.accumulate(state2);
    assert!(acc2.verify(&proof2));

    // Test folding
    let folded_proof = acc.fold(&acc2);
    assert!(acc.verify(&folded_proof));
}

#[test]
fn test_field_properties() {
    // Test associativity
    let a = FieldElement::random();
    let b = FieldElement::random();
    let c = FieldElement::random();

    assert_eq!((a + b) + c, a + (b + c));
    assert_eq!((a * b) * c, a * (b * c));

    // Test distributivity
    assert_eq!(a * (b + c), (a * b) + (a * c));

    // Test identity elements
    assert_eq!(a + FieldElement::zero(), a);
    assert_eq!(a * FieldElement::one(), a);

    // Test inverse properties
    if a.value() != 0 {
        let a_inv = a.inverse().unwrap();
        assert_eq!(a * a_inv, FieldElement::one());
    }
}

#[test]
fn test_density_consensus() {
    use endgame::consensus::density::SLOT_DURATION;
    use endgame::{
        accumulator::reed_solomon::{RSProof, ReedSolomonAccumulator},
        consensus::{
            density::{Block, DensityConsensus},
            Consensus,
        },
        FieldElement,
    }; // Import the constant

    let consensus = DensityConsensus::new();

    // Create two competing chains
    let mut chain_a = Vec::new();
    let mut chain_b = Vec::new();

    // Helper function to create blocks
    let create_block = |parent_hash: [u8; 32], height: u64, timestamp: u64| -> Block {
        let mut acc = ReedSolomonAccumulator::new();
        let state = vec![FieldElement::new(height)];
        let proof = acc.accumulate(state);

        Block {
            parent_hash,
            height,
            timestamp,
            state_proof: proof,
            accumulator: acc,
        }
    };

    // Create chain A with regular intervals
    for i in 0..10 {
        chain_a.push(create_block([0; 32], i as u64, i as u64 * SLOT_DURATION));
    }

    // Create chain B with some gaps
    for i in 0..8 {
        chain_b.push(create_block(
            [1; 32],
            i as u64,
            i as u64 * SLOT_DURATION * 2,
        ));
    }

    // Test fork choice rule
    let chosen_chain = consensus.choose_fork(&chain_a, &chain_b);
    assert_eq!(chosen_chain.len(), chain_a.len()); // Should choose chain_a due to higher density

    // Test block validation
    let state = vec![FieldElement::new(42)];
    let valid_block = create_block([0; 32], 0, consensus.current_slot() - 1);
    assert!(consensus.validate_block(&valid_block, &state));

    // Test future block validation (should fail)
    let future_block = create_block([0; 32], 0, consensus.current_slot() + 100);
    assert!(!consensus.validate_block(&future_block, &state));
}
