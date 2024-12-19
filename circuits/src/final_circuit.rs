use borsh::{BorshDeserialize, BorshSerialize};

use crate::{header_chain::BlockHeaderCircuitOutput, spv::SPV};


#[derive(Eq, PartialEq, Clone, Debug, BorshDeserialize, BorshSerialize)]

pub struct FinalCircuitInput {
    pub block_header_circuit_output: BlockHeaderCircuitOutput,
    pub spv: SPV,
}