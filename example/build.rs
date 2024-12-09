fn main() {
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Resources/pyembed/lib");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/./pyembed/lib");

    tauri_build::build();
}
