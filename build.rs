const LIB_ATOMIC_OPS_DIR: &str = "vendor/libatomic_ops";
const LIB_GC_DIR: &str = "vendor/bdwgc";

#[cfg(feature = "autotools")]
fn main() {
    for dir in &[LIB_ATOMIC_OPS_DIR, LIB_GC_DIR] {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("cd {dir} && ./autogen.sh"))
            .output()
            .unwrap();
    }

    let dst = autotools::Config::new(LIB_ATOMIC_OPS_DIR)
        .cflag("-fPIC")
        .build();

    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=atomic_ops");

    let dst = autotools::Config::new(LIB_GC_DIR)
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
            .output()
            .unwrap();
    }
}

#[cfg(feature = "cmake")]
fn main() {
    use cmake::Config;
    use std::path::Path;

    let libatomic_include_path = Path::new(LIB_ATOMIC_OPS_DIR)
        .join("src")
        .canonicalize()
        .unwrap()
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
}
