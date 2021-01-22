#!/bin/bash

set -ex

cargo build --release
for LANGUAGE in English French Portuguese; do
  ./target/release/mhorloge generate-phrases $LANGUAGE data/phrases/$LANGUAGE.json
  ./target/release/mhorloge generate-phrases $LANGUAGE:5 data/phrases/$LANGUAGE-5.json
done
./target/release/mhorloge generate-phrases English,French,Portuguese data/phrases/all.json
./target/release/mhorloge generate-phrases English:5,French:5,Portuguese:5 data/phrases/all-5.json