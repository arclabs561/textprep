#!/bin/sh
# Fetch a fixed sample of Tatoeba sentences per language into data/ (git
# ignored). Tatoeba sentences are CC BY 2.0 FR (https://tatoeba.org); they
# are not committed. Each file keeps every STEP-th sentence of the export,
# up to MAX lines, so a rerun picks the same lines while the export is
# unchanged.
#
# Usage: tools/lang-eval/fetch.sh
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
DATA="$HERE/data"
STEP=97
MAX=1500
LANGS="eng deu fra spa ita por nld swe dan nob fin hun rus tur ell ron cat"

mkdir -p "$DATA"
for lang in $LANGS; do
    out="$DATA/$lang.tsv"
    if [ -s "$out" ]; then
        echo "have $out"
        continue
    fi
    url="https://downloads.tatoeba.org/exports/per_language/$lang/${lang}_sentences.tsv.bz2"
    echo "fetch $url"
    curl -sSf -A "Mozilla/5.0" "$url" \
        | bzcat \
        | awk -F '\t' -v step="$STEP" -v max="$MAX" \
            'NR % step == 0 && n < max { print $2 "\t" $3; n++ }' \
        > "$out.tmp"
    mv "$out.tmp" "$out"
done
wc -l "$DATA"/*.tsv
