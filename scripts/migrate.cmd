@echo off
setlocal
cd /d "%~dp0.."

where sqlx >nul 2>nul
if errorlevel 1 (
  echo sqlx-cli is required for migrations.
  echo Install it with: cargo install sqlx-cli --no-default-features --features native-tls,postgres
  exit /b 1
)

if not exist migrations mkdir migrations
sqlx migrate run
