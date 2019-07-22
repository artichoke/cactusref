#!/usr/bin/env bash

set -euo pipefail

yarn install
PATH="$(yarn bin):$PATH"
export PATH
cd "$(pkg-dir)"

set -x

# Rust sources

## Format with rustfmt
cargo fmt
## Lint with Clippy
cargo clippy --all-targets --all-features
## Lint docs
cargo doc --no-deps --all

# Shell sources

## Format with shfmt
shfmt -f . | grep -v target/ | grep -v node_modules/ | xargs shfmt -i 2 -ci -s -w
## Lint with shellcheck
shfmt -f . | grep -v target/ | grep -v node_modules/ | xargs shellcheck

# Text sources (e.g. HTML, Markdown)

## Format with prettier
prettier --write --prose-wrap always '**/*.{css,html,js,json,md}'
