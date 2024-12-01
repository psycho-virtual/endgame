use super::Accumulator;
use crate::crypto::{field::FieldElement, merkle::MerkleTree};
use std::fmt::Write;

const EVAL_DOMAIN_SIZE: usize = 256;
const NUM_CHALLENGES: usize = 2;

// Helper for debug hex printing
fn hex_str(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(2 * bytes.len());
    for b in bytes {
        write!(s, "{:02x}", b).unwrap();
    }
    s
}

#[derive(Clone, Debug)]
pub struct ReedSolomonAccumulator {
    evaluations: Vec<FieldElement>,
    domain: Vec<FieldElement>,
    degree: usize,
    merkle_root: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct RSProof {
    challenge_evals: Vec<FieldElement>,
    challenge_points: Vec<FieldElement>,
    domain_evals: Vec<FieldElement>,
    eval_indices: Vec<usize>,
    merkle_root: Vec<u8>,
    merkle_proofs: Vec<Vec<Vec<u8>>>,
}

impl ReedSolomonAccumulator {
    // Evaluation functions remain unchanged...
    fn evaluate_at(&self, x: FieldElement) -> FieldElement {
        if self.degree == 0 {
            return FieldElement::zero();
        }

        for i in 0..self.degree {
            if x == self.domain[i] {
                return self.evaluations[i];
            }
        }

        let mut num = FieldElement::zero();
        let mut den = FieldElement::zero();

        for i in 0..self.degree {
            let mut weight = FieldElement::one();
            for j in 0..self.degree {
                if i != j {
                    weight = weight * (x - self.domain[j]) / (self.domain[i] - self.domain[j]);
                }
            }
            num = num + weight * self.evaluations[i];
            den = den + weight;
        }

        if den.value() == 0 {
            return FieldElement::zero();
        }
        num / den
    }

    fn serialize_field_element(fe: &FieldElement) -> Vec<u8> {
        let value = fe.value();
        let mut result = vec![0u8; 8];
        result.copy_from_slice(&value.to_le_bytes());
        result
    }

    fn build_merkle_tree(&self) -> (MerkleTree, Vec<Vec<u8>>) {
        println!("\nBuilding Merkle tree:");
        let leaves: Vec<Vec<u8>> = self.evaluations[..self.degree]
            .iter()
            .map(|eval| {
                let leaf = Self::serialize_field_element(eval);
                println!("Leaf for eval {}: {}", eval.value(), hex_str(&leaf));
                leaf
            })
            .collect();

        println!("Total leaves: {}", leaves.len());

        let tree = MerkleTree::new(leaves.clone());
        println!("Tree root: {}", hex_str(&tree.root()));

        (tree, leaves)
    }

    fn verify_merkle_proof(
        &self,
        root: &[u8],
        proof: &[Vec<u8>],
        leaf: &[u8],
        index: usize,
    ) -> bool {
        println!("\nVerifying Merkle proof:");
        println!("Root: {}", hex_str(root));
        println!("Leaf: {}", hex_str(leaf));
        println!("Index: {}", index);
        println!("Proof length: {}", proof.len());

        for (i, p) in proof.iter().enumerate() {
            println!("Proof element {}: {}", i, hex_str(p));
        }

        let result = MerkleTree::verify_proof(root, leaf, proof, index);
        println!("Verification result: {}", result);
        result
    }
}

impl Accumulator for ReedSolomonAccumulator {
    type Proof = RSProof;
    type State = Vec<FieldElement>;

    fn new() -> Self {
        let domain: Vec<FieldElement> = (0..EVAL_DOMAIN_SIZE)
            .map(|i| FieldElement::from(i as u64))
            .collect();

        let evaluations = vec![FieldElement::zero(); EVAL_DOMAIN_SIZE];
        let tree = MerkleTree::new(vec![]);

        ReedSolomonAccumulator {
            evaluations,
            domain,
            degree: 0,
            merkle_root: tree.root(),
        }
    }

    fn accumulate(&mut self, state: Self::State) -> Self::Proof {
        println!("\nAccumulating state of size: {}", state.len());

        self.evaluations.clear();
        self.evaluations.extend(state.iter());
        self.degree = state.len();

        let (tree, leaves) = self.build_merkle_tree();
        self.merkle_root = tree.root();

        let eval_indices: Vec<usize> = (0..NUM_CHALLENGES).map(|i| i % self.degree).collect();

        println!("Selected indices for proofs: {:?}", eval_indices);

        let domain_evals: Vec<FieldElement> = eval_indices
            .iter()
            .map(|&idx| self.evaluations[idx])
            .collect();

        let merkle_proofs: Vec<Vec<Vec<u8>>> = eval_indices
            .iter()
            .map(|&idx| {
                let proof = tree.generate_proof(idx);
                println!("Generated proof for index {}", idx);
                proof
            })
            .collect();

        let challenge_points: Vec<FieldElement> = (0..NUM_CHALLENGES)
            .map(|_| loop {
                let point = FieldElement::random();
                if !self.domain[..self.degree].contains(&point) {
                    return point;
                }
            })
            .collect();

        let challenge_evals: Vec<FieldElement> = challenge_points
            .iter()
            .map(|&point| self.evaluate_at(point))
            .collect();

        RSProof {
            challenge_evals,
            challenge_points,
            domain_evals,
            eval_indices,
            merkle_root: self.merkle_root.clone(),
            merkle_proofs,
        }
    }

