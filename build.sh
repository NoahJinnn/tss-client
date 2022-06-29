#!/bin/bash

CURRENT=`pwd`

TARGET=$1

case $TARGET in
"armv7-linux-androideabi" | "aarch64-linux-android" | "x86_64-linux-android" | "i686-linux-android")
    sudo apt-get install musl-tools libssl-dev
    mkdir -p $HOME/usr/local
    cd $HOME/usr/local
    rustup target add $TARGET
    if [ "$TARGET" == "armv7-linux-androideabi" ]
    then
        TARGET_SHORT_NAME=armeabi-v7a
        GMP_TARGET="armv7a-linux-androideabi"
        wget https://github.com/ezsyfi/gmp/releases/download/6.2.1/${GMP_TARGET}.zip
        unzip ${GMP_TARGET}.zip
        cd $CURRENT
        mkdir -p target/${TARGET}/release/deps
        cp -rf $HOME/usr/local/${GMP_TARGET}/lib/libgmp* target/${TARGET}/release/deps/
    else
        TARGET_SHORT_NAME=$TARGET
        wget https://github.com/ezsyfi/gmp/releases/download/6.2.1/${TARGET}.zip
        unzip ${TARGET}.zip
        cd $CURRENT
        mkdir -p target/${TARGET}/release/deps
        cp -rf $HOME/usr/local/${TARGET}/lib/libgmp* target/${TARGET}/release/deps/
    fi
    cargo install cargo-ndk
    cargo ndk --target $TARGET_SHORT_NAME build --lib --bin cli --release
    ;;
"aarch64-apple-ios" | "x86_64-apple-ios")
    rustup target add $TARGET
    cargo install cargo-lipo
    cargo lipo --targets $TARGET --release
    ;;
"universal")
    rustup target add aarch64-apple-ios x86_64-apple-ios
    cargo install cargo-lipo
    cargo lipo --release
    ;;
*)
    sudo apt-get install musl-tools libssl-dev
    rustup target add $TARGET
    cargo build --target $TARGET --lib --bin cli --release
    ;;
esac

cd target/$TARGET/release
if [ "$TARGET" == "universal" ]
then
    zip -r /tmp/$TARGET-ios.zip .
else
    zip -r /tmp/$TARGET.zip .
fi