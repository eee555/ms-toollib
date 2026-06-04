@echo off
setlocal

if "%~1"=="" (
    echo Usage: build.bat ^<path-to-jdk^>
    echo Example: build.bat "D:\Eclipse Adoptium\jdk-21.0.11.10-hotspot"
    exit /b 1
)
set JAVA_HOME=%~1
if not exist "%JAVA_HOME%\include\jni.h" (
    echo ERROR: jni.h not found at "%JAVA_HOME%\include\jni.h"
    exit /b 1
)

set RUSTLIB=..\c\target\release\ms_toollib.lib
set WINLIBS=ws2_32.lib Advapi32.lib Iphlpapi.lib Psapi.lib user32.lib userenv.lib bcrypt.lib ntdll.lib

echo === Step 1: Build Rust static library (if needed) ===
if not exist "%RUSTLIB%" (
    pushd ..\c
    cargo build --release
    popd
)

echo === Step 2: Build JNI DLL ===
if not exist native mkdir native
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" >nul
cl /EHsc /I"%JAVA_HOME%\include" /I"%JAVA_HOME%\include\win32" /I..\c\include /LD /Fenative\ms_toollib_jni.dll src\main\c\jni_wrapper.c /link "%RUSTLIB%" %WINLIBS%
if errorlevel 1 exit /b 1
if exist native\ms_toollib_jni.lib del native\ms_toollib_jni.lib
if exist native\ms_toollib_jni.exp del native\ms_toollib_jni.exp

echo === Step 3: Build Java JAR ===
set CLASSES=target\classes
if not exist %CLASSES% mkdir %CLASSES%
"%JAVA_HOME%\bin\javac" -d %CLASSES% src\main\java\ms_toollib\*.java
if errorlevel 1 exit /b 1
cd %CLASSES%
"%JAVA_HOME%\bin\jar" cf ..\..\target\ms_toollib.jar ms_toollib\
cd ..\..

echo === Done ===
echo DLL: native/ms_toollib_jni.dll
echo JAR: target/ms_toollib.jar
