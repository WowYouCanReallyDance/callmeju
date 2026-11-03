# callmeju

小聚 mini（RK3308）改造为大模型语音终端的**瘦客户端**。设备侧只负责「唤醒 → 拾音 → 上报 → 播放」，ASR / LLM / TTS 全部在局域网主机或云端完成。

```
RK3308 (callmeju)                      后端 (局域网/云)
  硬件VAD(event1) 唤醒
    → ALSA 录音  ────HTTP POST PCM────►  ASR → LLM → TTS
    ← ALSA 播放  ◄────JSON(文本+音频)────
```

## 架构

单 crate（lib + bin 拆分），模块划分：

| 模块 | 职责 |
|------|------|
| `main.rs` | 入口：tracing 初始化、配置加载、`tokio::select!` 优雅退出 |
| `config.rs` | 配置（`clap` CLI + `toml` 文件，`serde(deny_unknown_fields)` 防拼写错误）|
| `audio/` | `Recorder` / `Player` trait；`alsa`（Linux）与 `mock`（开发用）两套实现 |
| `vad.rs` | 硬件 VAD（`evdev`，Linux）与 stdin mock（开发用）|
| `net.rs` | 后端客户端（`reqwest`），POST 原始 PCM、解析 JSON 回复 |
| `session.rs` | 编排状态机：唤醒 → 录音 → 对话 → 播放 |

关键设计点：

- **阻塞 I/O 放 `spawn_blocking`**：ALSA 读/写与 evdev 等待都是阻塞的，统一丢到 `tokio::task::spawn_blocking`，不阻塞异步运行时。
- **后端可插拔**：`BackendClient` 只定义一条 wire 契约（`POST /v1/converse`，raw PCM 进、JSON 出），后续换 WebSocket 流式只需新增 transport，不动 session 逻辑。
- **音频后端按 `cfg(target_os)` 选择**：Linux 用 ALSA（直连 `pcmC1D0c/p`，可控），macOS/其它平台用 mock，保证本机 `cargo check` 也能过。
- **体积优化**：`opt-level = "z"` + `lto` + `strip` + `panic = "abort"`，面向 46MB 存储 / 84MB 内存的设备。

## 依赖清单（均已核实 crates.io 活跃度）

| crate | 版本 | 用途 | 说明 |
|-------|------|------|------|
| `tokio` | 1 | 异步运行时 | 事实标准 |
| `reqwest` | 0.12 | HTTP 客户端（rustls，无 OpenSSL）| 最流行，`default-features=false` 精简 |
| `serde` / `serde_json` | 1 | 序列化 | 事实标准 |
| `toml` | 0.8 | 配置文件解析 | 标准 |
| `clap` | 4 | CLI 参数 | 最流行 |
| `tracing` / `tracing-subscriber` | 0.1 / 0.3 | 结构化日志 | 现代标准 |
| `anyhow` | 1 | 应用错误处理 | 标准 |
| `bytes` | 1 | 字节流缓冲 | 标准 |
| `futures-util` | 0.3 | Stream 组合器 | 标准 |
| `base64` | 0.22 | 音频 base64 编解码 | 标准 |
| `alsa` *(linux)* | 0.12 | 音频 I/O | 27.7M 下载，2026-07 更新 |
| `evdev` *(linux)* | 0.13 | 硬件 VAD | 2.26M 下载，v0.13.2 稳定 |

备选（未启用）：`cpal`（27M 下载，若需要跨平台音频可替换 `alsa`）。

## 非 Rust 依赖（需讨论后再定，见下）

1. **libasound**：`alsa` crate 的编译期系统库。设备上已有（`arecord`/`aplay` 依赖它），交叉编译时需 armhf 版。
2. **交叉工具链**：`armv7-unknown-linux-gnueabihf` target + 链接器 + glibc 2.26 sysroot。
3. **Porcupine**（可选，自定义唤醒词）：闭源 C 库 + 模型文件，需 Picovoice key。当前方案用硬件 VAD 即可，暂不需要。

## 构建

### 本机快速校验（macOS，mock 后端）
```sh
cargo check
cargo run -- --server http://127.0.0.1:8000
```

### 设备交叉编译（GitHub Actions CI，容器化 + just，无需 zig）

所有构建都在 CI 完成，无需本地工具链：

- **push 到 `main` 或 PR**：自动交叉编译，产物在 Actions 的 Artifacts 里下载（`callmeju-armv7-linux-gnueabihf`）。
- **打 `v*.*.*-*` 形式的 tag**：自动发布到 Releases（附 `callmeju` 二进制资产）。

流程由 `.github/workflows/build.yml` 定义：`docker build` 出 `Containerfile` 环境镜像 → 容器内 `just release` 交叉编译 → 上传产物。

> 任务清单见 `Justfile`；构建镜像见 `Containerfile`；`.cargo/config.toml` 含交叉链接器（`arm-linux-gnueabihf-gcc`）。
> **glibc 说明**：设备是 glibc 2.26，必须用 ≤ 2.26 的 glibc 链接，故 base 选 Debian stretch（glibc 2.24）。


## 待办

- [ ] 实机探明 VAD 具体 key code，替换 `wait_wake` 的「任意按键」匹配
- [ ] 用硬件 VAD 的 end-of-speech 信号替代固定 `max_utterance_secs` 录音上限
- [ ] 后端（ASR→LLM→TTS）服务实现，或接入现有网关
- [ ] 流式响应（降低首字延迟）与 AEC 回声消除方案评估
