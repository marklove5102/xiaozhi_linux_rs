#!/bin/bash
set -e

# 加载共用下载函数（支持重试 + wget/curl 自动切换）
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/../download_helper.sh"

# =============================================================================
# armv7-unknown-linux-gnueabihf 混合链接编译脚本
#
# 本脚本会自动完成以下步骤：
#   1. 下载 GNU 交叉编译工具链（如已存在则跳过）
#   2. 下载并交叉编译 alsa-lib、opus、speexdsp 为静态库（.a）
#   3. 将音频库静态链接进二进制，但保持 libc (GLIBC) 和 libdl 动态链接
#
# 混合链接策略优势：
#   - 音频库静态打入，部署时无需拷贝 .so 文件
#   - 动态链接 libc，支持 dlopen 加载板子上的 ALSA 插件（如 PulseAudio）
#   - "default" 音频设备名可正常工作
#
# 无需手动安装任何工具链，适用于本地开发和 GitHub Actions CI。
# =============================================================================

# 获取脚本所在目录的绝对路径
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 跳转到项目根目录（../../）
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")/../"
cd "$PROJECT_ROOT"
PROJECT_ROOT="$(pwd)"

echo "============================================="
echo "  混合链接编译 - armv7-unknown-linux-gnueabihf"
echo "============================================="
echo "Project root: $PROJECT_ROOT"

# =============================================================================
# 1. 基础配置
# =============================================================================

TARGET="armv7-unknown-linux-gnueabihf"
CROSS_PREFIX="arm-linux-gnueabihf"

# 所有第三方内容统一放在 third_party/<target> 下，避免多目标冲突
THIRD_PARTY="$PROJECT_ROOT/third_party"
TARGET_DIR="$THIRD_PARTY/$TARGET"
mkdir -p "$TARGET_DIR"

# --- 1A. 下载 GNU 交叉编译工具链 ---
TOOLCHAIN_NAME="gcc-arm-8.3-2019.02-x86_64-arm-linux-gnueabihf"
TOOLCHAIN_DIR="$TARGET_DIR/$TOOLCHAIN_NAME"

if [ -x "$TOOLCHAIN_DIR/bin/${CROSS_PREFIX}-gcc" ]; then
    echo "GNU 工具链已存在，跳过下载。"
else
    echo "=== 下载 GNU 交叉编译工具链 ==="
    TOOLCHAIN_TARBALL="${TOOLCHAIN_NAME}.tar.xz"
    TOOLCHAIN_URL="https://github.com/Hyrsoft/xiaozhi_linux_rs/releases/download/Source_Mirror/${TOOLCHAIN_TARBALL}"

    echo "下载: $TOOLCHAIN_URL"
    download_file "$TOOLCHAIN_URL" "$TARGET_DIR/${TOOLCHAIN_TARBALL}"
    echo "解压工具链..."
    tar -xJf "$TARGET_DIR/${TOOLCHAIN_TARBALL}" -C "$TARGET_DIR"
    rm -f "$TARGET_DIR/${TOOLCHAIN_TARBALL}"
    echo "工具链安装完成: $TOOLCHAIN_DIR"
fi

# 设置交叉编译工具路径
CROSS_GCC="$TOOLCHAIN_DIR/bin/${CROSS_PREFIX}-gcc"
CROSS_CXX="$TOOLCHAIN_DIR/bin/${CROSS_PREFIX}-g++"
CROSS_AR="$TOOLCHAIN_DIR/bin/${CROSS_PREFIX}-ar"
CROSS_RANLIB="$TOOLCHAIN_DIR/bin/${CROSS_PREFIX}-ranlib"
CROSS_STRIP="$TOOLCHAIN_DIR/bin/${CROSS_PREFIX}-strip"

echo "CC: $CROSS_GCC"
echo "GCC version: $($CROSS_GCC --version | head -1)"

# 静态库输出目录
STATIC_SYSROOT="$TARGET_DIR/sysroot"
STATIC_LIBDIR="$STATIC_SYSROOT/usr/lib"
STATIC_INCDIR="$STATIC_SYSROOT/usr/include"

# 源码下载与构建目录
BUILD_DIR="$TARGET_DIR/build"

# C 依赖库版本
ALSA_VERSION="1.2.12"
OPUS_VERSION="1.5.2"
SPEEXDSP_VERSION="1.2.1"

# 并行编译线程数
NPROC=$(nproc 2>/dev/null || echo 4)

# =============================================================================
# 2. 下载并编译 C 依赖库（静态 .a）
# =============================================================================

mkdir -p "$STATIC_SYSROOT" "$STATIC_LIBDIR" "$STATIC_INCDIR" "$BUILD_DIR"

