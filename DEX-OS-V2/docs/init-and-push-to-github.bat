@echo off
echo Initializing Git repository and pushing to GitHub...

REM Check if Git is available
git --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Git is not installed or not in PATH.
    echo Please install Git from https://git-scm.com/downloads and try again.
    pause
    exit /b 1
)

REM Initialize Git repository
echo Initializing Git repository...
git init

REM Add all files
echo Adding all files to repository...
git add .

REM Make first commit
echo Making first commit...
git commit -m "Initial commit"

REM Rename master branch to main
echo Renaming branch to main...
git branch -M main

REM Add remote origin (replace with your actual GitHub repository URL)
echo Adding remote origin...
git remote add origin https://github.com/ctvc9988-wq/DEX-OS-V1.git

REM Push to GitHub
echo Pushing to GitHub...
git push -u origin main

echo Repository successfully pushed to GitHub!
pause