use std::env;
use std::path::PathBuf;

fn find_dir(env_key: &'static str, candidates: Vec<&'static str>) -> Option<PathBuf> {
    match env::var_os(env_key) {
        Some(val) => Some(PathBuf::from(&val)),
        _ => {
            for candidate in candidates {
                let path = PathBuf::from(candidate);
                if path.exists() {
                    return Some(path);
                }
            }

            None
        }
    }
}

fn main() {
    let cuda_lib_dir = find_dir(
        "CUDA_LIB_PATH",
        vec!["/usr/local/cuda/lib64", "/usr/local/cuda/lib"],
    ).expect("Could not find CUDA library path");

    

    let cuda_include = find_dir(
        "CUDA_INCLUDE_PATH",
        vec!["/opt/cuda/include", "/usr/local/cuda/include"],
    ).expect("Could not find CUDA include path");

    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", cuda_lib_dir.to_string_lossy());
    println!("cargo:rustc-link-arg=-L{}", cuda_lib_dir.to_string_lossy());

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", cuda_include.to_string_lossy()))
        .header(cuda_include.join("cuda.h").to_string_lossy())
        .blocklist_function("strtold")
        .blocklist_function("qecvt")
        .blocklist_function("qfcvt")
        .blocklist_function("qgcvt")
        .blocklist_function("qecvt_r")
        .blocklist_function("qfcvt_r")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