# 通用交叉编译环境变量
export CC="$CROSS_GCC"
export CXX="$CROSS_CXX"
export AR="$CROSS_AR"
export RANLIB="$CROSS_RANLIB"
export STRIP="$CROSS_STRIP"
# GNU 目标编译为 PIE，静态库必须使用 -fPIC 才能链接进 PIE 二进制
export CFLAGS="-fPIC"
export CXXFLAGS="-fPIC"

# --- 2A. 编译 alsa-lib ---
echo ""
echo "=== Step 2A: 编译 alsa-lib ${ALSA_VERSION} (静态) ==="

ALSA_SRC_DIR="$BUILD_DIR/alsa-lib-${ALSA_VERSION}"
if [ -f "$STATIC_LIBDIR/libasound.a" ]; then
    echo "alsa-lib 静态库已存在，跳过编译。"
else
    ALSA_TARBALL="alsa-lib-${ALSA_VERSION}.tar.bz2"
    ALSA_URL="https://github.com/Hyrsoft/xiaozhi_linux_rs/releases/download/Source_Mirror/${ALSA_TARBALL}"

    if [ ! -d "$ALSA_SRC_DIR" ]; then
        echo "下载 alsa-lib..."
        download_file "$ALSA_URL" "$BUILD_DIR/${ALSA_TARBALL}"
        echo "解压 alsa-lib..."
        tar -xjf "$BUILD_DIR/${ALSA_TARBALL}" -C "$BUILD_DIR"
        rm -f "$BUILD_DIR/${ALSA_TARBALL}"
    fi

    cd "$ALSA_SRC_DIR"
    echo "配置 alsa-lib..."
    ./configure \
        --host="${CROSS_PREFIX}" \
        --enable-static \
        --disable-shared \
        --disable-python \
        --disable-alisp \
        --disable-old-symbols \
        --with-configdir="/usr/share/alsa" \
        --with-plugindir="/usr/lib/alsa-lib" \
        --prefix="/usr" \
        --quiet

    echo "编译 alsa-lib (使用 ${NPROC} 线程)..."
    make -j"$NPROC" --quiet
    make DESTDIR="$STATIC_SYSROOT" install --quiet
    echo "alsa-lib 编译完成!"
fi

# --- 2B. 编译 Opus ---
echo ""
echo "=== Step 2B: 编译 opus ${OPUS_VERSION} (静态) ==="

OPUS_SRC_DIR="$BUILD_DIR/opus-${OPUS_VERSION}"
if [ -f "$STATIC_LIBDIR/libopus.a" ]; then
    echo "opus 静态库已存在，跳过编译。"
else
    OPUS_TARBALL="opus-${OPUS_VERSION}.tar.gz"
    OPUS_URL="https://github.com/Hyrsoft/xiaozhi_linux_rs/releases/download/Source_Mirror/${OPUS_TARBALL}"

    if [ ! -d "$OPUS_SRC_DIR" ]; then
        echo "下载 opus..."
        download_file "$OPUS_URL" "$BUILD_DIR/${OPUS_TARBALL}"
        echo "解压 opus..."
        tar -xzf "$BUILD_DIR/${OPUS_TARBALL}" -C "$BUILD_DIR"
        rm -f "$BUILD_DIR/${OPUS_TARBALL}"
    fi

    cd "$OPUS_SRC_DIR"
    echo "配置 opus..."
    ./configure \
        --host="${CROSS_PREFIX}" \
        --enable-static \
        --disable-shared \
        --disable-doc \
        --disable-extra-programs \
        --prefix="/usr" \
        --quiet

    echo "编译 opus (使用 ${NPROC} 线程)..."
    make -j"$NPROC" --quiet
    make DESTDIR="$STATIC_SYSROOT" install --quiet
    echo "opus 编译完成!"
fi

# --- 2C. 编译 SpeexDSP ---
echo ""
echo "=== Step 2C: 编译 speexdsp ${SPEEXDSP_VERSION} (静态) ==="

SPEEXDSP_SRC_DIR="$BUILD_DIR/speexdsp-${SPEEXDSP_VERSION}"
if [ -f "$STATIC_LIBDIR/libspeexdsp.a" ]; then
    echo "speexdsp 静态库已存在，跳过编译。"
