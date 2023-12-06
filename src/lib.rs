pub type Data = Vec<u8>;
pub type Hash = Vec<u8>;

pub struct MerkleTree {
    nodes: Vec<Hash>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HashDirection {
    Left,
    Right,
}

#[derive(Debug, Default)]
pub struct Proof<'a> {
    hashes: Vec<(HashDirection, &'a Hash)>,
}

impl MerkleTree {
    /// Gets root hash for this tree
    pub fn root(&self) -> Hash {
        self.nodes[0].clone()
    }

    /// Constructs a Merkle tree from given input data
    pub fn construct(input: &[Data]) -> MerkleTree {
        let leaves: Vec<Hash> = input.iter().map(|v| utils::hash_data(v)).collect();

        Self::build_tree_from_leaves(leaves.as_slice())
    }

    /// Verifies that the given input data produces the given root hash
    pub fn verify(input: &[Data], root_hash: &Hash) -> bool {
        let constructed_tree = Self::construct(input);
        let constructed_root_hash = constructed_tree.root();

        constructed_root_hash == *root_hash
    }

    /// Verifies that the given data and proof_path correctly produce the given root_hash
    pub fn verify_proof(data: &Data, proof: &Proof, root_hash: &Hash) -> bool {
        let reconstructed_hash = proof.hashes.iter().fold(
            utils::hash_data(data),
            |current_hash, (direction, sibling_hash)| match direction {
                HashDirection::Left => utils::hash_concat(&current_hash, sibling_hash),
                HashDirection::Right => utils::hash_concat(sibling_hash, &current_hash),
            },
        );

        reconstructed_hash == *root_hash
    }

    /// Returns a list of hashes that can be used to prove that the given data is in this tree
    pub fn prove(&self, data: &Data) -> Option<Proof> {
        let mut proof_hashes = Vec::new();

        let mut current_index = self
            .nodes
            .iter()
            .position(|hash| *hash == utils::hash_data(data))?;

        while current_index > 0 {
            let (sibling_index, direction) = if current_index % 2 == 0 {
                (current_index - 1, HashDirection::Right)
            } else {
                (current_index + 1, HashDirection::Left)
            };

            proof_hashes.push((direction, &self.nodes[sibling_index]));

            if current_index == 1 {
                break;
            }

            current_index = (current_index - 1) / 2;
        }

        if proof_hashes.is_empty() {
            return None;
        }

        Some(Proof {
            hashes: proof_hashes,
        })
    }

    ///////////////////////////////
    /// Helpers for Exercise 1a ///
    ///////////////////////////////

    // Builds the Merkle tree from leaves
    fn build_tree_from_leaves(leaves: &[Hash]) -> Self {
        let count_leaves = leaves.len();
        let count_internal_nodes = utils::next_power_of_2(count_leaves) - 1;
        let mut nodes = vec![Vec::new(); count_internal_nodes + count_leaves];

        // Copy leaves
        nodes[count_internal_nodes..].clone_from_slice(leaves);

        // Build internal nodes
        Self::build_internal_nodes(&mut nodes, count_internal_nodes);

        MerkleTree { nodes }
    }

    // Internal node builder helper
    fn build_internal_nodes(nodes: &mut Vec<Hash>, count_internal_nodes: usize) {
        // Init..
        let mut parent_nodes = Self::construct_upper_level(&nodes[count_internal_nodes..]);
        if count_internal_nodes < parent_nodes.len() {
            return;
        }
        let mut upper_level_start = count_internal_nodes - parent_nodes.len();
        let mut upper_level_end = upper_level_start + parent_nodes.len();
        nodes[upper_level_start..upper_level_end].clone_from_slice(&parent_nodes);

        while parent_nodes.len() > 1 {
            parent_nodes = Self::construct_upper_level(parent_nodes.as_slice());
            upper_level_start -= parent_nodes.len();
            upper_level_end = upper_level_start + parent_nodes.len();
            nodes[upper_level_start..upper_level_end].clone_from_slice(&parent_nodes);
        }

        nodes[0] = parent_nodes.remove(0);
    }

    // Constructs nodes at a certain level
    fn construct_upper_level(nodes: &[Hash]) -> Vec<Hash> {
        let mut count = 0_usize;
        let mut level = Vec::with_capacity((nodes.len() + 1) / 2);

        while count + 1 < nodes.len() {
            level.push(Self::hash_internal_node(
                &nodes[count],
                Some(&nodes[count + 1]),
            ));
            count += 2;
        }

        if count < nodes.len() {
            level.push(nodes[count].clone());
        }

        level
    }

    fn hash_internal_node(left: &Hash, right: Option<&Hash>) -> Hash {
        if let Some(right) = right {
            utils::hash_concat(left, right)
        } else {
            utils::hash_data(left)
        }
    }
}

mod utils {
    use crate::{Data, Hash};
    use sha2::Digest;

    pub(crate) fn hash_data(data: &Data) -> Hash {
        sha2::Sha256::digest(data).to_vec()
    }

    pub(crate) fn hash_concat(h1: &Hash, h2: &Hash) -> Hash {
        let h3 = h1.iter().chain(h2).copied().collect();
        hash_data(&h3)
    }

    ///////////////
    /// Helpers ///
    ///////////////

