#!/bin/sh

if [[ "$1" == "release" ]]; then
	cargo build --release --features "internal"
else 
	cargo build --features "internal"
fi

if [[ "$2" == "run" ]]; then
	if [[ "$1" == "release" ]]; then
		pushd data
		RUST_BACKTRACE=1 ../target/release/rust_hero
		popd
	else
		pushd data
		RUST_BACKTRACE=1 ../target/debug/rust_hero
		popd
	fi
fi
