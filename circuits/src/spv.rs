use crate::{
    header_chain::CircuitBlockHeader, merkle_tree::BlockInclusionProof, mmr_guest::MMRGuest,
    mmr_native::MMRInclusionProof, transaction::CircuitTransaction,
};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Eq, PartialEq, Clone, Debug, BorshDeserialize, BorshSerialize)]
pub struct SPV {
    pub transaction: CircuitTransaction,
    pub block_inclusion_proof: BlockInclusionProof,
    pub block_version: i32,
    pub block_prev_block_hash: [u8; 32],
    pub block_time: u32,
    pub block_bits: u32,
    pub block_nonce: u32,
    pub mmr_inclusion_proof: MMRInclusionProof,
}

impl SPV {
    pub fn new(
        transaction: CircuitTransaction,
        block_inclusion_proof: BlockInclusionProof,
        block_version: i32,
        block_prev_block_hash: [u8; 32],
        block_time: u32,
        block_bits: u32,
        block_nonce: u32,
        mmr_inclusion_proof: MMRInclusionProof,
    ) -> Self {
        SPV {
            transaction,
            block_inclusion_proof,
            block_version,
            block_prev_block_hash,
            block_time,
            block_bits,
            block_nonce,
            mmr_inclusion_proof,
        }
    }

    pub fn verify(&self, mmr_guest: MMRGuest) -> bool {
        let txid: [u8; 32] = self.transaction.txid();
        let block_merkle_root = self.block_inclusion_proof.get_root(txid);
        let block_header = CircuitBlockHeader {
            version: self.block_version,
            prev_block_hash: self.block_prev_block_hash,
            merkle_root: block_merkle_root,
            time: self.block_time,
            bits: self.block_bits,
            nonce: self.block_nonce,
        };
        let block_hash = block_header.compute_block_hash();
        mmr_guest.verify_proof(block_hash, &self.mmr_inclusion_proof)
    }
}