else
    SPEEXDSP_TARBALL="speexdsp-${SPEEXDSP_VERSION}.tar.gz"
    SPEEXDSP_URL="https://github.com/Hyrsoft/xiaozhi_linux_rs/releases/download/Source_Mirror/${SPEEXDSP_TARBALL}"

    if [ ! -d "$SPEEXDSP_SRC_DIR" ]; then
        echo "下载 speexdsp..."
        download_file "$SPEEXDSP_URL" "$BUILD_DIR/${SPEEXDSP_TARBALL}"
        echo "解压 speexdsp..."
        tar -xzf "$BUILD_DIR/${SPEEXDSP_TARBALL}" -C "$BUILD_DIR"
        rm -f "$BUILD_DIR/${SPEEXDSP_TARBALL}"
    fi

    cd "$SPEEXDSP_SRC_DIR"
    echo "配置 speexdsp..."
    ./configure \
        --host="${CROSS_PREFIX}" \
        --enable-static \
        --disable-shared \
        --prefix="/usr" \
        --quiet

    echo "编译 speexdsp (使用 ${NPROC} 线程)..."
    make -j"$NPROC" --quiet
    make DESTDIR="$STATIC_SYSROOT" install --quiet
    echo "speexdsp 编译完成!"
fi

cd "$PROJECT_ROOT"

echo ""
echo "=== 所有 C 依赖库编译完成 ==="
echo "静态库目录: $STATIC_LIBDIR"
ls -la "$STATIC_LIBDIR"/*.a 2>/dev/null || echo "（无 .a 文件，请检查编译日志）"

# =============================================================================
# 3. 设置 Rust 交叉编译环境
# =============================================================================

echo ""
echo "=== Step 3: 设置 Rust 编译环境 ==="

# 安装 gnu target（如果尚未安装）
rustup target add "$TARGET" 2>/dev/null || true

# CC / CXX 环境变量（Cargo 使用下划线格式的目标三元组）
export CC_armv7_unknown_linux_gnueabihf="$CROSS_GCC"
export CXX_armv7_unknown_linux_gnueabihf="$CROSS_CXX"
export AR_armv7_unknown_linux_gnueabihf="$CROSS_AR"

# Cargo linker
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER="$CROSS_GCC"

# 告诉 Rust cc crate 编译 C 源码时使用 -fPIC（PIE 二进制必需）
export CFLAGS_armv7_unknown_linux_gnueabihf="-fPIC"

# 混合链接：不使用 +crt-static，保持 libc/libdl 动态链接
# 通过 -L 指向静态库目录，并显式链接 dl/pthread/m
export RUSTFLAGS="-C link-arg=-L$STATIC_LIBDIR -C link-arg=-ldl -C link-arg=-lpthread -C link-arg=-lm"

# 告诉 audiopus_sys 使用静态链接 opus
export LIBOPUS_STATIC=1

# 告诉 alsa-sys 使用静态链接 alsa
export ALSA_STATIC=1

# pkg-config 配置 —— 指向我们编译出的静态库
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH=""
export PKG_CONFIG_LIBDIR="$STATIC_LIBDIR/pkgconfig"
export PKG_CONFIG_SYSROOT_DIR="$STATIC_SYSROOT"
# 强制 pkg-config 只报告静态链接标志
export PKG_CONFIG_ALL_STATIC=1

# 传递静态 sysroot 路径给 audio/build.rs
export STATIC_AUDIO_SYSROOT="$STATIC_SYSROOT"

echo "CC:           $CROSS_GCC"
echo "STATIC_LIBS:  $STATIC_SYSROOT"
echo "RUSTFLAGS:    $RUSTFLAGS"

# =============================================================================
# 4. 编译 Rust 项目
# =============================================================================

echo ""
echo "=== Step 4: 编译 Rust 项目 (混合链接: 音频库静态 + libc 动态) ==="
echo "Building in: $PROJECT_ROOT"
echo "Target: $TARGET"

cargo build \
    --target "$TARGET" \
    --release

echo ""
echo "============================================="
echo "  编译完成!"
echo "============================================="

OUTPUT_BIN="$PROJECT_ROOT/target/$TARGET/release/xiaozhi_linux_rs"
if [ -f "$OUTPUT_BIN" ]; then
    echo "输出文件: $OUTPUT_BIN"
    echo "文件大小: $(du -h "$OUTPUT_BIN" | cut -f1)"
    echo ""
    echo "文件信息:"
    file "$OUTPUT_BIN"
    echo ""
    echo "提示: 可使用以下命令验证链接方式:"
    echo "  file $OUTPUT_BIN"
    echo "  （应显示 'dynamically linked'，表示 libc 动态链接）"
    echo ""
    echo "  readelf -d $OUTPUT_BIN | grep NEEDED"
    echo "  （应仅显示 libc/libdl/libpthread 等系统库，不应出现 libasound/libopus/libspeexdsp）"
else
    echo "警告: 未找到输出文件 $OUTPUT_BIN"
    echo "请检查编译日志。"
fi
