cargo build -p cappy3ds
cp ./target/debug/libcappy3ds.dylib ./macos/
cbindgen --config ./Cappy3ds/cbindgen.toml --crate cappy3ds --output ./bindings.h
