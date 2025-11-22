#!/bin/bash
set -e

echo "Generating clients for all programs..."

echo "Generating sol-mind-protocol clients..."
npx codama run --all -c codama-sol-mind-protocol.json

echo "Generating nft-operations clients..."
npx codama run --all -c codama-nft-operations.json

echo "All clients generated successfully!"

