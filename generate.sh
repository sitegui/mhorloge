#!/bin/bash

set -ex

cargo build --release
BIN=./target/release/mhorloge

function lyrics() {
    SONG="$1"

    "$BIN" grid "docs/data/$SONG-sync-lyrics.json" "docs/build/$SONG-grid.json" \
       --debug-tokens-svg "docs/build/$SONG-tokens.svg"
    "$BIN" lyrics-puzzle "docs/data/$SONG-sync-lyrics.json" "docs/build/$SONG-grid.json" \
        "docs/$SONG-grid.html"
}

lyrics beggin
lyrics feeling-good

"$BIN" time-phrases English:5,French:5,German:5,Portuguese:5 docs/build/time-phrases.json
"$BIN" grid docs/build/time-phrases.json docs/build/time-phrases-grid.json \
  --grid-html-output docs/time-phrases.html \
  --debug-tokens-svg docs/build/time-phrases-tokens.svg \
