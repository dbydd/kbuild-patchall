# cargo-byteos

> This is a cargo tool for building byteos.

## Installation

```shell
cargo install --git https://github.com/Byte-OS/cargo-byteos.git
```

## Download kernel

```shell
byteos download test.yaml
```

test.yaml is the file that contains the file tree about the module of the byteos.

## Build kernel

```shell
byteos build test.toml riscv64-qemu
```

TIPS: test.toml is the config included in the byteos.