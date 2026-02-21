#!/bin/bash
# =============================================================================
# 共用下载函数 —— 支持重试和 wget/curl 自动切换
# =============================================================================

download_file() {
    local url="$1"
    local output="$2"
    local max_retries=3
    local retry_delay=5
    local ua="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36"

    echo "下载: $url"

    for i in $(seq 1 $max_retries); do
        if command -v wget &>/dev/null; then
            if wget --timeout=120 --tries=1 -U "$ua" -q --show-progress -O "$output" "$url"; then
                return 0
            fi
        fi

        if command -v curl &>/dev/null; then
            if curl -fSL --connect-timeout 120 -A "$ua" --retry 0 -o "$output" "$url"; then
                return 0
            fi
        fi

        if [ "$i" -lt "$max_retries" ]; then
            echo "下载失败 (尝试 $i/$max_retries)，${retry_delay}s 后重试..."
            rm -f "$output"
            sleep $retry_delay
            retry_delay=$((retry_delay * 2))
        fi
    done

    echo "错误：下载失败（已重试 $max_retries 次）: $url"
    rm -f "$output"
    return 1
}
