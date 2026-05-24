#!/bin/sh

set -e

cargo llvm-cov clean --workspace

./clippy.sh llvm-cov --no-report "$@"

cargo llvm-cov report --html
