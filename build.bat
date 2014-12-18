@echo off
REM this file only exits because i don't know how to get cargo to do this
REM stuff

rustc --crate-type=dylib --crate-name=game -C prefer-dynamic src/lib.rs && move game.dll target/game.dll

IF NOT "%1" == "dll" (
    rustc --crate-type=bin --crate-name=rust_hero src/main.rs -l winmm -l gdi32 && move rust_hero.exe target/rust_hero.exe || del rust_hero.exe
)

IF "%1" == "run" (
    pushd data
    start ../target/rust_hero.exe
    popd
)
