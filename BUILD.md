# Step by step build

## Build Rust

```bash
# building the library only
cargo build --lib --release

# building the library AND the cli binary
cargo build --release
```
## Build nyc-client FFI

```bash
cd newyorkcity_tss_client
```

```bash
make init
```

For iOS

```bash
make ios
```

Then, we use `cbindgen` to generate a C header file

```bash
cbindgen src/lib.rs -l c > libclient.h
```

Now that we have `libclient_lib.a` and `libclient.h`, we can copy them into our flutter plugin repository.

Getting back up to our project root

```bash
cd ..
```

As an example,

```bash
# This assumes that we have a flutter-rus-plugin git repo in the directory path at the same directory level as our project root

cp newyorkcity_tss_client/target/universal/release/libclient_lib.a ../flutter-rust-plugin/ios/

cp newyorkcity_tss_client/libclient.h ../flutter-rust-plugin/ios/Classes/
```
