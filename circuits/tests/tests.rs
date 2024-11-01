mod mock_zkvm;

#[cfg(test)]
mod tests {
    use crate::mock_zkvm::MockZkvmHost;
    use borsh::BorshDeserialize;
    use circuits::header_chain::{
        header_chain_circuit, BlockHeader, BlockHeaderCircuitOutput, HeaderChainCircuitInput,
    };
    use circuits::ZkvmHost;

    #[test]
    fn test_header_chain_circuit() {
        // Download the headers.bin file from https://zerosync.org/chaindata/headers.bin
        let headers = include_bytes!("../../headers.bin");
        let headers = headers
            .chunks(80)
            .map(|header| BlockHeader::try_from_slice(header).unwrap())
            .collect::<Vec<BlockHeader>>();

        let host = MockZkvmHost::new();

        let input = HeaderChainCircuitInput {
            method_id: [0; 8],
            prev_proof: circuits::header_chain::HeaderChainPrevProofType::GenesisBlock,
            block_headers: headers[..4000].to_vec(),
        };
        host.write(&input);
        header_chain_circuit(&host);
        let proof = host.prove([0; 8].as_ref());

        let output = BlockHeaderCircuitOutput::try_from_slice(&proof.journal).unwrap();
        let new_host = MockZkvmHost::new();

        let newinput = HeaderChainCircuitInput {
            method_id: [0; 8],
            prev_proof: circuits::header_chain::HeaderChainPrevProofType::PrevProof(output),
            block_headers: headers[4000..8000].to_vec(),
        };
        new_host.write(&newinput);
        new_host.add_assumption(proof);

        header_chain_circuit(&new_host);

        let new_proof = new_host.prove([0; 8].as_ref());

        let new_output = BlockHeaderCircuitOutput::try_from_slice(&new_proof.journal).unwrap();

        assert_eq!(
            hex::encode(new_output.chain_state.total_work),
            "00000000000000000000000000000000000000000000000000001f401f401f40"
        );
    }
}
