#!/bin/sh

if [ -z "$1" ]; then
    echo "Usage: query.sh <path>"
    exit 1
fi

res="$(typst query "$1" "<qrcloak>" --input "display=false")"

values="$(echo "$res" | jq -r '[.[].value][]' | jq -r tostring)"

PATHS=""

while IFS= read -r value; do
    path="$(echo "$value" | jq -r '.path')"
    data="$(echo "$value" | jq -r '.data')"
    keys="$(echo "$value" | jq -r '.keys | join(",")')"

    if [[ "$path" = "null" ]]; then
        echo "Missing path"
        exit 1
    fi

    if [[ "$data" = "null" ]]; then
        echo "Missing data for $path"
        exit 1
    fi

    if [[ "$keys" = "null" ]]; then
        echo "No age keys found for $path"
        exit 1
    fi

    sha="$(echo "$value" | sha256sum | cut -d ' ' -f 1)"


    if [ -f "$path.sha256" ]; then
        old_sha="$(cat "$path.sha256")"
        if [ "$sha" = "$old_sha" ]; then
            echo "Skipping $path: already exists"
            PATHS="$PATHS $path"
            continue
        fi
    fi
        
    AGE_KEY="$keys" qrcloak-cli qrcode generate --age-key --text "$data" "$path" || exit 1

    echo "Created $path"

    echo "$sha" > "$path.sha256"
    
    PATHS="$PATHS $path"

done <<EOF
$values
EOF

out="$(jq -n --arg paths "$PATHS" '[$paths | split(" ") | del(..|select(. == "")) | {(.[]): true}] | reduce .[] as $path ({}; . + $path)' | jq -r tostring)"

typst compile "$1" --input "paths=$out"

echo "Done"