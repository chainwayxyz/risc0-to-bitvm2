use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::utils::{calculate_sha256, hash_pair};

/// Represents the MMR for outside zkVM (native)
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, BorshDeserialize, BorshSerialize)]
pub struct MMRNative {
    nodes: Vec<Vec<[u8; 32]>>,
}

impl MMRNative {
    /// Creates a new MMR for native usage
    pub fn new() -> Self {
        MMRNative {
            nodes: vec![vec![]],
        }
    }

    /// Appends a new leaf to the MMR
    pub fn append(&mut self, leaf: [u8; 32]) {
        self.nodes[0].push(leaf);
        self.recalculate_peaks();
    }

    /// Recalculates peaks based on the current leaves
    fn recalculate_peaks(&mut self) {
        let depth = self.nodes.len();
        for level in 0..depth - 1 {
            if self.nodes[level].len() % 2 == 1 {
                break;
            } else {
                let node = hash_pair(
                    self.nodes[level][self.nodes[level].len() - 2],
                    self.nodes[level][self.nodes[level].len() - 1],
                );
                self.nodes[level + 1].push(node);
            }
        }
        if self.nodes[depth - 1].len() > 1 {
            let node = hash_pair(self.nodes[depth - 1][0], self.nodes[depth - 1][1]);
            self.nodes.push(vec![node]);
        }
    }

    fn get_subroots(&self) -> Vec<[u8; 32]> {
        let mut subroots: Vec<[u8; 32]> = vec![];
        for level in &self.nodes {
            if level.len() % 2 == 1 {
                subroots.push(level[level.len() - 1]);
            }
        }
        subroots.reverse();
        subroots
    }

    pub fn get_root(&self) -> [u8; 32] {
        let mut preimage: Vec<u8> = vec![];
        let subroots = self.get_subroots();
        for i in 0..subroots.len() {
            preimage.extend_from_slice(&subroots[i]);
        }
        calculate_sha256(&preimage)
    }

    fn get_subroot_helpers(&self, subroot: [u8; 32]) -> Vec<[u8; 32]> {
        let mut subroots: Vec<[u8; 32]> = vec![];
        for level in &self.nodes {
            if level.len() % 2 == 1 {
                if level[level.len() - 1] != subroot {
                    subroots.push(level[level.len() - 1]);
                }
            }
        }
        subroots
    }

    pub fn generate_proof(&self, index: u32) -> ([u8; 32], Vec<[u8; 32]>, u32) {
        if self.nodes[0].len() == 0 {
            panic!("MMR is empty");
        }
        if self.nodes[0].len() <= index as usize {
            panic!("Index out of bounds");
        }
        let mut proof: Vec<[u8; 32]> = vec![];
        let mut current_index = index;
        let mut current_level = 0;
        while !(current_index == self.nodes[current_level].len() as u32 - 1
            && self.nodes[current_level].len() % 2 == 1)
        {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };
            proof.push(self.nodes[current_level][sibling_index as usize]);
            current_index = current_index / 2;
            current_level += 1;
        }
        let subroot = self.nodes[current_level][current_index as usize];
        proof.extend(self.get_subroot_helpers(subroot));
        (self.nodes[0][index as usize], proof, index)
    }

    fn get_helpers_from_index(&self, index: u32) -> (usize, usize, u32) {
        let xor = (self.nodes[0].len() as u32) ^ index;
        let xor_leading_digit = 31 - xor.leading_zeros() as usize;
        let internal_idx = index & ((1 << xor_leading_digit) - 1);
        let leading_zeros_size = 31 - (self.nodes[0].len() as u32).leading_zeros() as usize;
        let mut tree_idx = 0;
        for i in xor_leading_digit + 1..=leading_zeros_size {
            if self.nodes[0].len() & (1 << i) != 0 {
                tree_idx += 1;
            }
        }
        (tree_idx, xor_leading_digit, internal_idx)
    }

    pub fn verify_proof(&self, leaf: [u8; 32], proof: &Vec<[u8; 32]>, index: u32) -> bool {
        let (subroot_idx, subtree_size, internal_idx) = self.get_helpers_from_index(index);
        let mut current_hash = leaf;
        for i in 0..subtree_size {
            let sibling = proof[i];
            if internal_idx & (1 << i) == 0 {
                current_hash = hash_pair(current_hash, sibling);
            } else {
                current_hash = hash_pair(sibling, current_hash);
            }
        }
        let subroots = self.get_subroots();
        let mut preimage: Vec<u8> = vec![];
        for i in 0..subroot_idx {
            preimage.extend_from_slice(&subroots[i]);
        }
        preimage.extend_from_slice(&current_hash);
        for i in subroot_idx + 1..subroots.len() {
            preimage.extend_from_slice(&subroots[i]);
        }
        let calculated_root = calculate_sha256(&preimage);
        calculated_root == self.get_root()
    }
}

mod tests {

    use crate::mmr_guest::MMRGuest;

    use super::*;

    #[test]
    #[should_panic(expected = "MMR is empty")]
    fn test_mmr_native_fail_0() {
        let mmr = MMRNative::new();
        let root = mmr.get_root();
        let (leaf, proof, index) = mmr.generate_proof(0);
        assert_eq!(mmr.get_root(), root);
        assert_eq!(mmr.verify_proof(leaf, &proof, index), true);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_mmr_native_fail_1() {
        let mut mmr = MMRNative::new();
        mmr.append([0; 32]);
        let root = mmr.get_root();
        let (leaf, proof, index) = mmr.generate_proof(1);
        assert_eq!(mmr.get_root(), root);
        assert_eq!(mmr.verify_proof(leaf, &proof, index), true);
    }

    #[test]
    fn test_mmr_native() {
        let mut mmr = MMRNative::new();
        let mut leaves = vec![];

        for i in 0..42 {
            let leaf = [i as u8; 32];
            leaves.push(leaf);

            mmr.append(leaf);

            for j in 0..=i {
                let (leaf, proof, index) = mmr.generate_proof(j);
                assert!(mmr.verify_proof(leaf, &proof, index));
            }
        }
    }

    #[test]
    fn test_mmr_crosscheck() {
        let mut mmr_native = MMRNative::new();
        let mut mmr_guest = MMRGuest::new();
        let mut leaves = vec![];

        for i in 0..42 {
            let leaf = [i as u8; 32];
            leaves.push(leaf);

            mmr_native.append(leaf);
            mmr_guest.append(leaf);

            let root_native = mmr_native.get_root();
            let root_guest = mmr_guest.get_root();
            assert_eq!(
                root_native, root_guest,
                "Roots do not match after adding leaf {}",
                i
            );

            for j in 0..=i {
                let (leaf, proof, index) = mmr_native.generate_proof(j);
                assert!(
                    mmr_native.verify_proof(leaf, &proof, index),
                    "Failed to verify proof for leaf {} in native MMR",
                    j
                );
                assert!(
                    mmr_guest.verify_proof(leaf, &proof, index),
                    "Failed to verify proof for leaf {} in guest MMR",
                    j
                );
            }
        }
    }
}
