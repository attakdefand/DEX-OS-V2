@echo off
REM Codex CLI wrapper for Windows
REM This script allows you to run 'codex' instead of 'npx @openai/codex'

REM Check if Node.js is installed
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Node.js is not installed. Please install Node.js from https://nodejs.org/
    exit /b 1
)

REM Run the codex command
npx @openai/codex %*