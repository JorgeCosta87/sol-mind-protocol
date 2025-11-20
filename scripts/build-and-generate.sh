#!/bin/bash
set -e

echo "Building programs..."
anchor build

echo "Generating clients for all programs..."

echo "Generating sol-mind-protocol clients..."
npx codama run --all -c codama-sol-mind-protocol.json

echo "Generating nft-operations clients..."
npx codama run --all -c codama-nft-operations.json

echo "Build and client generation completed successfully!"

