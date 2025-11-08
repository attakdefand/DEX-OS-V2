# Git Installation Guide

This guide will help you install Git on your system so you can initialize and push this repository to GitHub.

## Windows Installation

1. Download Git for Windows from the official website: https://git-scm.com/download/win
2. Run the installer and follow the installation wizard
3. During installation, make sure to select these options:
   - Add Git to PATH (this allows you to use Git from Command Prompt)
   - Choose default editor (recommended: Vim or VS Code if you have it)
   - Choose default branch name as "main"
4. After installation, restart your command prompt or PowerShell
5. Verify installation by running:
   ```cmd
   git --version
   ```

## Initializing and Pushing the Repository

After installing Git, you can initialize the repository and push it to GitHub using the provided scripts:

### Using the Batch Script (Windows)
```cmd
init-and-push-to-github.bat
```

### Using the Shell Script (Unix-like systems)
```bash
chmod +x init-and-push-to-github.sh
./init-and-push-to-github.sh
```

## Manual Commands

If you prefer to run the commands manually:

```bash
# Initialize Git repository
git init

# Add all files
git add .

# Make first commit
git commit -m "Initial commit"

# Rename master branch to main (if needed)
git branch -M main

# Add remote origin (replace with your actual GitHub repository URL)
git remote add origin https://github.com/ctvc9988-wq/DEX-OS-V1.git

# Push to GitHub
git push -u origin main
```

Note: If you have Rust installed, it's recommended to generate the Cargo.lock file before committing:

```bash
# Generate Cargo.lock file
cargo check

# Add and commit the Cargo.lock file
git add Cargo.lock
git commit -m "Add Cargo.lock file"
```

## Troubleshooting

If you encounter any issues:

1. Make sure Git is properly installed and added to PATH
2. Ensure your GitHub repository exists (create it first on GitHub)
3. Check that you have proper permissions to push to the repository
4. If you get authentication errors, consider using a Personal Access Token instead of password