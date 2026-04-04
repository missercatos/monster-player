# 项目名称
TMPlayer（TUI Music Player）

# 项目仓库地址
https://github.com/professor-lee/TMPlayer

# 项目简介
TMPlayer 是一个面向 Linux 终端场景的 Rust TUI 音乐播放器，核心目标是在轻量命令行环境中提供“本地播放 + 系统播放监听 + 可视化渲染”的一体化体验。项目支持本地音频文件播放、MPRIS 系统播放器状态同步、歌词与封面展示，并提供可交互设置面板与播放列表操作。与传统 GUI 播放器相比，它强调低依赖、键盘优先和终端审美，适用于开发者工作流或服务器/远程环境下的音乐播放需求。项目还提供可选的 `bundle-cava` 构建特性，用于在构建期打包 `cava` 以增强频谱能力，整体定位清晰，工程化结构完整。

# 项目技术栈
- 语言与构建
  - Rust 2021（`Cargo.toml`）
  - Cargo 构建脚本：`build.rs`
- 终端 UI
  - `ratatui = 0.26`
  - `crossterm = 0.27`
- 音频播放与解码
  - `rodio = 0.17`（播放）
  - `symphonia = 0.5`（解码；启用 `aac/alac/flac/isomp4/mp3/ogg/wav/vorbis`）
  - `cpal = 0.15`
- 系统媒体控制与平台依赖
  - Linux 条件依赖：`mpris = 2.0`、`alsa = 0.9`
- 元数据与媒体处理
  - `lofty = 0.18`（标签与元数据）
  - `image = 0.24`（封面处理）
  - `chromaprint = 0.2`（音频指纹）
- 网络与序列化
  - `ureq = 2`（HTTP，含 json feature）
  - `serde = 1`、`toml = 0.8`
- 工具与基础库
  - `anyhow = 1`、`thiserror = 2`
  - `env_logger = 0.11`、`log = 0.4`
  - `directories = 6.0.0`、`tempfile = 3`
- 打包与发布相关
  - AUR 打包脚本：`AUR/PKGBUILD`
  - 可选 `cava` 源码打包能力（`bundle-cava` feature + `build.rs`）

# 项目功能
- 双模式播放与监听
  - 本地播放模式（文件夹扫描、播放列表、上一首/下一首、播放模式切换）
  - 系统监听模式（基于 MPRIS 轮询系统播放器状态）
- 音频播放能力
  - 支持多种主流音频格式（通过 Symphonia 解码能力）
  - 播放/暂停、进度跳转、音量调节
  - 本地播放 10 段均衡器（EQ）
- 可视化与渲染
  - Bars 频谱渲染（可调柱数、间距、通道方向、平滑）
  - Oscilloscope（示波）渲染
  - 终端内封面渲染：ASCII（默认）与 Kitty Graphics（可选）
- 元数据、歌词与封面
  - 本地优先：嵌入标签、本地 LRC、本地封面
  - 远程补全：LRCLIB（歌词）、MusicBrainz + Cover Art Archive（封面）
  - 元数据缺失时可选 Chromaprint + AcoustID 补全
- 交互与体验
  - 键盘快捷键体系（如 `Ctrl+F`、`Ctrl+K`、`P`、`T`、`E`）
  - 播放列表侧栏、设置弹窗、帮助弹窗、Toast 提示
- 配置与持久化
  - TOML 配置与主题文件自动初始化
  - 记录播放顺序、上次歌曲、上次位置、封面 ASCII 缓存

# 项目管线
- 代码构建流程
  - 开发运行：`cargo run`
  - 发布构建：`cargo build --release`
  - 可选能力：`cargo build --release --features bundle-cava`
- `bundle-cava` 构建子流程（`build.rs`）
  - 条件触发：仅在启用 `bundle-cava` 时执行
  - 主要步骤：下载 `cava` 源码 tarball -> 解压 -> autotools 构建 -> 复制到目标目录 -> 写入 `OUT_DIR/cava.bin`
  - 兼容处理：注入最小 `AX_CHECK_GL` 宏覆盖，减少 autotools 差异导致的失败风险
