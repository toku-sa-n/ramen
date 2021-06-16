#!/bin/bash

find . -name Cargo.toml -printf '%h\n'|xargs -P 2 -I {} sh -c "cd {} && cargo clippy -- -D clippy::pedantic -D clippy::all || exit 255"
