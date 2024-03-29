#!/bin/sh

CLANG_SCRIPTS_PATH="$PWD/clang_scripts/"
MANIFEST_PATH="$PWD/qrypt-uniffi/Cargo.toml"
ANDROID_APP_PATH="$PWD/../qrypt-android/"

export PATH="$PATH:$CLANG_SCRIPTS_PATH"

cargo build --manifest-path=$MANIFEST_PATH --target aarch64-linux-android --release
cargo build --manifest-path=$MANIFEST_PATH --target armv7-linux-androideabi --release
cargo build --manifest-path=$MANIFEST_PATH --target i686-linux-android --release
cargo build --manifest-path=$MANIFEST_PATH --target x86_64-linux-android --release
cargo build --manifest-path=$MANIFEST_PATH --target x86_64-unknown-linux-gnu --release

SRC_PATH="$ANDROID_APP_PATH/app/src/"

cargo run --manifest-path=$MANIFEST_PATH generate --library target/aarch64-linux-android/release/libqrypt_uniffi.so --language kotlin --config ./uniffi.toml --out-dir "$SRC_PATH/arm64-v8a/java"
cargo run --manifest-path=$MANIFEST_PATH generate --library target/armv7-linux-androideabi/release/libqrypt_uniffi.so --language kotlin --config ./uniffi.toml --out-dir "$SRC_PATH/armeabi-v7a/java"
cargo run --manifest-path=$MANIFEST_PATH generate --library target/i686-linux-android/release/libqrypt_uniffi.so --language kotlin --config ./uniffi.toml --out-dir "$SRC_PATH/x86/java"
cargo run --manifest-path=$MANIFEST_PATH generate --library target/x86_64-linux-android/release/libqrypt_uniffi.so --language kotlin --config ./uniffi.toml --out-dir "$SRC_PATH/x86_64/java"

ARM64_PATH="$SRC_PATH/arm64-v8a/jniLibs/arm64-v8a"
ARM32_PATH="$SRC_PATH/armeabi-v7a/jniLibs/armeabi-v7a"
X86_PATH="$SRC_PATH/x86/jniLibs/x86"
X64_PATH="$SRC_PATH/x86_64/jniLibs/x86_64"

mkdir -p "$ARM64_PATH"
mkdir -p "$ARM32_PATH"
mkdir -p "$X86_PATH"
mkdir -p "$X64_PATH"

ln -sf "$PWD/target/aarch64-linux-android/release/libqrypt_uniffi.so" "$ARM64_PATH/"
ln -sf "$PWD/target/armv7-linux-androideabi/release/libqrypt_uniffi.so" "$ARM32_PATH/"
ln -sf "$PWD/target/i686-linux-android/release/libqrypt_uniffi.so" "$X86_PATH/"
ln -sf "$PWD/target/x86_64-linux-android/release/libqrypt_uniffi.so" "$X64_PATH/"
