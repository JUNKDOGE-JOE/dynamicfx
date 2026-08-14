@echo off
REM DynamicFx After Effects-only installer for explicit 2025/2026 targets.
REM Never install this AEX in shared MediaCore: Premiere Pro scans it too.
setlocal EnableExtensions DisableDelayedExpansion

set "VERSION=%~1"
if not "%VERSION%"=="2023" if not "%VERSION%"=="2024" if not "%VERSION%"=="2025" if not "%VERSION%"=="2026" (
    echo Usage: install.bat 2023 ^| 2024 ^| 2025 ^| 2026
    echo The version is required so the plug-in is never copied to MediaCore.
    exit /b 2
)

REM Prefer the release build, fall back to debug.
set "SRC=%~dp0..\target\release\dynamicfx.dll"
if not exist "%SRC%" set "SRC=%~dp0..\target\debug\dynamicfx.dll"
set "AE_ROOT=C:\Program Files\Adobe\Adobe After Effects %VERSION%\Support Files"
set "DEST=%AE_ROOT%\Plug-ins\DynamicFx"
set "SHARED_ROOT=C:\Program Files\Adobe\Common\Plug-ins\7.0\MediaCore"
set "LEGACY="
if exist "%SHARED_ROOT%\DynamicFx\DynamicFx.aex" set "LEGACY=%SHARED_ROOT%\DynamicFx\DynamicFx.aex"
if exist "%SHARED_ROOT%\DynamicFx\dynamicfx.dll" set "LEGACY=%SHARED_ROOT%\DynamicFx\dynamicfx.dll"
if exist "%SHARED_ROOT%\DynamicFx.aex" set "LEGACY=%SHARED_ROOT%\DynamicFx.aex"
if exist "%SHARED_ROOT%\dynamicfx.dll" set "LEGACY=%SHARED_ROOT%\dynamicfx.dll"

if not exist "%SRC%" (
    echo [ERROR] build artifact not found, run cargo build --release first
    exit /b 1
)

if not exist "%AE_ROOT%\AfterFX.exe" (
    echo [ERROR] After Effects %VERSION% was not found at:
    echo         %AE_ROOT%
    exit /b 1
)

REM A shared MediaCore copy is also scanned by Premiere and would create a
REM duplicate if an AE-specific copy were installed. Refuse safely; this
REM script never deletes, moves, launches, or terminates anything.
if defined LEGACY (
    echo [ERROR] legacy shared plug-in detected:
    echo         %LEGACY%
    echo Premiere Pro also scans %SHARED_ROOT%.
    echo Close Adobe hosts and move/remove that file explicitly before retrying.
    exit /b 3
)

REM Never overwrite an AEX while an AE host may have it loaded. Detection is
REM read-only; the installer does not terminate either process.
call :ensure_not_running "AfterFX.exe" "After Effects"
if errorlevel 1 exit /b %errorlevel%
call :ensure_not_running "aerender.exe" "aerender"
if errorlevel 1 exit /b %errorlevel%

net session >nul 2>&1
if errorlevel 1 (
    echo [ERROR] please right-click and run as administrator
    exit /b 1
)

if not exist "%DEST%" mkdir "%DEST%"
if errorlevel 1 (
    echo [ERROR] could not create %DEST%
    exit /b 1
)
echo Installing %SRC% ...
copy /b /y "%SRC%" "%DEST%\DynamicFx.aex" >nul
if errorlevel 1 (
    echo [ERROR] copy failed
    exit /b 1
)
echo [OK] installed for After Effects %VERSION% only:
echo      %DEST%
echo Restart After Effects, then find DynamicFx under Effect ^> DynamicFx
exit /b 0

:ensure_not_running
if not defined TEMP (
    echo [ERROR] TEMP is unavailable; cannot verify whether %~2 is running.
    exit /b 5
)
set "TASKLIST_RESULT=%TEMP%\DynamicFx-tasklist-%RANDOM%-%RANDOM%-%RANDOM%.tmp"
if exist "%TASKLIST_RESULT%" (
    echo [ERROR] could not reserve a temporary process-query file.
    exit /b 5
)
tasklist /FI "IMAGENAME eq %~1" /NH >"%TASKLIST_RESULT%" 2>&1
if errorlevel 1 goto :process_query_failed
find /I "%~1" "%TASKLIST_RESULT%" >nul 2>&1
if errorlevel 2 goto :process_query_failed
if not errorlevel 1 goto :process_is_running
del /q "%TASKLIST_RESULT%" >nul 2>&1
if exist "%TASKLIST_RESULT%" (
    echo [ERROR] could not clean up the temporary process-query file.
    exit /b 5
)
exit /b 0

:process_is_running
del /q "%TASKLIST_RESULT%" >nul 2>&1
echo [ERROR] %~2 is running. Close it normally or wait for it to finish, then retry.
exit /b 4

:process_query_failed
del /q "%TASKLIST_RESULT%" >nul 2>&1
echo [ERROR] could not verify whether %~2 is running; installation refused.
exit /b 5
