#!/bin/sh

rustc --crate-type=dylib --crate-name=game -g -C prefer-dynamic src/lib.rs && mv libgame.so target/game.dll

if [[ "$1" != "dll" ]]; then
    rustc --crate-type=bin --crate-name=rust_hero -l SDL2 -l pthread -g src/main.rs && mv rust_hero target/rust_hero
fi

if [[ "$1" == "run" ]]; then
	pushd data
	../target/rust_hero
	popd
fi
