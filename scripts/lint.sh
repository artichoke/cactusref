#!/usr/bin/env bash

set -euo pipefail

yarn install
PATH="$(yarn bin):$PATH"
export PATH
cd "$(pkg-dir)"

set -x

# Yarn orchestration

## Lint package.json
pjv

# Rust sources

## Format with rustfmt
cargo fmt
## Lint with Clippy
cargo clippy --all-targets --all-features
## Lint docs
cargo doc --no-deps --all

# Shell sources

## Format with shfmt
shfmt -f . | grep -v target/ | grep -v node_modules/ | grep -v /vendor/ | xargs shfmt -i 2 -ci -s -w
## Lint with shellcheck
shfmt -f . | grep -v target/ | grep -v node_modules/ | grep -v /vendor/ | xargs shellcheck

# Web sources

## Format with prettier
./scripts/format-text.sh --format "css"
./scripts/format-text.sh --format "html"
./scripts/format-text.sh --format "js"
./scripts/format-text.sh --format "json"
./scripts/format-text.sh --format "yaml"
./scripts/format-text.sh --format "yml"
## Lint with eslint
# yarn run eslint --fix --ext .html,.js .

# Text sources

## Format with prettier
./scripts/format-text.sh --format "md"
