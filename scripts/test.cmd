@echo off
setlocal
cd /d "%~dp0.."
cargo test || exit /b %errorlevel%
cd /d "%~dp0..\frontend"
npm.cmd test
