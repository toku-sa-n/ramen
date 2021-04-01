#!/bin/bash

set -e

find . -name Cargo.toml -printf '%h\n'|xargs -P 2 -I {} sh -c "cd {} && cargo test || exit 255"
