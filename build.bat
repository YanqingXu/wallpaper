@echo off
setlocal EnableExtensions

set "ROOT=%~dp0"
set "ROOT=%ROOT:~0,-1%"
set "EXE_NAME=wallpaper-switcher.exe"
set "TAURI_EXE=%ROOT%\src-tauri\target\release\%EXE_NAME%"
set "OUTPUT_DIR=%ROOT%\release"
set "OUTPUT_EXE=%OUTPUT_DIR%\%EXE_NAME%"

cd /d "%ROOT%"

if exist "%USERPROFILE%\.cargo\bin" (
  set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)

echo.
echo === Wallpaper Switcher build ===
echo Project: %ROOT%
echo.

where npm >nul 2>nul
if errorlevel 1 (
  echo [ERROR] npm was not found. Please install Node.js first.
  exit /b 1
)

where cargo >nul 2>nul
if errorlevel 1 (
  echo [ERROR] cargo was not found. Please install Rust first.
  echo         If Rust is already installed, reopen the terminal and try again.
  exit /b 1
)

echo [1/3] Installing frontend dependencies...
call npm install
if errorlevel 1 (
  echo [ERROR] npm install failed.
  exit /b 1
)

echo.
echo [2/3] Building Tauri release executable...
call npm run tauri build
if errorlevel 1 (
  echo [ERROR] Tauri build failed.
  exit /b 1
)

if not exist "%TAURI_EXE%" (
  echo [ERROR] Build finished but exe was not found:
  echo         %TAURI_EXE%
  exit /b 1
)

echo.
echo [3/3] Copying executable to release folder...
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"
copy /Y "%TAURI_EXE%" "%OUTPUT_EXE%" >nul
if errorlevel 1 (
  echo [ERROR] Failed to copy exe to:
  echo         %OUTPUT_EXE%
  exit /b 1
)

echo.
echo Build complete.
echo EXE: %OUTPUT_EXE%
echo.

endlocal
