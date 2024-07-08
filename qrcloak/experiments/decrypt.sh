#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
#
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

zbarimg --quiet --raw encrypted.png |
  jq -r .data |
  base45 --decode |
  age --decrypt |
  gzip --decompress
