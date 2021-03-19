#!/bin/bash

find . -name Cargo.toml -printf '%h\n'|xargs -P 2 -I {} sh -c "cd {} && cargo udeps || exit 255"