- 打包流程（AUR）
  - `prepare`：`cargo fetch --locked`
  - `build`：`cargo build --frozen --release`
  - `package`：安装二进制到 `/usr/bin/tmplayer`，并安装默认配置与主题到 `/usr/share/tmplayer`
- 测试流程
  - 根据提供信息暂未找到具体细节：仓库中未检索到自动化测试代码（如 `#[test]`）与 `cargo test` 流程说明
- CI/CD 流程
  - 根据提供信息暂未找到具体细节：未发现 `.github/workflows`、`Jenkinsfile`、`Dockerfile` 等持续集成/持续部署配置
- 部署流程
  - 当前可确认方式为本地构建运行与 AUR 打包分发；自动化发布流水线根据提供信息暂未找到具体细节

# 项目功能具体实现
- 应用入口与主循环
  - `main.rs` 初始化日志、安装 ALSA stderr 过滤器、加载配置与主题，然后进入 `app::event_loop::run`
  - 事件循环统一处理：输入事件 -> MPRIS 轮询 -> 可视化更新 -> 本地播放状态同步 -> UI 绘制 -> 帧率控制
- 模式管理
  - `ModeManager` 聚合 `LocalPlayer` 与 `MprisClient`
  - 切换到本地模式时暂停系统播放器；切换到系统模式时暂停本地播放，保证互斥
- 本地播放实现
  - `LocalPlayer` 使用 `rodio::Sink` 负责播放
  - 通过 `SymphoniaSource` 做音频解码，支持 seek
  - `EqSource` + 原子参数实现 EQ 运行时更新，避免重建播放器造成明显卡顿
  - `TapSource` 将采样写入无锁环形缓冲，用于可视化数据读取
- MPRIS 监听与控制
  - Linux 下使用 `mpris::PlayerFinder` 轮询活动播放器
  - 同步元数据（标题/艺术家/专辑/封面）、位置、音量、播放状态
  - 支持 `play_pause/next/previous/seek/set_volume_delta`
- 可视化与渲染
  - Bars：读取 `cava` 输出并经 EMA 平滑后绘制
  - Oscilloscope：将立体声数据映射为 Braille 点阵波形并做垂直渐变着色
  - 封面：优先 ASCII 渲染；终端支持且启用时走 Kitty Graphics 路径（含异步编码与重放置）
- 远程元数据补全
  - 后台 worker 线程（含防抖与节流）处理歌词/封面/指纹请求
  - 数据流：TrackKey 判重 -> 远程查询 -> 结果回传 -> 应用到当前曲目与本地缓存
- 本地数据持久化
  - `config/default.toml`：全局配置
  - `.order.toml`：播放顺序、最后打开歌曲、最后专辑、上次播放位置、封面 ASCII 缓存

```rust
// 主循环伪代码（基于 event_loop.rs）
loop {
  处理键盘/鼠标输入();
  应用远程抓取结果();

  if 到达_mpris_poll_ms() {
    snapshot = 轮询_mpris();
    同步模式与播放器状态(snapshot);
  }

  if 到达_spectrum_hz() {
    if visualize == Bars {
      bars = 从_cava_读取();
      bars = EMA_平滑(bars);
    } else {
      stereo = 从_cava_读取立体声();
      生成示波相位并更新波形();
    }
  }

  if mode == LocalPlayback {
    同步本地播放位置与结束状态();
  }

  绘制_TUI();
  sleep_到下一帧(ui_fps);
}
```

补充说明：
- README 声明“无 `cava` 时回退内部 FFT”，但当前代码中 `audio/fft.rs` 与 `audio/spectrum.rs` 已移除实现，且事件循环注释为“无 `cava` 时保持频谱为空”。
- `render/lissajous_renderer.rs` 文件存在，但根据当前模块导出关系未看到实际接入。

# 项目可配置项
- 配置文件
  - 主要文件：`config/default.toml`
  - 首次运行会自动在系统配置目录（如 Linux 的 `~/.config/tmplayer`）生成配置与主题文件

