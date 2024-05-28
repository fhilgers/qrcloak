#!/usr/bin/env bash

zbarimg --quiet --raw encrypted.png |
  jq -r .data |
  base45 --decode |
  age --decrypt |
  gzip --decompress
