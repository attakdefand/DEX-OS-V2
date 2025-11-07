#!/bin/bash

echo "Initializing Git repository and pushing to GitHub..."

# Check if Git is available
if ! command -v git &> /dev/null; then
    echo "Error: Git is not installed or not in PATH."
    echo "Please install Git and try again."
    exit 1
fi

# Initialize Git repository
echo "Initializing Git repository..."
git init

# Add all files
echo "Adding all files to repository..."
git add .

# Make first commit
echo "Making first commit..."
git commit -m "Initial commit"

# Rename master branch to main
echo "Renaming branch to main..."
git branch -M main

# Add remote origin (replace with your actual GitHub repository URL)
echo "Adding remote origin..."
git remote add origin https://github.com/ctvc9988-wq/DEX-OS-V1.git

# Push to GitHub
echo "Pushing to GitHub..."
git push -u origin main

echo "Repository successfully pushed to GitHub!"