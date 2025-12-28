extern crate cmake;

use cmake::Config;
use std::{env, path::PathBuf};

fn env_flag(name: &str) -> Option<bool> {
    env::var(name).ok().and_then(|value| match value.to_ascii_lowercase().as_str() {
        "1" | "true" | "on" | "yes" => Some(true),
        "0" | "false" | "off" | "no" => Some(false),
        _ => None,
    })
}

fn build_and_link_mlx_c() {
    let mut config = Config::new("src/mlx-c");
    config.very_verbose(true);
    config.define("CMAKE_INSTALL_PREFIX", ".");

    #[cfg(debug_assertions)]
    {
        config.define("CMAKE_BUILD_TYPE", "Debug");
    }

    #[cfg(not(debug_assertions))]
    {
        config.define("CMAKE_BUILD_TYPE", "Release");
    }

    let mut build_metal = cfg!(feature = "metal");
    if let Some(flag) = env_flag("MLX_RS_ENABLE_METAL") {
        build_metal = flag;
    }

    let mut build_accelerate = cfg!(feature = "accelerate");
    if let Some(flag) = env_flag("MLX_RS_ENABLE_ACCELERATE") {
        build_accelerate = flag;
    }

    config.define("MLX_BUILD_METAL", if build_metal { "ON" } else { "OFF" });
    config.define(
        "MLX_BUILD_ACCELERATE",
        if build_accelerate { "ON" } else { "OFF" },
    );

    // build the mlx-c project
    let dst = config.build();

    println!("cargo:rustc-link-search=native={}/build/lib", dst.display());
    println!("cargo:rustc-link-lib=static=mlx");
    println!("cargo:rustc-link-lib=static=mlxc");

    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=dylib=objc");
    println!("cargo:rustc-link-lib=framework=Foundation");

    if build_metal {
        println!("cargo:rustc-link-lib=framework=Metal");
    }

    if build_accelerate {
        println!("cargo:rustc-link-lib=framework=Accelerate");
    }
}

fn main() {
    build_and_link_mlx_c();

    // generate bindings
    let bindings = bindgen::Builder::default()
        .rust_target("1.73.0".parse().expect("rust-version"))
        .header("src/mlx-c/mlx/c/mlx.h")
        .header("src/mlx-c/mlx/c/linalg.h")
        .header("src/mlx-c/mlx/c/error.h")
        .header("src/mlx-c/mlx/c/transforms_impl.h")
        .clang_arg("-Isrc/mlx-c")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
