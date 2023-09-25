cargo build
cp ./target/debug/libcappy3ds.dylib ./macos/
cbindgen --config cbindgen.toml --crate Cappy3ds --output ../bindings.h
