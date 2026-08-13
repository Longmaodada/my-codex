@echo off
cd /d "%~dp0"
call "D:\Development Environment Setup\VSI\Common7\Tools\VsDevCmd.bat" -arch=x64
if errorlevel 1 exit /b 1
npm.cmd run tauri:dev
