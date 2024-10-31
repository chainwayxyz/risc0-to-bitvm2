mod mock_zkvm;

#[cfg(test)]
mod tests {
    use crate::mock_zkvm::MockZkvmHost;
    use circuits::header_chain::{header_chain_circuit, HeaderChainCircuitInput};
    use circuits::ZkvmHost;

    #[test]
    fn test_header_chain_circuit() {
        let host = MockZkvmHost::new();

        let input = HeaderChainCircuitInput {
            method_id: [0; 8],
            prev_proof: circuits::header_chain::HeaderChainPrevProofType::GenesisBlock,
            block_headers: vec![],
        };
        host.write(&input);
        header_chain_circuit(&host);
        let proof = host.prove([0; 8].as_ref());
        println!("{:?}", proof);
    }
}
