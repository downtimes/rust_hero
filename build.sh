#!/bin/sh

cargo build
cp -f target/debug/deps/libgame-* target/debug/deps/libgame.so

if [[ "$1" == "run" ]]; then
	pushd data
	RUST_BACKTRACE=1 ../target/debug/rust_hero
	popd
fi
