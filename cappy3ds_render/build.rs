fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("libcappy3ds")
        .generate_csharp_file("../win/NativeMethods.g.cs")
        .unwrap();
}
