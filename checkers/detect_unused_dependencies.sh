#!/bin/bash

find . -name Cargo.toml -printf '%h\n'|xargs -I {} sh -c "cd {} && cargo udeps || exit 255"
