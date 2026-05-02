fn main() {
    println!("cargo:rustc-check-cfg=cfg(p14_asm)");
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "linux" || target_os == "macos" {
        if target_arch == "x86_64" {
            println!("cargo:rerun-if-changed=../../asm/p14_x86_64.s");
            cc::Build::new()
                .file("../../asm/p14_x86_64.s")
                .compile("rco_p14_asm");
            println!("cargo:rustc-cfg=p14_asm");
        } else if target_arch == "aarch64" {
            println!("cargo:rerun-if-changed=../../asm/p14_aarch64.s");
            cc::Build::new()
                .file("../../asm/p14_aarch64.s")
                .compile("rco_p14_asm");
            println!("cargo:rustc-cfg=p14_asm");
        }
    }
}
