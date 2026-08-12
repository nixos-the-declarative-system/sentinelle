#!/bin/sh
# Builds the official release of sentinelle.
#
# The binary is compiled in a Debian 12 container (image
# rust:1-bookworm), so it is dynamically linked against the glibc and
# OpenSSL 3 of a mainstream distribution, with the ELF interpreter at
# the FHS rendezvous (/lib64/ld-linux-x86-64.so.2): exactly the
# "binary from elsewhere" the book needs.
#
# Output: dist/sentinelle-linux-x86_64.tar.gz, which contains
# sentinelle-<version>/ (stripped binary + LICENSE + README.md).

set -eu
cd "$(dirname "$0")"

VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
IMAGE=docker.io/library/rust:1-bookworm

podman run --rm -v "$PWD":/src -w /src -e CARGO_TARGET_DIR=/src/target-dist \
  "$IMAGE" cargo build --release --locked

rm -rf dist
mkdir -p "dist/sentinelle-$VERSION"
cp target-dist/release/sentinelle "dist/sentinelle-$VERSION/"
cp LICENSE README.md "dist/sentinelle-$VERSION/"
tar -C dist -czf dist/sentinelle-linux-x86_64.tar.gz "sentinelle-$VERSION"

echo
echo "— Artifact check —"
file "dist/sentinelle-$VERSION/sentinelle"
ls -lh "dist/sentinelle-$VERSION/sentinelle" dist/sentinelle-linux-x86_64.tar.gz
