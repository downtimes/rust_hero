@echo off
REM this file only exits because i don't know how to get cargo to do this
REM stuff

cargo build
copy target/debug/rust_hero.exe target/rust_hero.exe
copy src/target/debug/game* target/game.dll

IF "%1" == "run" (
    pushd data
    start ../target/rust_hero.exe
    popd
)
