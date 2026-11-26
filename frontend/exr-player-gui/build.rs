fn main() {
    #[cfg(unix)]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let lib_dir = std::path::Path::new(&manifest_dir).join("../../dist/engine/lib");
        if lib_dir.exists() {
            println!("cargo:rustc-link-search=native={}", lib_dir.display());
            println!("cargo:rustc-link-lib=dylib=exr-x86");
            println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../../../../dist/engine/lib");
            println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/engine/lib");
            println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib");
        }
    }
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/logo/exr.ico");
        let _ = res.compile();
    }
}
