@echo off
set PATH=%PATH%;%USERPROFILE%\.cargo\bin
echo Testing Rust installation...
rustc --version
if %errorlevel% equ 0 (
    echo Rust is installed!
) else (
    echo Rust is not installed!
)