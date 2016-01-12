#!/bin/sh

if [[ "$1" == "clean"]]; then
    cargo clean
else
    cargo build --features "internal"
    cp -f target/debug/deps/libgame-* target/debug/deps/libgame.so
fi

if [[ "$1" == "run" ]]; then
	pushd data
	RUST_BACKTRACE=1 ../target/debug/rust_hero
	popd
fi
