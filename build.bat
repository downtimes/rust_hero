@echo off
REM this file only exits because i don't know how to get cargo to do this
REM stuff

IF "%1" == "clean" (
    cargo clean
) ELSE (
    cargo build
    copy /Y /B target\debug\rust_hero.exe target\rust_hero.exe
    copy /Y /B target\debug\deps\game* target\game.dll
)

IF "%1" == "run" (
    pushd data
    start ../target/rust_hero.exe
    popd
)
