cargo build -p cappy3ds_render
cp ./target/debug/libcappy3ds_render.dylib ./macos/
cbindgen --config ./cappy3ds_render/cbindgen.toml --lang c --crate cappy3ds_render --output ./bindings.h
