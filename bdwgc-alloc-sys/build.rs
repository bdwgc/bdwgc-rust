//! A build script.

use core::error::Error;
use std::{env, path::PathBuf};

// cspell: ignore reconf

const LIB_ATOMIC_OPS_DIR: &str = "vendor/libatomic_ops";
const LIB_GC_DIR: &str = "vendor/bdwgc";

fn main() -> Result<(), Box<dyn Error>> {
    build_library()?;

    bindgen::builder()
        .header(format!("{LIB_GC_DIR}/include/gc.h"))
        .clang_arg(format!("-I{LIB_GC_DIR}/include"))
        .clang_arg("-DGC_THREADS")
        .use_core()
        .allowlist_item("GC_.*")
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .generate()?
        .write_to_file(PathBuf::from(env::var("OUT_DIR")?).join("bindings.rs"))?;

    Ok(())
}

cfg_select! {
    feature = "cmake" => {
        fn build_library() -> Result<(), Box<dyn Error>> {
            use cmake::Config;
            use std::path::Path;

            let libatomic_include_path = Path::new(LIB_ATOMIC_OPS_DIR)
                .join("src")
                .canonicalize()?
                .display()
                .to_string()
                .replace(r"\\?\", "");

            let dst = Config::new(LIB_GC_DIR)
                .profile("Release")
                .define("GC_BUILD_SHARED_LIBS", "FALSE")
                .cflag(format!("-I{}", libatomic_include_path))
                .build();

            println!(
                "cargo:rustc-link-search=native={}",
                dst.join("lib").display()
            );
            println!("cargo:rustc-link-lib=static=gc");

            Ok(())
        }
    }
    feature = "autotools" => {
        fn build_library() -> Result<(), Box<dyn Error>> {
            let dst = autotools::Config::new(LIB_ATOMIC_OPS_DIR)
                .reconf("-i")
                .cflag("-fPIC")
                .build();

            println!(
                "cargo:rustc-link-search=native={}",
                dst.join("lib").display()
            );
            println!("cargo:rustc-link-lib=static=atomic_ops");

            let dst = autotools::Config::new(LIB_GC_DIR)
                .reconf("-i")
                .cflag(format!(
                    // spell-checker: disable-next-line
                    "-I{} -L/lib/x86_64-linux-gnu -lpthread -fPIC",
                    dst.join("include").display()
                ))
                .build();

            println!(
                "cargo:rustc-link-search=native={}",
                dst.join("lib").display()
            );
            println!("cargo:rustc-link-lib=static=gc");

            for dir in &[LIB_ATOMIC_OPS_DIR, LIB_GC_DIR] {
                std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("cd {dir} && git clean -dfx"))
                    .output()?;
            }

            Ok(())
        }
    }
}
