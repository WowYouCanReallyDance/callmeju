# callmeju 任务清单
# CI 在构建容器内执行 `just release` 完成 armv7 交叉编译

target := "armv7-unknown-linux-gnueabihf"

default: release

# 快速编译校验（宿主机目标）
check:
    cargo check

# 静态检查：clippy + 格式
lint:
    cargo clippy --all-targets -- -D warnings
    cargo fmt --check

fmt:
    cargo fmt

# 单元测试（宿主机目标）
test:
    cargo test

# 交叉编译 release 产物（默认）
release: cross

cross:
    PKG_CONFIG_ALLOW_CROSS=1 PKG_CONFIG_LIBDIR=/usr/lib/arm-linux-gnueabihf/pkgconfig cargo build --release --target {{target}}

clean:
    cargo clean
