#!/bin/bash

# Configuration
NETWORK="signet"
RPC_USER="admin"
RPC_PASSWORD="admin"
RPC_PORT=38332
OUTPUT_FILE="signet-headers.bin"

# Function to make RPC calls
bitcoin_cli() {
    bitcoin-cli -"$NETWORK" -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASSWORD" -rpcport="$RPC_PORT" "$@"
}

# Get the current block height
height=$(bitcoin_cli getblockcount)
echo "Current block height: $height"

# Create/clear the output file
> "$OUTPUT_FILE"

# Fetch and serialize headers
for ((i=0; i<=50000; i++)); do
    if [ $((i % 1000)) -eq 0 ]; then
        echo "Processing block $i of 50000..."
    fi
    
    # Get block hash for current height
    blockhash=$(bitcoin_cli getblockhash "$i")
    
    # Get block header and extract the hex data
    header=$(bitcoin_cli getblockheader "$blockhash" false)
    
    # Convert hex to binary and append to file
    echo "$header" | xxd -r -p >> "$OUTPUT_FILE"
done

echo "Done! Headers saved to $OUTPUT_FILE"
echo "Total file size: $(wc -c < "$OUTPUT_FILE") bytes"