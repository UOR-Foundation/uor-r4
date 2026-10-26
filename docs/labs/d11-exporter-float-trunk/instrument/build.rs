//! Build the FBS core's host twin (C++17). The host twin only qualifies the
//! port against the Rust integer session; the panel is evaluated by the CUDA
//! module (CHARTER §3.5, §3.6).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/fbs_core.h");
    println!("cargo:rerun-if-changed=src/host_kernel.cpp");
    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .include("src")
        .warnings(true)
        .file("src/host_kernel.cpp")
        .compile("fbs_host_kernel");
}
