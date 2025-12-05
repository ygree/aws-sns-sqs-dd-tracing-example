#!/bin/bash
set -e

# Check if .env exists
if [ ! -f .env ]; then
    echo "❌ Error: .env file not found"
    echo "   Please run ./setup.sh first"
    exit 1
fi

# Source .env and run producer
set -a && source .env && set +a && DD_SERVICE="sns-producer" cargo run --bin producer

