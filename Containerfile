# 交叉编译环境镜像：Debian stretch (glibc 2.24) + GNU armhf 交叉工具链 + Rust + just
# 用途：GitHub Actions CI 构建 callmeju 的 armv7 产物（glibc 2.24 ≤ 设备的 2.26）

FROM debian:stretch

# stretch 已 EOL：apt 源切到 archive，并关闭有效期检查
RUN sed -i '/stretch-updates/d; s|deb.debian.org|archive.debian.org|g; s|security.debian.org|archive.debian.org|g' /etc/apt/sources.list \
    && echo 'Acquire::Check-Valid-Until "false";' > /etc/apt/apt.conf.d/99no-valid-until

# 多架构 armhf + 交叉工具链 + armhf libasound（alsa crate 依赖）
RUN dpkg --add-architecture armhf \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates curl pkg-config build-essential \
        crossbuild-essential-armhf libc6-dev-armhf-cross \
        libasound2-dev:armhf \
    && rm -rf /var/lib/apt/lists/*

# Rust 工具链 + armv7 目标
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal \
    && /root/.cargo/bin/rustup target add armv7-unknown-linux-gnueabihf

# just（prebuilt musl 静态二进制，免编译）
ARG JUST_VERSION=1.59.0
RUN curl -sSL -o /tmp/just.tgz \
        "https://github.com/casey/just/releases/download/${JUST_VERSION}/just-${JUST_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
    && tar -xzf /tmp/just.tgz -C /usr/local/bin just \
    && chmod +x /usr/local/bin/just \
    && rm /tmp/just.tgz

ENV PATH="/root/.cargo/bin:/usr/local/bin:${PATH}"

WORKDIR /build

CMD ["just", "release"]
