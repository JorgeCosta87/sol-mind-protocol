#!/bin/bash

# Generate keypair for compute node
# This script generates a Solana keypair and saves it to id.json

set -e

KEYPAIR_PATH="${1:-/home/equilibrium/repos/turbine-course/sol-mind-protocol/nodes/compute-node/id.json}"

echo "Generating keypair for compute node..."

# Generate keypair using Solana CLI
solana-keygen new --outfile "$KEYPAIR_PATH" --no-bip39-passphrase --force

# Extract pubkey from the keypair file
PUBKEY=$(solana-keygen pubkey "$KEYPAIR_PATH")

echo ""
echo "Generated keypair for compute node:"
echo "Pubkey: $PUBKEY"
echo "Keypair saved to: $KEYPAIR_PATH"
echo ""
echo "The keypair file contains both public and private key."

