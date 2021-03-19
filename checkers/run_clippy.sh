#!/bin/bash

find . -name Cargo.toml -printf '%h\n'|xargs -P 5 -I {} sh -c "cd {} && cargo clippy -- -D clippy::pedantic -D clippy::all || exit 255"
