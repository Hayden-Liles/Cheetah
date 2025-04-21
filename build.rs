use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Compile the any_to_string.c file
    let out_dir = env::var("OUT_DIR").unwrap();
    let c_file = "src/compiler/runtime/any_to_string.c";
    
    // Check if the C file exists
    if !Path::new(c_file).exists() {
        panic!("Could not find {}", c_file);
    }
    
    // Compile the C file
    let status = Command::new("cc")
        .args(&[c_file, "-c", "-o"])
        .arg(&format!("{}/any_to_string.o", out_dir))
        .status()
        .expect("Failed to compile any_to_string.c");
    
    if !status.success() {
        panic!("Failed to compile any_to_string.c");
    }
    
    // Create a static library
    let status = Command::new("ar")
        .args(&["crus", "libany_to_string.a", "any_to_string.o"])
        .current_dir(&Path::new(&out_dir))
        .status()
        .expect("Failed to create static library");
    
    if !status.success() {
        panic!("Failed to create static library");
    }
    
    // Tell cargo to link the library
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=any_to_string");
    
    // Tell cargo to re-run this if the C file changes
    println!("cargo:rerun-if-changed={}", c_file);
}
