## aarch64-unknown-linux-musl 完全静态编译说明

### 概述

本脚本自动完成以下步骤，生成完全静态链接的 AArch64 二进制文件：

1. **下载 musl 交叉编译工具链** — 从 musl.cc 下载（已有则跳过）
2. **下载并编译 C 依赖库** — alsa-lib、opus、speexdsp 编译为静态 `.a` 文件
3. **配置 Rust 交叉编译环境** — 设置 CC、pkg-config、静态链接标志
4. **编译 Rust 项目** — 输出完全静态链接的二进制文件

### 前置条件

- **Rust nightly 工具链** — `rustup toolchain install nightly`
- **构建工具** — `wget`、`make`、`tar`

### 使用方法

```bash
# 执行编译（工具链会自动下载）
bash scripts/aarch64-unknown-linux-musl/build.sh

# 输出文件
# target/aarch64-unknown-linux-musl/release/xiaozhi_linux_rs
```

### 缓存机制

所有下载和编译产物缓存在 `third_party/aarch64-unknown-linux-musl/` 目录下：
- `arm-linux-musl-cross/` — musl 交叉编译工具链
- `sysroot/` — 编译出的静态库
- `build/` — C 库源码和构建中间文件

如需重新编译：
```bash
rm -rf third_party/aarch64-unknown-linux-musl
```

### 已知限制

> [!WARNING]
> **ALSA 静态链接限制**：完全静态链接的 ALSA 只能直接访问硬件设备（如 `hw:0,0`），无法使用 `dmix`、`PulseAudio` 等 ALSA 插件共享声卡。
