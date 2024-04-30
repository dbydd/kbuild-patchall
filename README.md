# kbuild

> This is a cargo tool for building a rust os.

## Installation

```shell
cargo install kbuild
```

## Download kernel

```shell
kbuild download test.yaml
```

test.yaml is the file that contains the file tree about the module of the kbuild.

## Build kernel

```shell
kbuild build test.toml riscv64-qemu
```

TIPS: test.toml is the config included in the kbuild.