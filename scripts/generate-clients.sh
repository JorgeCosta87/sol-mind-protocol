#!/bin/bash
set -e

echo "Generating clients for all programs..."

echo "Generating sol-mind-protocol clients..."
npx codama run --all -c codama-sol-mind-protocol.json

echo "Generating token-manager clients..."
npx codama run --all -c codama-token-manager.json

echo "Generating marketplace clients..."
npx codama run --all -c codama-marketplace.json


echo "All clients generated successfully!"

