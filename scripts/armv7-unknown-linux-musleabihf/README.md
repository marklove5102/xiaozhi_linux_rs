## armv7-unknown-linux-musleabihf 完全静态编译说明

### 概述

本脚本自动完成以下步骤，生成完全静态链接的 ARM 二进制文件：

1. **下载并编译 C 依赖库** — alsa-lib、opus、speexdsp 编译为静态 `.a` 文件
2. **配置 Rust 交叉编译环境** — 设置 CC、pkg-config、静态链接标志
3. **编译 Rust 项目** — 使用 `cargo +nightly build` 输出静态二进制

### 前置条件

- **musl 交叉编译器** — 需要 `arm-linux-musleabihf-gcc` 工具链
  - 预编译下载地址：https://musl.cc/
  - 或通过 buildroot 自行构建
- **Rust nightly 工具链** — `rustup toolchain install nightly`
- **rust-src 组件** — `rustup component add rust-src --toolchain nightly`
- **构建工具** — `wget`、`make`、`tar`

### 使用方法

```bash
# 1. 设置 musl 工具链路径（如果不使用默认路径）
export MUSL_TOOLCHAIN_PATH="/path/to/arm-linux-musleabihf-cross"

# 2. 执行编译
bash scripts/armv7-unknown-linux-musleabihf/build.sh

# 3. 输出文件
# target/armv7-unknown-linux-musleabihf/release/xiaozhi_linux_rs
```

### 工具链配置

默认路径为 `/opt/arm-linux-musleabihf-cross`，可通过环境变量覆盖：

```bash
export MUSL_TOOLCHAIN_PATH="/home/user/toolchains/arm-linux-musleabihf-cross"
```

工具链目录结构应包含：
```
arm-linux-musleabihf-cross/
└── bin/
    ├── arm-linux-musleabihf-gcc
    ├── arm-linux-musleabihf-g++
    ├── arm-linux-musleabihf-ar
    ├── arm-linux-musleabihf-ranlib
    └── arm-linux-musleabihf-strip
```

### 缓存机制

C 依赖库编译产物缓存在 `third_party/musleabihf/` 目录下。若已存在 `.a` 文件则自动跳过编译。如需重新编译：

```bash
rm -rf third_party/musleabihf third_party/build_musleabihf
```

### 已知限制

> [!WARNING]
> **ALSA 静态链接限制**：完全静态链接的 ALSA 只能直接访问硬件设备（如 `hw:0,0`），无法使用 `dmix`、`PulseAudio` 等 ALSA 插件共享声卡。适合嵌入式场景下独占音频设备的使用方式。

- 二进制体积会显著增加（几 MB 级别），但部署时无需在目标设备准备 `.so` 文件
- 需要 nightly Rust 工具链（因使用 `-Z build-std`）