| 配置项 | 默认值（默认配置文件） | 说明 |
|---|---|---|
| theme | frappe | 主题名（system/latte/frappe/macchiato/mocha） |
| ui_fps | 60 | UI 帧率 |
| spectrum_hz | 60 | 频谱更新频率 |
| mpris_poll_ms | 100 | MPRIS 轮询间隔（毫秒） |
| visualize | bars | 可视化模式（bars/oscilloscope） |
| transparent_background | true | 是否透明背景 |
| album_border | false | 专辑区域边框开关 |
| kitty_graphics | false | 是否启用 Kitty 图形协议渲染 |
| kitty_cover_scale_percent | 100 | Kitty 封面缩放质量百分比 |
| super_smooth_bar | false | 柱形更细粒度高度显示 |
| bars_gap | false | 柱间距开关 |
| bar_number | auto | 柱数量策略（auto/16/32/48/64/80/96） |
| bar_channels | mono | 柱通道布局（mono/stereo） |
| bar_channel_reverse | false | 柱通道方向反转 |
| lyrics_cover_fetch | false | 是否启用远程歌词/封面抓取 |
| lyrics_cover_download | false | 是否下载抓取到的歌词/封面 |
| audio_fingerprint | false | 是否启用音频指纹（依赖 AcoustID） |
| acoustid_api_key | 空字符串 | AcoustID API Key |
| resume_last_position | false | 是否续播上次本地进度 |
| default-opening-folder | 空字符串 | 启动自动打开本地目录 |

- 代码默认值（`Config::default`）
  - 根据提供信息，`Config::default` 与 `config/default.toml` 的部分初始值存在差异（如 `theme`、`transparent_background`、`album_border`）；运行时通常以实际加载到的配置文件为准
- 环境变量
  - `TMPLAYER_ASSET_DIR`：覆盖资源根目录
  - `TMPLAYER_CAVA`：指定 `cava` 可执行文件路径
  - `TMPLAYER_CAVA_BUNDLE_VERSION`：构建期指定 `bundle-cava` 版本
  - `TMPLAYER_CAVA_BUNDLE_URL`：构建期指定 `bundle-cava` 下载地址
  - `TMPLAYER_CAVA_BUNDLE_SKIP`：构建期跳过 `bundle-cava`
  - `COLORTERM`、`TERM`：用于颜色能力探测
  - `PATH`：用于查找 `cava`
- 命令行参数
  - 根据提供信息暂未找到具体细节：未看到基于 `clap` 或 `std::env::args` 的 CLI 参数解析实现

# 项目补充与优化建议
- 项目优势与亮点
  - Rust + TUI 的模块化结构较清晰，播放、渲染、数据、状态分层明确
  - 同时覆盖本地播放与系统监听，满足终端场景下的核心使用路径
  - 配置和主题可落地到文件系统，用户定制与迁移体验较好
  - 远程元数据补全采用后台 worker，具备防抖与节流机制
- 潜在风险或局限性
  - 无自动化测试与 CI/CD 配置，回归风险与发布稳定性依赖人工流程
  - README 中关于“内部 FFT 回退”与当前实现存在不一致，可能导致用户预期偏差
  - 平台能力以 Linux 为主，跨平台特性在当前信息下仍有限
  - `Cargo.toml` 版本与部分发布目录版本表现不完全一致，需统一版本治理策略
- 推荐最佳实践与优化方向
  - 增加最小可用测试集：配置加载、播放列表逻辑、远程抓取流程与关键渲染单元
  - 建立 CI：至少执行 `cargo fmt --check`、`cargo clippy`、`cargo test`、`cargo build --release`
  - 校准文档与实现一致性，尤其是 `cava` 不可用时的行为描述
  - 在可视化层增加降级策略（无 `cava` 时启用内部算法或静态占位提示）
  - 为网络抓取增加可观测性（日志级别、失败原因统计、可选重试策略）
- 其他补充信息
  - 安全与依赖管理：建议固定关键依赖版本并定期执行审计（如 `cargo audit`）
  - 性能方向：可将渲染与数据采集性能指标（帧耗时、抓取耗时）纳入诊断开关
  - 发行方向：可补充容器化构建或更多发行渠道脚本，提高可复现构建能力
