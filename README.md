# mikan-os-rs
An OS written in Rust

# How to buikd

## Clone mikanos

```
gh repo clone uchan-nos/mikanos
cd mikanos
git checkout osbook_day07b
```

## Download libraries

```
curl -LO https://github.com/uchan-nos/mikanos-build/releases/download/v2.0/x86_64-elf.tar.gz
tar xvf x86_64-elf.tar.gz
```

## Directory layouts

```
 .
├──  mikan-os-rs       << This repository
│   ├──  disk.img
│   ├──  kernel
│   ├──  LICENSE
│   ├──  loader
│   ├──  Makefile.toml
│   ├──  mnt
│   ├──  ovmf
│   └── 󰂺 README.md
└──  mikanos           << Cloned repository
    ├──  kernel
    ├──  LICENSE
    ├──  MikanLoaderPkg
    ├── 󰂺 README.md
    ├──  tools
    └──  x86_64-elf    << Downloaded libraries
```

## Launch qemu

```
cargo make launch
```
