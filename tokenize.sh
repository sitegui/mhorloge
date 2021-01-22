#!/bin/bash

set -ex

cargo build --release
for PHRASES in data/phrases/*; do
  NAME=$(basename "$PHRASES" .json)
  ./target/release/mhorloge tokenize "$PHRASES" "data/tokens/$NAME.json" --output-svg "data/tokens/$NAME.svg"
done