// src/crypto/merkle.rs

use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Clone)]
pub struct MerkleTree {
    nodes: Vec<Vec<u8>>,
    leaf_count: usize,
}

impl fmt::Debug for MerkleTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "MerkleTree {{")?;
        writeln!(f, "  leaf_count: {}", self.leaf_count)?;
        writeln!(f, "  nodes: [")?;
        for (i, node) in self.nodes.iter().enumerate() {
            let node_hex: String = node.iter().map(|b| format!("{:02x}", b)).collect();
            writeln!(f, "    {}: {}", i, node_hex)?;
        }
        writeln!(f, "  ]")?;
        write!(f, "}}")
    }
}

impl MerkleTree {
    pub fn new(leaves: Vec<Vec<u8>>) -> Self {
        if leaves.is_empty() {
            return Self {
                nodes: vec![vec![0u8; 32]],
                leaf_count: 0,
            };
        }

        let leaf_count = leaves.len();
        let total_nodes = 2 * leaf_count - 1;
        let mut nodes = vec![vec![0u8; 32]; total_nodes];

        // Copy leaves into the second half of the array
        for (i, leaf) in leaves.into_iter().enumerate() {
            let mut hasher = Sha256::new();
            hasher.update(&leaf);
            nodes[leaf_count - 1 + i] = hasher.finalize().to_vec();
        }

        // Build internal nodes
        for i in (0..leaf_count - 1).rev() {
            let mut hasher = Sha256::new();
            hasher.update(&nodes[2 * i + 1]); // Left child
            hasher.update(&nodes[2 * i + 2]); // Right child
            nodes[i] = hasher.finalize().to_vec();
        }

        Self { nodes, leaf_count }
    }

    pub fn root(&self) -> Vec<u8> {
        self.nodes[0].clone()
    }

    pub fn generate_proof(&self, index: usize) -> Vec<Vec<u8>> {
        if index >= self.leaf_count {
            return vec![];
        }

        let mut proof = Vec::new();
        let mut current = self.leaf_count - 1 + index;

        while current > 0 {
            // If we're a left child, get right sibling, and vice versa
            let sibling = if current % 2 == 0 {
                current - 1
            } else {
                current + 1
            };

            if sibling < self.nodes.len() {
                proof.push(self.nodes[sibling].clone());
            }

            // Move up to parent
            current = (current - 1) / 2;
        }

        proof
    }

    pub fn verify_proof(root: &[u8], leaf: &[u8], proof: &[Vec<u8>], index: usize) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(leaf);
        let mut current = hasher.finalize().to_vec();
        let mut current_index = index;

        for proof_element in proof {
            let mut hasher = Sha256::new();
            if current_index % 2 == 0 {
                hasher.update(&current);
                hasher.update(proof_element);
            } else {
                hasher.update(proof_element);
                hasher.update(&current);
            }
            current = hasher.finalize().to_vec();
            current_index /= 2;
        }

        current == root
    }

    // Helper function to visualize the tree (useful for debugging)
    pub fn print_tree(&self) {
        println!("\nMerkle Tree Structure:");
        println!("Leaf count: {}", self.leaf_count);
        println!("Total nodes: {}", self.nodes.len());

        let mut level = 0;
        let mut level_size = 1;
        let mut printed = 0;

        while printed < self.nodes.len() {
            println!("\nLevel {}:", level);
            for i in 0..level_size {
                if printed + i < self.nodes.len() {
                    let node_hex: String = self.nodes[printed + i]
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect();
                    println!("  Node {}: {}", printed + i, node_hex);
                }
            }
            printed += level_size;
            level_size *= 2;
            level += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes_to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::new(vec![]);
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.leaf_count, 0);
    }

    #[test]
    fn test_single_leaf() {
        let leaf = vec![1u8, 2u8, 3u8];
        let tree = MerkleTree::new(vec![leaf.clone()]);

        let mut hasher = Sha256::new();
        hasher.update(&leaf);
        let expected_hash = hasher.finalize().to_vec();

        assert_eq!(tree.root(), expected_hash);
        assert_eq!(tree.leaf_count, 1);
    }

    #[test]
    fn test_two_leaves() {
        let leaf1 = vec![1u8];
        let leaf2 = vec![2u8];
        let tree = MerkleTree::new(vec![leaf1.clone(), leaf2.clone()]);

        // Calculate expected hashes
        let mut hasher = Sha256::new();
        hasher.update(&leaf1);
        let hash1 = hasher.finalize().to_vec();

        let mut hasher = Sha256::new();
        hasher.update(&leaf2);
        let hash2 = hasher.finalize().to_vec();

        let mut hasher = Sha256::new();
        hasher.update(&hash1);
        hasher.update(&hash2);
        let root_hash = hasher.finalize().to_vec();

        assert_eq!(tree.root(), root_hash);

        // Verify proofs for both leaves
        let proof0 = tree.generate_proof(0);
        let proof1 = tree.generate_proof(1);

        assert!(MerkleTree::verify_proof(&root_hash, &leaf1, &proof0, 0));
        assert!(MerkleTree::verify_proof(&root_hash, &leaf2, &proof1, 1));
    }

    #[test]
    fn test_four_leaves() {
        let leaves: Vec<Vec<u8>> = (0..4).map(|i| vec![i as u8]).collect();
        let tree = MerkleTree::new(leaves.clone());

        println!("\nTree structure:");
        tree.print_tree();

        let root = tree.root();
        println!("\nRoot hash: {}", bytes_to_hex(&root));

        // Test proofs for all leaves
        for (i, leaf) in leaves.iter().enumerate() {
            println!("\nTesting proof for leaf {}:", i);
            let proof = tree.generate_proof(i);

            println!("Proof elements:");
            for (j, p) in proof.iter().enumerate() {
                println!("  {}: {}", j, bytes_to_hex(p));
            }

            assert!(
                MerkleTree::verify_proof(&root, leaf, &proof, i),
                "Proof verification failed for leaf {}",
                i
            );
        }
    }

    #[test]
    fn test_invalid_proof() {
        let leaves = vec![vec![1u8], vec![2u8]];
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.root();

        // Generate proof for leaf 0
        let proof = tree.generate_proof(0);

        // Try to verify with wrong leaf
        let wrong_leaf = vec![3u8];
        assert!(!MerkleTree::verify_proof(&root, &wrong_leaf, &proof, 0));

        // Try to verify with wrong index
        assert!(!MerkleTree::verify_proof(&root, &leaves[0], &proof, 1));

        // Try to verify with modified proof
        let mut bad_proof = proof.clone();
        if !bad_proof.is_empty() {
            bad_proof[0] = vec![0u8; 32];
        }
        assert!(!MerkleTree::verify_proof(&root, &leaves[0], &bad_proof, 0));
    }

    #[test]
    fn test_proof_consistency() {
        let leaves: Vec<Vec<u8>> = (0..8).map(|i| vec![i as u8]).collect();
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.root();

        // Generate and verify all proofs
        for (i, leaf) in leaves.iter().enumerate() {
            let proof1 = tree.generate_proof(i);
            let proof2 = tree.generate_proof(i);

            // Same index should generate identical proofs
            assert_eq!(proof1, proof2);

            // Both proofs should verify
            assert!(MerkleTree::verify_proof(&root, leaf, &proof1, i));
            assert!(MerkleTree::verify_proof(&root, leaf, &proof2, i));
        }
    }
}
