#![allow(clippy::print_literal)]

fn main() {
    // log level
    println!("cargo:rustc-env=log_level={}", "trace");
    // values: "trace" | "debug" | "info" | "warn" | "error"

    // initproc file
    println!(
        "cargo:rustc-env=memfs_initproc={}",
        "../../../crates/nekos-initproc/target/riscv64gc-unknown-none-elf/debug/nekos-initproc"
    );
    // values: string
}
