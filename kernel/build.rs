#![allow(clippy::print_literal)]

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // kernel address
    println!("cargo:rustc-env=kernel_address={}", 0x80000000usize);
    // values: usize

    // logging level
    println!("cargo:rustc-env=logging_level={}", "trace");
    // values: "trace" | "debug" | "info" | "warn" | "error"

    // backtrace limit
    println!("cargo:rustc-env=backtrace_limit={}", 64);
    // values: usize

    match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
        "riscv64" => {
            println!("cargo:rerun-if-changed=src/arch/riscv64/linker.ld");

            // RISC-V paging
            println!("cargo:rustc-cfg=riscv64_paging=\"{}\"", "sv39");
            // values: "sv39" | "sv48"
        }
        _ => {
            panic!("unknown target_arch");
        }
    }
}