    fn verify(&self, proof: &Self::Proof) -> bool {
        println!("\nVerifying proof");
        println!("Number of merkle proofs: {}", proof.merkle_proofs.len());
        println!("Number of evaluations: {}", proof.domain_evals.len());

        // Verify Merkle proofs
        for (i, (&idx, proof_path)) in proof
            .eval_indices
            .iter()
            .zip(proof.merkle_proofs.iter())
            .enumerate()
        {
            let eval = proof.domain_evals[i];
            println!(
                "\nVerifying proof {} for eval {} at index {}",
                i,
                eval.value(),
                idx
            );

            let leaf = Self::serialize_field_element(&eval);
            if !self.verify_merkle_proof(&proof.merkle_root, proof_path, &leaf, idx) {
                return false;
            }
        }

        // Verify polynomial evaluations
        for (i, &point) in proof.challenge_points.iter().enumerate() {
            let expected = proof.challenge_evals[i];
            let computed = self.evaluate_at(point);
            if expected != computed {
                return false;
            }
        }

        true
    }

    fn fold(&mut self, other: &Self) -> Self::Proof {
        let alpha = FieldElement::random();
        let max_deg = self.degree.max(other.degree);

        println!("\nFolding two accumulators:");
        println!("First degree: {}", self.degree);
        println!("Second degree: {}", other.degree);
        println!("Max degree: {}", max_deg);

        let mut new_evals = Vec::with_capacity(max_deg);

        for i in 0..max_deg {
            let self_eval = if i < self.degree {
                self.evaluations[i]
            } else {
                FieldElement::zero()
            };

            let other_eval = if i < other.degree {
                other.evaluations[i]
            } else {
                FieldElement::zero()
            };

            new_evals.push(self_eval + alpha * other_eval);
        }

        self.evaluations = new_evals;
        self.degree = max_deg;

        self.accumulate(self.evaluations[..self.degree].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization_consistency() {
        let fe = FieldElement::new(123);
        let bytes = ReedSolomonAccumulator::serialize_field_element(&fe);
        assert_eq!(bytes.len(), 8, "Serialized field element should be 8 bytes");

        // Verify value can be recovered
        let mut value_bytes = [0u8; 8];
        value_bytes.copy_from_slice(&bytes);
        let value = u64::from_le_bytes(value_bytes);
        assert_eq!(value, fe.value());
    }

    #[test]
    fn test_merkle_tree_basic() {
        println!("\nRunning basic Merkle tree test");

        let leaves: Vec<Vec<u8>> = vec![vec![1, 0, 0, 0, 0, 0, 0, 0], vec![2, 0, 0, 0, 0, 0, 0, 0]];

        println!("Building tree with leaves:");
        for (i, leaf) in leaves.iter().enumerate() {
            println!("Leaf {}: {}", i, hex_str(leaf));
        }

        let tree = MerkleTree::new(leaves.clone());
        let root = tree.root();
        println!("Tree root: {}", hex_str(&root));

        let proof = tree.generate_proof(0);
        let verified = MerkleTree::verify_proof(&root, &leaves[0], &proof, 0);
        assert!(verified, "Basic Merkle proof verification failed");
    }

    #[test]
    fn test_merkle_proof_verification() {
        let mut acc = ReedSolomonAccumulator::new();
        let state = vec![FieldElement::new(1), FieldElement::new(2)];
        let proof = acc.accumulate(state);
        assert!(acc.verify(&proof), "Basic Merkle proof verification failed");
    }

    #[test]
    fn test_accumulator_basic() {
        let mut acc = ReedSolomonAccumulator::new();
        let state = vec![
            FieldElement::new(1),
            FieldElement::new(2),
            FieldElement::new(3),
        ];
        let proof = acc.accumulate(state);
        assert!(acc.verify(&proof), "Basic test failed");
    }

    #[test]
    fn test_accumulator_fold() {
        let mut acc1 = ReedSolomonAccumulator::new();
        let mut acc2 = ReedSolomonAccumulator::new();

        let state1 = vec![FieldElement::new(1), FieldElement::new(2)];
        let state2 = vec![FieldElement::new(3), FieldElement::new(4)];

        let proof1 = acc1.accumulate(state1);
        assert!(
            acc1.verify(&proof1),
            "First accumulator verification failed"
        );

        let proof2 = acc2.accumulate(state2);
        assert!(
            acc2.verify(&proof2),
            "Second accumulator verification failed"
        );

        let folded_proof = acc1.fold(&acc2);
        assert!(acc1.verify(&folded_proof), "Folded verification failed");
    }

    #[test]
    fn test_accumulator_large_state() {
        let mut acc = ReedSolomonAccumulator::new();
        let state: Vec<FieldElement> = (0..5).map(|i| FieldElement::new(i as u64)).collect();
        let proof = acc.accumulate(state);
        assert!(acc.verify(&proof), "Large state verification failed");
    }
}