    pub(crate) fn next_power_of_2(input: usize) -> usize {
        let mut val = input;

        val -= 1;
        val |= val >> 1;
        val |= val >> 2;
        val |= val >> 4;
        val |= val >> 8;
        val |= val >> 16;
        val += 1;

        val
    }
}

mod tests {
    use super::*;

    #[allow(dead_code)]
    fn example_data(n: usize) -> Vec<Data> {
        let mut data = vec![];
        for i in 0..n {
            data.push(vec![i as u8]);
        }
        data
    }

    #[test]
    fn test_constructions() {
        //////////////////////////////
        // Data set 1 (Power of 2) ///
        //////////////////////////////

        let data = example_data(4);
        let tree = MerkleTree::construct(&data);
        let expected_root = "9675e04b4ba9dc81b06e81731e2d21caa2c95557a85dcfa3fff70c9ff0f30b2e";
        assert_eq!(hex::encode(tree.root()), expected_root);

        ///////////////////////////////
        // Data set 2 (!Power of 2) ///
        ///////////////////////////////

        let data = example_data(3);
        let tree = MerkleTree::construct(&data);
        let expected_root = "773a93ac37ea78b3f14ac31872c83886b0a0f1fec562c4e848e023c889c2ce9f";
        assert_eq!(hex::encode(tree.root()), expected_root);

        //////////////////////////////
        // Data set 3 (Power of 2) ///
        //////////////////////////////

        let data = example_data(8);
        let tree = MerkleTree::construct(&data);
        let expected_root = "0727b310f87099c1ba2ec0ba408def82c308237c8577f0bdfd2643e9cc6b7578";
        assert_eq!(hex::encode(tree.root()), expected_root);
    }

    #[test]
    fn test_verify_root_hash_success() {
        //////////////////////////////
        // Data set 1 (Power of 2) ///
        //////////////////////////////

        let data = example_data(4);
        let tree = MerkleTree::construct(&data);
        assert!(MerkleTree::verify(&data, &tree.root()));

        ///////////////////////////////
        // Data set 2 (!Power of 2) ///
        ///////////////////////////////

        let data = example_data(3);
        let tree = MerkleTree::construct(&data);
        assert!(MerkleTree::verify(&data, &tree.root()));

        //////////////////////////////
        // Data set 3 (Power of 2) ///
        //////////////////////////////

        let data = example_data(8);
        let tree = MerkleTree::construct(&data);
        assert!(MerkleTree::verify(&data, &tree.root()));

        ///////////////////////////////
        // Data set 4 (!Power of 2) ///
        ///////////////////////////////

        let data = example_data(1);
        let tree = MerkleTree::construct(&data);
        assert!(MerkleTree::verify(&data, &tree.root()));
    }

    #[test]
    fn test_verify_proof_success() {
        //////////////////////////////
        // Data set 1 (Power of 2) ///
        //////////////////////////////

        let data = example_data(4);
        let tree = MerkleTree::construct(&data);

        // Test proof generation and verification, index 2
        let proof = tree.prove(&data[2]).unwrap();
        assert!(MerkleTree::verify_proof(&data[2], &proof, &tree.root()));

        // Test proof generation and verification, index 3
        let proof = tree.prove(&data[3]).unwrap();
        assert!(MerkleTree::verify_proof(&data[3], &proof, &tree.root()));

        ///////////////////////////////
        // Data set 2 (!Power of 2) ///
        ///////////////////////////////

        let data = example_data(3);
        let tree = MerkleTree::construct(&data);

        // Test proof generation and verification, index 1
        let proof = tree.prove(&data[1]).unwrap();
        assert!(MerkleTree::verify_proof(&data[1], &proof, &tree.root()));

        // Test proof generation and verification, index 2
        let proof = tree.prove(&data[2]).unwrap();
        assert!(MerkleTree::verify_proof(&data[2], &proof, &tree.root()));

        //////////////////////////////
        // Data set 3 (Power of 2) ///
        //////////////////////////////

        let data = example_data(8);
        let tree = MerkleTree::construct(&data);

        // Test proof generation and verification, index 4
        let proof = tree.prove(&data[4]).unwrap();
        assert!(MerkleTree::verify_proof(&data[4], &proof, &tree.root()));

        // Test proof generation and verification, index 7
        let proof = tree.prove(&data[7]).unwrap();
        assert!(MerkleTree::verify_proof(&data[7], &proof, &tree.root()));
    }

    #[test]
    fn test_verify_proof_failure() {
        ///////////////////////////////
        // Data set 1 (!Power of 2) ///
        ///////////////////////////////

        let data = example_data(3);
        let tree = MerkleTree::construct(&data);

        // Test proof generation index 1
        let proof = tree.prove(&data[1]).unwrap();
        // Test verification failure for proof generated for index 1 using data at index 2
        assert_eq!(
            MerkleTree::verify_proof(&data[2], &proof, &tree.root()),
            false
        );

        //////////////////////////////
        // Data set 2 (Power of 2) ///
        //////////////////////////////

        let data = example_data(4);
        let tree = MerkleTree::construct(&data);

        // Test proof generation index 2
        let proof = tree.prove(&data[2]).unwrap();
        // Test verification failure for proof generated for index 2 using data at index 3
        assert_eq!(
            MerkleTree::verify_proof(&data[3], &proof, &tree.root()),
            false
        );
    }
}
