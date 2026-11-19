@echo off
rem =====================================================================
rem  TDPilot launcher - requests elevation, then runs the PowerShell UI
rem =====================================================================
setlocal EnableExtensions
cd /d "%~dp0"

set "PS1=%~dp0TDPilot.ps1"
if not exist "%PS1%" (
  echo.
  echo   TDPilot.ps1 was not found next to this launcher.
  echo.
  pause
  exit /b 1
)

rem --- already elevated?  fltmc fails for non-elevated shells on most systems
fltmc >nul 2>&1
if not errorlevel 1 goto :run

echo.
echo   Requesting administrator rights - this is needed to talk to the SMU.
echo.
powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Process -FilePath '%~f0' -Verb RunAs" >nul 2>&1
exit /b 0

:run
powershell -NoProfile -ExecutionPolicy Bypass -File "%PS1%" %*
if errorlevel 1 (
  echo.
  echo   TDPilot exited with an error. Press any key to close.
  pause >nul
)
endlocal & exit /b 0
