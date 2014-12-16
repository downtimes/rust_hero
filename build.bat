@echo off
REM this file only exits because i don't know how to get cargo to do this
REM stuff

IF "%1" == "run" (
    cd target
    start rust_hero.exe
) ELSE (
    rustc --crate-type=dylib --crate-name=game -C prefer-dynamic src/lib.rs 
    rustc --crate-type=bin --crate-name=rust_hero src/main.rs -l winmm -l gdi32
    move game.dll target/game.dll
    move rust_hero.exe target/rust_hero.exe
)
