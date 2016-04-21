@echo off
REM this file only exits because i don't know how to get cargo to do this
REM stuff

IF "%1" == "clean" (
    cargo clean
) ELSE (
    cargo build --features "internal"
    copy /Y /B target\debug\deps\game*.dll target\debug\game.dll
)

IF "%1" == "run" (
    pushd data
    start ..\target\debug\rust_hero.exe
    popd
)
