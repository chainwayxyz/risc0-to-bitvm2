#!/bin/bash

set -eoux

#TEST VERSION OF THE MAIN FLOW

# Make all the node commands actually have 64GB of room
export NODE_OPTIONS="--max-old-space-size=65536"

# Do initial second stage ceremony setup
(cd proof; snarkjs g16s $1 $2 test_verify_for_guest_0000.zkey)

# Add a pretend contributon, in reality each person in the ceremony would do this
(cd proof; echo 'Entropy' | snarkjs zkey contribute test_verify_for_guest_0000.zkey test_verify_for_guest_0001.zkey --name="1st Contributor Name" -v)

# Finalize
(cd proof; snarkjs zkey beacon test_verify_for_guest_0001.zkey test_verify_for_guest_final.zkey 0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f 10 -n="Final Beacon phase2")

# Export verification key
(cd proof; snarkjs zkey export verificationkey test_verify_for_guest_final.zkey test_verify_for_guest_verification_key.json)

# Export solidity smart contract
(cd proof; snarkjs zkey export solidityverifier test_verify_for_guest_final.zkey test_verifier.sol)
