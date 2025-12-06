#!/bin/bash
# Codex CLI wrapper for Linux/WSL
# This script allows you to run 'codex' instead of 'npx @openai/codex'

# Check if Node.js is installed
if ! command -v node &> /dev/null
then
    echo "Error: Node.js is not installed. Please install Node.js from https://nodejs.org/"
    exit 1
fi

# Run the codex command
npx @openai/codex "$@"