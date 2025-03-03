use std::env;
use std::{path::Path, process::Command};

fn main() {
    println!("cargo:rustc-link-search=../../../osbook/devenv/x86_64-elf/lib");
    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=c");

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search={}", out_dir);
    println!("cargo:rustc-link-lib=static=usb");

    if env::var("SKIP_BUILD_SCRIPT").is_err() {
        let files = [
            "src/usb/lib.cpp",
            "src/usb/logger.cpp",
            "../mikanos/kernel/pci.cpp",
            "../mikanos/kernel/libcxx_support.cpp",
            "../mikanos/kernel/usb/memory.cpp",
            "../mikanos/kernel/usb/device.cpp",
            "../mikanos/kernel/usb/xhci/ring.cpp",
            "../mikanos/kernel/usb/xhci/trb.cpp",
            "../mikanos/kernel/usb/xhci/xhci.cpp",
            "../mikanos/kernel/usb/xhci/port.cpp",
            "../mikanos/kernel/usb/xhci/device.cpp",
            "../mikanos/kernel/usb/xhci/devmgr.cpp",
            "../mikanos/kernel/usb/xhci/registers.cpp",
            "../mikanos/kernel/usb/classdriver/base.cpp",
            "../mikanos/kernel/usb/classdriver/hid.cpp",
            "../mikanos/kernel/usb/classdriver/keyboard.cpp",
            "../mikanos/kernel/usb/classdriver/mouse.cpp",
        ];

        cc::Build::new()
            .include("../mikanos/kernel")
            .include("../../../osbook/devenv/x86_64-elf/include/c++/v1")
            .include("../../../osbook/devenv/x86_64-elf/include")
            .cpp(true)
            .std("c++17")
            .flag("-O2")
            .flag("-mno-red-zone")
            .flag("-ffreestanding")
            .flag("-fno-exceptions")
            .flag("-fno-rtti")
            .flag("-fpermissive")
            .flag("-Wno-unused-parameter")
            .flag("-Wno-sign-compare")
            .define("__ELF__", None)
            .compiler("clang++")
            .no_default_flags(true)
            .cpp_link_stdlib(None)
            .cargo_warnings(false)
            .files(files.iter())
            .compile("usb");

        cc::Build::new()
            .file("../mikanos/kernel/newlib_support.c")
            .include("../../../osbook/devenv/x86_64-elf/include")
            .flag("-O2")
            .flag("-ffreestanding")
            .flag("-mno-red-zone")
            .flag("-nostdlibinc")
            .flag("-Wno-unused-parameter")
            .no_default_flags(true)
            .define("__ELF__", None)
            .cargo_warnings(false)
            .compiler("clang")
            .compile("newlib_support");

        Command::new("nasm")
            .arg("-f")
            .arg("elf64")
            .arg("-o")
            .arg(Path::new(&out_dir).join("asmfunc.o"))
            .arg("../mikanos/kernel/asmfunc.asm")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        Command::new("llvm-ar")
            .current_dir(&out_dir)
            .arg("rcs")
            .arg("libasmfunc.a")
            .arg("asmfunc.o")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
    println!("cargo:rustc-link-lib=static=asmfunc");
    println!("cargo:rerun-if-changed=asmfunc.asm");
}
