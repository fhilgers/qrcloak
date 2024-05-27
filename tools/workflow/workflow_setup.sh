#!/usr/bin/env bash

set -eo pipefail

install_jre() {
	export DEBIAN_FRONTEND=noninteractive
	apt-get update
	apt-get install --yes openjdk-17-jre
}

sudo bash -c "$(declare -f install_jre); install_jre"

VERSION_LONG="10406996"
FILENAME="commandlinetools-linux-${VERSION_LONG}_latest.zip"
SHA256="8919e8752979db73d8321e9babe2caedcc393750817c1a5f56c128ec442fb540"

if [[ ! -d ~/.android/sdk ]]; then
	wget -O "$FILENAME" "https://dl.google.com/android/repository/commandlinetools-linux-${VERSION_LONG}_latest.zip"
	echo "${SHA256}  ${FILENAME}" >SHA256SUMS
	sha256sum -c SHA256SUMS
	mkdir -p ~/.android/sdk/cmdline-tools/
	unzip $FILENAME -d ~/.android/sdk/tmp
	mv ~/.android/sdk/tmp/cmdline-tools ~/.android/sdk/cmdline-tools/latest
fi

(yes || true) | ~/.android/sdk/cmdline-tools/latest/bin/sdkmanager --licenses

~/.android/sdk/cmdline-tools/latest/bin/sdkmanager "ndk;26.3.11579264" "build-tools;34.0.0" "platforms;android-34"
