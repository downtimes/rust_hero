#!/bin/sh

cargo build
cp target/debug/rust_hero target/rust_hero
cp src/target/debug/libgame* target/libgame.so

if [[ "$1" == "run" ]]; then
	pushd data
	RUST_BACKTRACE=1 ../target/rust_hero
	popd
fi
