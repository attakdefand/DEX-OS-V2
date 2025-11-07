# Running Codex

Simply type `codex` to run the OpenAI Codex tool.

## Prerequisites

First, you need to install the @openai/codex package globally:

```bash
npm install -g @openai/codex
```

Alternatively, you can run it directly with npx without installing:

```bash
npx @openai/codex "Your prompt here"
```

## Usage Examples

```bash
# Run codex with a simple prompt
codex "Generate a function to calculate fibonacci numbers"

# Run codex with a specific model
codex exec --model gpt-5-codex "Your prompt here"

# Run codex with additional options
codex exec --model gpt-5-codex --temperature 0.7 "Create a React component for a login form"
```

## Setup

Make sure you have Node.js installed and the OpenAI API key set in your environment variables:

```bash
# On Windows
set OPENAI_API_KEY=your_api_key_here

# On Linux/WSL/Mac
export OPENAI_API_KEY=your_api_key_here
```

## Windows Users

For Windows users, simply run the `codex.bat` file:

```cmd
codex.bat "Your prompt here"
```

Or add the project directory to your PATH environment variable to run `codex` from anywhere.

## WSL/Linux Users

For WSL or Linux users, make the script executable and run it:

```bash
chmod +x codex.sh
./codex.sh "Your prompt here"
```

Or add the project directory to your PATH environment variable to run `codex` from anywhere.

git add .
git commit -m "Update RUNNING-CODEX-IN-WSL.md"