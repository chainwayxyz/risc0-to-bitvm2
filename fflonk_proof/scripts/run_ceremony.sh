#!/bin/bash

set -eoux

# We presume node and snarkyjs are installed system wide.
# This script is meant to be run via
# scripts/run_ceremony.sh <stark_verify.r1cs> <power_of_tau_23> only
# AFTER the scripts/build.sh has been ran.
#
# It outputs everything to groth16

# Make all the node commands actually have 32GB of room
export NODE_OPTIONS="--max-old-space-size=65536"

# Do initial second stage ceremony setup
(cd groth16; snarkjs g16s $1 $2 verify_for_guest_0000.zkey)

# Add a pretend contributon, in reality each person in the ceremony would do this
(cd groth16; echo 'Entropy' | snarkjs zkey contribute verify_for_guest_0000.zkey verify_for_guest_0001.zkey --name="1st Contributor Name" -v)

# Finalize
(cd groth16; snarkjs zkey beacon verify_for_guest_0001.zkey verify_for_guest_final.zkey 0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f 10 -n="Final Beacon phase2")

# Export verification key
(cd groth16; snarkjs zkey export verificationkey verify_for_guest_final.zkey verify_for_guest_verification_key.json)

# Export solidity smart contract
(cd groth16; snarkjs zkey export solidityverifier verify_for_guest_final.zkey verifier.sol)
