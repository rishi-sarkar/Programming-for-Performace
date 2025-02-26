use std::process::Command;

// Depends on NVCC version on the system, which indicates maximum supported GCC version
const MAX_GCC_VERSION: u32 = 7;  // was 8

fn main() {
    let filename = "kernel/kernel.cu";

    // Compile kernel using NVCC
    let out = Command::new("nvcc")
        .args(&["-ccbin", &format!("gcc-{}", MAX_GCC_VERSION)])
        .args(&[filename, "--ptx", "-o", "kernel/kernel.ptx"])
        .output()
        .unwrap();

    if !out.status.success() {
        panic!("{}", std::str::from_utf8(&out.stderr).unwrap());
    } else {
        println!("cargo:rerun-if-changed={}", filename);
    }
}
