#!/bin/bash

set -ex

cargo build --release
BIN=./target/release/mhorloge

function lyrics() {
    SONG="$1"

    "$BIN" grid "docs/data/$SONG-sync-lyrics.json" "docs/build/$SONG-grid.json" \
       --debug-tokens-svg "docs/build/$SONG-tokens.svg"
    "$BIN" lyrics-puzzle "docs/data/$SONG-sync-lyrics.json" "docs/build/$SONG-grid.json" \
        "docs/$SONG.html"
}

function time_phrases() {
    PRECISION="$1"

    "$BIN" time-phrases \
      "English:$PRECISION,French:$PRECISION,German:$PRECISION,Portuguese:$PRECISION" \
      "docs/build/time-phrases-$PRECISION.json"
    "$BIN" grid "docs/build/time-phrases-$PRECISION.json" \
      "docs/build/time-phrases-$PRECISION-grid.json" \
      --grid-html-output "docs/time-phrases-$PRECISION.html" \
      --debug-tokens-svg "docs/build/time-phrases-$PRECISION-tokens.svg"
}

lyrics beggin
lyrics feeling-good
lyrics shining-light

time_phrases 1
time_phrases 5
time_phrases 15
