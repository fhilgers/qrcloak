#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
#
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

TEMPLATE='{
  "compression": "Gzip",
  "encryption": "AgePassphrase",
  "data": "$DATA"
}'

function fill_template() {
  echo "${TEMPLATE/\$DATA/$(cat)}"
}

if [[ -t 0 ]]; then
  echo "Please pipe the data to this script" >&2
  exit 1
fi

cat |
  gzip --stdout |
  age --encrypt --passphrase |
  base45 --wrap 0 |
  fill_template |
  jq -c . |
  qrencode -o $PWD/encrypted.png

echo "Encrypted output saved at $PWD/encrypted.png"
