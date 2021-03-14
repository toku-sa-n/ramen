#!/bin/bash

find . -name Cargo.toml -printf '%h\n'|xargs -I {} sh -c "cd {} && cargo clippy -- -D clippy::pedantic -D clippy::all || exit 255"
