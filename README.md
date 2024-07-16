<!--
SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com"

SPDX-License-Identifier: CC-BY-4.0
-->

# QRCloak

[![REUSE status](https://api.reuse.software/badge/github.com/fhilgers/qrcloak)](https://api.reuse.software/info/github.com/fhilgers/qrcloak)

QRCloak is a project for embedding encryption information 
inside of printed documents. 


# Building

The project is built with [Bazel](https://bazel.build/), but the
core part in Rust can also be built with Cargo. For easy local 
development, nix flakes and direnv are used.


## Bazel

Bazel handles all dependencies, so for typical build actions, nothing
has to be installed other than bazel itself. However, as the current
Android rules for bazel still require a path to the Android SDK and NDK
from the local file system or environment variable, they have to be 
installed manually first.

After installing the Android SDK and NDK, set the following environment
variables:

```
ANDROID_HOME="/path/to/android/sdk"
ANDROID_NDK_HOME="/path/to/android/ndk"
```

To discover the targets in the project, bazel's query system can be used:

```
bazel query //...
```

The relevant targets to build are:

```
bazel build //qrcloak/qrcloak-core # The core library
bazel build //qrcloak/qrcloak-cli # The command line interface
bazel build //qrcloak/qrcloak-pandoc # The pandoc filter
bazel build //qrcloak/qrcloak-word # The word add-in
bazel build //qrcloak/qrcloak-android # The android app
```

Bazel automatically wires up all inter project dependencies and builds
the shared library or wasm files from the core library to be used by
the word add-in and android app.
