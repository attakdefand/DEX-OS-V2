@echo off
set PATH=%PATH%;%USERPROFILE%\.cargo\bin
echo [STATUS] Building the project...
cargo build
if %errorlevel% equ 0 (
    echo [STATUS] Project built successfully!
) else (
    echo [ERROR] Failed to build the project
)
pause