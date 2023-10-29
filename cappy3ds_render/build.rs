fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("cappy3ds_render")
        .generate_csharp_file("../win/NativeMethods.g.cs")
        .unwrap();
}
