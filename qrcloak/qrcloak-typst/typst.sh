#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
#
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

fifo=$(mktemp -u)
mkfifo -m 600 "$fifo"

if [ -z "$1" ]; then
  echo "Usage: query.sh <path>"
  exit 1
fi

handle_sigint() {
  kill 0
  rm $fifo
}

trap handle_sigint SIGINT

preprocess() {
  res="$(typst query "$1" "<qrcloak>" --input "display=false")"
  values="$(echo "$res" | jq -r '.[].value' | jq -r tostring)"

  while IFS= read -r value; do
    path="$(echo "$value" | jq -r '.path')"
    data="$(echo "$value" | jq -r '.data')"
    keys="$(echo "$value" | jq -r '.keys | join(",")')"
    cmd="$(echo "$value" | jq -r '.cmd')"

    if [[ $path == "null" ]]; then
      echo "Missing path"
      exit 1
    fi

    if [[ $data == "null" ]]; then
      echo "Missing data for $path"
      exit 1
    fi

    if [[ $keys == "null" ]]; then
      echo "No age keys found for $path"
      exit 1
    fi

    sha="$(echo -n "$value" | sha256sum | cut -d ' ' -f 1)"

    if [ -f "$path.sha256" ]; then
      old_sha="$(cat "$path.sha256")"
      if [ "$sha" = "$old_sha" ]; then
        echo "Skipping $path: already exists" >&2
        continue
      fi
    fi

    if [[ $cmd != "null" ]]; then
      data="$(echo "$data" | "$cmd")"
    fi

    AGE_KEY="$keys" qrcloak-cli qrcode generate --age-key --text "$data" "$path"

    echo "Created $path" >&2

    echo -n "$sha" >"$path.sha256"
  done <<EOF
$values
EOF

  echo "$res" | jq 'reduce .[].value as $val ({}; .[$val.path] = { cmd: $val.cmd, data: $val.data, keys: $val.keys } )' | jq -r tostring
}

FILE="$1"
update() {
  preprocess "$FILE" >/dev/null
  touch -a "$FILE"
}

update
typst watch "$FILE" >"$fifo" 2>&1 &

while read -r line; do
  if [[ "$(echo "$line" | grep "^error:.*please run the preprocessor again")" != "" ]]; then
    echo "Rerunning the preprocessor: data changed" >&2
    update
  elif [[ "$(echo "$line" | grep "^error: file not found (searched at .*\.sha256)")" != "" ]]; then
    echo "Rerunning the preprocessor: path changed" >&2
    update
  fi
done <"$fifo"
