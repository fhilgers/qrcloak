#!/bin/bash

mkdir tmp
cd tmp

mkdir -p node_modules/@fhilgers/
ln -s $PWD/../$QRCLOAK node_modules/@fhilgers/qrcloak

cp ../$TESTFILE ./

../$BUN test
