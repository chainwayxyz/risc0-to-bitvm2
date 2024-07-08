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

# Preprocess the keys
(cd fflonk; snarkjs fflonk setup $1 $2 fflonk.zkey)

# Export verification key
(cd fflonk; snarkjs zkey export verificationkey fflonk.zkey fflonk_verification_key.json)

# Export solidity smart contract
(cd fflonk; snarkjs zkey export solidityverifier fflonk.zkey test_verifier.sol)
