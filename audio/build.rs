fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();

    if target.contains("musl") {
        // musl 目标：使用手动编译的静态库，不依赖 pkg-config
        if let Ok(sysroot) = std::env::var("MUSL_SYSROOT") {
            println!("cargo:rustc-link-search=native={}/usr/lib", sysroot);
        }
        println!("cargo:rustc-link-lib=static=speexdsp");
        return;
    }

    // GNU 目标：ALSA 动态链接（由 alsa-sys 自动处理），speexdsp 通过 pkg-config 查找
    // 直接 fall through 到下方 pkg-config 分支

    // 其他目标：通过 pkg-config 查找 libspeexdsp
    pkg_config::Config::new()
        .probe("speexdsp")
        .expect("Failed to find speexdsp. Please install libspeexdsp-dev.");
}
