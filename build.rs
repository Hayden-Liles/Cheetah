use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // Compile the any_to_string.c file
    let any_to_string_file = "src/compiler/runtime/any_to_string.c";
    if !Path::new(any_to_string_file).exists() {
        panic!("Could not find {}", any_to_string_file);
    }

    // Compile any_to_string.c
    let status = Command::new("cc")
        .args(&[any_to_string_file, "-c", "-o"])
        .arg(&format!("{}/any_to_string.o", out_dir))
        .status()
        .expect("Failed to compile any_to_string.c");

    if !status.success() {
        panic!("Failed to compile any_to_string.c");
    }

    // Compile the print_list_any.c file
    let print_list_any_file = "src/compiler/runtime/print_list_any.c";
    if !Path::new(print_list_any_file).exists() {
        panic!("Could not find {}", print_list_any_file);
    }

    // Compile print_list_any.c
    let status = Command::new("cc")
        .args(&[print_list_any_file, "-c", "-o"])
        .arg(&format!("{}/print_list_any.o", out_dir))
        .status()
        .expect("Failed to compile print_list_any.c");

    if !status.success() {
        panic!("Failed to compile print_list_any.c");
    }

    // Create a static library with both object files
    let status = Command::new("ar")
        .args(&["crus", "libcheetah_runtime.a", "any_to_string.o", "print_list_any.o"])
        .current_dir(&Path::new(&out_dir))
        .status()
        .expect("Failed to create static library");

    if !status.success() {
        panic!("Failed to create static library");
    }

    // Tell cargo to link the library
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=cheetah_runtime");

    // Tell cargo to re-run this if the C files change
    println!("cargo:rerun-if-changed={}", any_to_string_file);
    println!("cargo:rerun-if-changed={}", print_list_any_file);
}
