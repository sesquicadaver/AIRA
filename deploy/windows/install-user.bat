@echo off
REM Install AIRA Desktop Windows Developer Preview (QUEUE #92).
REM
REM Layouts:
REM 1) Extracted zip (this script at package root with bin\*.exe)
REM 2) Repo: deploy\windows\install-user.bat with target\release\*.exe built
setlocal EnableExtensions EnableDelayedExpansion

set "HERE=%~dp0"
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

set "REPO_RELEASE=%HERE%..\..\target\release"

if not exist "%DEST%" mkdir "%DEST%"

for %%B in (aira aira-node aira-desktop) do (
  set "SRC=%HERE%bin\%%B.exe"
  if exist "!SRC!" (
    copy /Y "!SRC!" "%DEST%\%%B.exe" >nul
    echo copied %%B.exe
  ) else if exist "%REPO_RELEASE%\%%B.exe" (
    copy /Y "%REPO_RELEASE%\%%B.exe" "%DEST%\%%B.exe" >nul
    echo staged %%B.exe from release build
  ) else (
    echo missing binary: %%B.exe (extract zip or build release) >&2
    exit /b 1
  )
)

echo OK: installed to %DEST%
echo Run: "%DEST%\aira-desktop.exe"
echo CLI: "%DEST%\aira.exe" desktop status
endlocal
