@echo off
REM Remove AIRA Desktop binaries from user Programs (QUEUE #92).
setlocal EnableExtensions

if not defined INSTALL_DEST (
  if defined LOCALAPPDATA (
    set "DEST=%LOCALAPPDATA%\Programs\AIRA"
  ) else (
    echo LOCALAPPDATA is not set >&2
    exit /b 1
  )
) else (
  set "DEST=%INSTALL_DEST%"
)

if exist "%DEST%\aira-desktop.exe" del /F /Q "%DEST%\aira-desktop.exe"
if exist "%DEST%\aira-node.exe" del /F /Q "%DEST%\aira-node.exe"
if exist "%DEST%\aira.exe" del /F /Q "%DEST%\aira.exe"
if exist "%DEST%" rmdir "%DEST%" 2>nul

echo OK: AIRA Desktop uninstalled (data under %%LOCALAPPDATA%%\AIRA kept)
endlocal
