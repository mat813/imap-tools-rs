#!/bin/sh

if [ -z "$1" ]; then
  set -- clippy --all-targets
fi

# shellcheck disable=SC2086
cargo hack \
  --feature-powerset \
  --exclude-features default,__tls \
  --mutually-exclusive-features rustls,native-tls \
  "$@"
