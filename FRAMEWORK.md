# monster-player — Architecture Documentation

**monster-player** — 塞壬唱片 (Monster Siren) 音乐播放器，支持 TUI (ratatui) 和 GUI (egui) 双前端，内核共享。

---

## Architecture Diagram

```
┌──────────────────────────────────────────────────────────┐
│                      main.rs                             │
│           ┌─────────────┬─────────────┐                  │
│           │  #[cfg(tui)]│  #[cfg(gui)]│                  │
│           ▼             ▼             │                  │
│     tui::run()    origin_gui::run()   │                  │
│           │             │             │                  │
└───────────┼─────────────┼─────────────┘                  │
            │             │                                │
   ┌────────▼──────┐  ┌──▼──────────────┐                  │
   │  tui/         │  │  origin_gui/    │                  │
   │  ├─ mod.rs    │  │  ├─ mod.rs      │                  │
   │  ├─ app.rs    │  │  ├─ app.rs      │  ← 前端层        │
   │  ├─ ui.rs     │  │  ├─ ui.rs       │                  │
   │  └─ event.rs  │  │  └─ ...         │                  │
   └───────┬───────┘  └──┬──────────────┘                  │
           │              │                                │
           └──────┬───────┘                                │
                  │                                        │
    ┌─────────────▼──────────────┐                         │
    │       lib.rs (公共 API)    │                         │
    │  ┌───────────────────────┐ │                         │
    │  │ kernel.rs ← Engine    │ │  ← 业务核心             │
    │  │ player.rs ← Player    │ │  ← 音频渲染             │
    │  │ ascii_art.rs          │ │  ← 封面字符画           │
    │  │ error.rs              │ │  ← 错误类型             │
    │  └───────────┬───────────┘ │                         │
    │              │             │                         │
    │  ┌───────────▼───────────┐ │                         │
    │  │ api/                  │ │                         │
    │  │ ├─ client.rs          │ │  ← HTTP 客户端          │
    │  │ └─ types.rs           │ │  ← 数据类型定义         │
    │  └───────────────────────┘ │                         │
    └────────────────────────────┘                         │
                  │                                        │
                  ▼                                        │
     monster-siren.hypergryph.com (外部 API)               │
```

### Data Flow

```
用户输入 (键盘/鼠标)
    │
    ▼
TUI: event.rs → app.rs ─┐
GUI: ui.rs    → app.rs ─┤
                        │
                   kernel.rs (Engine)
                   │  update() 每帧轮询
                   │  │
                   │  ├──── api/client.rs ── GET /api/albums
                   │  ├──── api/client.rs ── GET /api/album/{cid}/detail
                   │  ├──── api/client.rs ── GET /api/song/{cid}
                   │  ├──── ureq::get ────── 下载 WAV 音频
                   │  └──── ureq::get ────── 下载 .lrc 歌词
                   │
                   ▼
             player.rs (rodio)
             解码 WAV → Sink 输出音频
```

---

## Source Files

### `src/main.rs`

**目的**: 程序入口，根据 feature flag 分发到 TUI 或 GUI 前端。

| 行号 | 函数 | 访问 | 类别 | 描述 |
|------|------|------|------|------|
| 9 | `main()` | — | 入口 | 初始化日志，条件编译路由到 tui::run() 或 origin_gui::run() |

---

### `src/lib.rs`

**目的**: 库根模块，声明所有公共子模块。

*(无函数 — 仅模块声明)*

---

### `src/error.rs`

**目的**: 统一错误类型与 Result 别名，供整个 crate 使用。

*(无函数 — 仅类型定义: `Error` enum, `Result<T>` type alias)*

| 类型 | 行号 | 访问 | 类别 | 描述 |
|------|------|------|------|------|
| `Error` | 5 | pub | 错误类型 | 统一错误枚举：Http/Io/Audio/Api 4 种变体 |
| `Result<T>` | 24 | pub | 错误类型 | `std::result::Result<T, Error>` 别名 |

---

### `src/player.rs`

**目的**: rodio 音频播放器封装，管理播放/暂停/进度/音量。

| 行号 | 函数 | 访问 | 类别 | 描述 |
|------|------|------|------|------|
| 26 | `new()` | pub | 初始化 | 创建音频输出流与 Sink 实例 |
| 42 | `play_url()` | pub | 播放控制 | 下载 URL 指向的 WAV，解码并送入播放队列 |
| 61 | `play_bytes()` | pub | 播放控制 | 将内存中的 WAV 字节解码并播放（缓冲播放入口） |
| 76 | `play_bytes_at()` | pub | 进度跳转 | 从指定秒数位置开始播放（seek 跳转） |
| 93 | `play_song()` | pub | 播放控制 | 记录歌曲信息后通过 URL 播放 |
| 100 | `pause()` | pub | 播放控制 | 暂停并记录已播放时间 |
| 111 | `resume()` | pub | 播放控制 | 恢复播放，重新标记起始时刻 |
| 117 | `toggle()` | pub | 播放控制 | 切换 暂停/播放 状态 |
| 127 | `stop()` | pub | 播放控制 | 停止播放并复位计时 |
| 134 | `set_volume()` | pub | 音量控制 | 设置 rodio Sink 输出音量 0.0~1.0 |
| 140 | `elapsed()` | pub | 状态查询 | 返回当前播放进度（秒），含暂停累加逻辑 |
| 151 | `duration()` | pub | 状态查询 | 返回当前曲目总时长（秒） |
| 156 | `is_paused()` | pub | 状态查询 | 返回当前是否暂停 |
| 161 | `is_empty()` | pub | 状态查询 | 返回播放队列是否为空 |
| 166 | `current_song()` | pub | 状态查询 | 返回当前播放歌曲的元信息 |
| 173 | `default()` | pub | 初始化 | Default trait 实现，调用 new() |

---

### `src/kernel.rs`

**目的**: 播放引擎核心，管理专辑/歌曲/收藏/歌词全生命周期。

**重要类型**:

| 类型 | 行号 | 访问 | 描述 |
|------|------|------|------|
| `PlayMode` | 11 | pub | 播放模式枚举：AlbumList/AlbumRandom/GlobalList/GlobalRandom/Single/LoveList/LoveRandom |
| `LovedEntry` | 22 | pub | 收藏条目：cid + name + artists |
| `Engine` | 38 | pub | 播放引擎主结构，包含播放状态/缓存/异步请求句柄 |

| 行号 | 函数 | 访问 | 类别 | 描述 |
|------|------|------|------|------|
| 81 | `new()` | pub | 初始化 | 创建配置目录，从磁盘恢复收藏数据 |
| 131 | `update()` | pub | 帧更新 | 每帧轮询异步结果 + 歌词 + 自动切歌 |
| 144 | `play_song_at()` | pub | 播放控制 | 播放列表中指定位置的歌曲 |
| 165 | `play_cid()` | private | 播放控制 | 通过 CID 播放：缓存命中直接播，否则异步获取 |
| 186 | `toggle_pause()` | pub | 播放控制 | 切换 暂停/播放 状态 |
| 194 | `next_album()` | pub | 专辑导航 | 切换到下一张专辑（循环） |
| 203 | `prev_album()` | pub | 专辑导航 | 切换到上一张专辑（循环） |
| 215 | `cycle_mode()` | pub | 模式/音量 | 循环切换播放模式 7 种 |
| 228 | `volume_up()` | pub | 模式/音量 | 音量 +5%（上限 100） |
| 234 | `volume_down()` | pub | 模式/音量 | 音量 -5%（下限 0） |
| 240 | `seek_forward()` | pub | 进度跳转 | 进度前进 5%（重新解码跳转） |
| 255 | `seek_backward()` | pub | 进度跳转 | 进度后退 5% |
| 267 | `restart_song()` | pub | 播放控制 | 从头重新播放当前歌曲 |
| 276 | `is_loved()` | pub | 收藏管理 | 检查指定 CID 是否已收藏 |
| 281 | `toggle_love()` | pub | 收藏管理 | 收藏/取消收藏，写入 loved.json |
| 299 | `rebuild_loved_list()` | pub | 收藏管理 | 从缓存和当前专辑重建收藏列表条目信息 |
| 319 | `load_loved_entries()` | private | 数据持久化 | 从 JSON 文件加载收藏数据 |
| 325 | `save_loved()` | private | 数据持久化 | 将收藏数据序列化写入 JSON 文件 |
| 332 | `apply_volume()` | private | 模式/音量 | 将 volume (0-100) 转换为 f32 并应用到 rodio |
| 339 | `fetch_albums()` | private | 数据获取 | 启动异步线程：GET /api/albums |
| 354 | `check_albums()` | private | 数据获取 | 轮询专辑列表异步结果 |
| 374 | `fetch_album_detail()` | private | 数据获取 | 启动异步线程：GET /api/album/{cid}/detail |
| 412 | `check_detail()` | private | 数据获取 | 轮询专辑详情异步结果 |
| 437 | `preload_adjacent()` | private | 缓存 | 后台预取前后各2张专辑详情 |
| 462 | `preload_song_details()` | private | 缓存 | 后台预取前3首歌详情 |
| 479 | `check_song()` | private | 数据获取 | 轮询歌曲详情异步结果 |
| 498 | `start_playback()` | private | 播放控制 | 设置歌曲信息 + 后台渐进下载 WAV（首个 chunk 到即播） |
| 561 | `check_stream()` | private | 数据获取 | 渐进下载帧检查：chunk 到达 → 播放；全曲到 → 无缝切换以支持 seek |
| 615 | `check_wav()` | private | 数据获取 | 轮询 WAV 下载结果（保留兼容，渐进模式不走此路径） |
| 660 | `advance_to_next()` | private | 播放控制 | 强制跳到下一首（下载/解码失败时回调） |
| 678 | `fetch_lyrics()` | private | 歌词处理 | 下载 .lrc 歌词并解析 |
| 699 | `update_lyric_index()` | private | 歌词处理 | 计算当前歌词行 + 进度百分比 |
| 723 | `auto_advance()` | private | 播放控制 | 歌曲结束后根据模式自动切歌 |
| 773 | `parse_lrc()` | private | 歌词处理 | 解析 LRC 格式 ([mm:ss.xx]文本 → Vec) |

---

### `src/ascii_art.rs`

**目的**: 将封面图片下载并转为灰度 ASCII 字符画（供 TUI 使用）。

| 行号 | 函数 | 访问 | 类别 | 描述 |
|------|------|------|------|------|
| 7 | `cover_to_ascii()` | pub | 布局渲染 | 下载封面图片，缩放后映射到灰度字符集 |

---

### `src/api/mod.rs`

**目的**: 声明 api 子模块（client、types）。

*(无函数 — 仅模块声明)*

---

### `src/api/types.rs`

**目的**: 定义所有 API 请求/响应的数据结构。

*(无函数 — 仅类型定义)*

| 类型 | 行号 | 访问 | 描述 |
|------|------|------|------|
| `ApiResponse<T>` | 9 | pub | 通用 API 响应包装 {code, msg, data} |
| `Album` | 22 | pub | 专辑列表项 {cid, name, coverUrl, artistes} |
| `AlbumDetail` | 40 | pub | 专辑详情（含歌曲列表）{cid, name, intro, belong, coverUrl, coverDeUrl, songs} |
| `AlbumSong` | 62 | pub | 专辑内歌曲摘要 {cid, name, artistes} |
| `Song` | 77 | pub | 歌曲列表项 {cid, name, albumCid, artists} |
| `SongDetail` | 94 | pub | 歌曲详情（含音源直链）{cid, name, albumCid, sourceUrl, lyricUrl, mvUrl, mvCoverUrl, artists} |
| `SongsResponse` | 118 | pub | 歌曲列表响应体 {list: Vec\<Song\>} |
| `NewsItem` | 128 | pub | 新闻动态条目 {cid, title, cate, date} |
| `NewsResponse` | 145 | pub | 新闻列表响应体 {list, end} |
| `SearchList<T>` | 158 | pub | 搜索结果分页列表 {list, end} |
| `SearchResponse` | 171 | pub | 搜索响应体 {albums: SearchList\<Album\>, news: SearchList\<NewsItem\>} |

---

### `src/api/client.rs`

**目的**: HTTP 客户端，封装对塞壬唱片 API 的所有请求。

| 行号 | 函数 | 访问 | 类别 | 描述 |
|------|------|------|------|------|
| 19 | `new()` | pub | 初始化 | 创建 HTTP 客户端，配置 base URL 与 ureq agent |
| 29 | `albums()` | pub | 数据获取 | GET /api/albums 获取全量专辑列表 |
| 38 | `album_detail()` | pub | 数据获取 | GET /api/album/{cid}/detail 获取专辑详情 |
| 51 | `songs()` | pub | 数据获取 | GET /api/songs 获取全量歌曲列表 |
| 61 | `song_detail()` | pub | 数据获取 | GET /api/song/{cid} 获取歌曲详情（含 WAV 直链） |
| 70 | `news()` | pub | 数据获取 | GET /api/news 获取新闻动态 |
| 80 | `search()` | pub | 数据获取 | GET /api/search?keyword= 模糊搜索专辑和新闻 |

---

### `src/tui/mod.rs`

**目的**: TUI 入口，初始化终端交替屏幕 + 事件循环。

| 行号 | 函数 | 访问 | 类别 | 描述 |
|------|------|------|------|------|
| 19 | `run()` | pub | 初始化 | 启用交替屏幕 + 原始模式，创建事件循环按帧渲染 |

---

### `src/tui/app.rs`

**目的**: TUI 应用状态层，包装 Engine + 选中项/视图开关。

| 行号 | 函数 | 访问 | 类别 | 描述 |
|------|------|------|------|------|
| 14 | `new()` | pub | 初始化 | 创建 Engine 实例，默认隐藏帮助/歌词 |
| 25 | `update()` | pub | 帧更新 | 检测播放模式切换 + 调用 engine.update() |
| 38 | `play_selected()` | pub | 播放控制 | → engine.play_song_at(selected_song) |
| 43 | `toggle_pause()` | pub | 播放控制 | → engine.toggle_pause() |
| 48 | `next_album()` | pub | 专辑导航 | → engine.next_album()，选中归零 |
| 54 | `prev_album()` | pub | 专辑导航 | → engine.prev_album()，选中归零 |
| 59 | `song_count()` | private | 工具 | 根据视图返回歌曲数量（收藏/专辑） |
| 68 | `next_song()` | pub | 歌曲导航 | 选中下移，Single 模式则重播 |
| 80 | `prev_song()` | pub | 歌曲导航 | 选中上移，Single 模式则重播 |
| 92 | `cycle_mode()` | pub | 模式/音量 | → engine.cycle_mode() |
| 97 | `volume_up()` | pub | 模式/音量 | → engine.volume_up() |
| 102 | `volume_down()` | pub | 模式/音量 | → engine.volume_down() |
| 107 | `seek_forward()` | pub | 进度跳转 | → engine.seek_forward() |
| 112 | `seek_backward()` | pub | 进度跳转 | → engine.seek_backward() |
| 117 | `toggle_help()` | pub | 视图控制 | 切换帮助面板显示 |
| 122 | `toggle_lyrics()` | pub | 视图控制 | 切换歌词面板显示 |
| 127 | `toggle_love()` | pub | 收藏管理 | → engine.toggle_love() 切换当前选中歌曲收藏 |

---

### `src/tui/ui.rs`

**目的**: TUI 布局渲染：左右分栏、歌曲列表、歌词视图、状态栏。

| 行号 | 函数 | 访问 | 类别 | 描述 |
|------|------|------|------|------|
| 14 | `draw()` | pub | 布局渲染 | 主渲染入口：外层边框 + 左右分栏 |
| 34 | `draw_left()` | private | 布局渲染 | 左侧区域：信息栏 + 底部状态栏 |
| 45 | `draw_info()` | private | 布局渲染 | 左侧顶部：专辑简介 + 歌曲信息 + 进度条 |
| 100 | `draw_bottom()` | private | 布局渲染 | 左侧底部：播放状态/模式/音量/快捷键 |
| 170 | `draw_loved_view()` | private | 布局渲染 | 收藏歌曲列表视图 |
| 243 | `draw_right()` | private | 布局渲染 | 右侧分发：歌词/收藏/专辑歌曲 |
| 269 | `draw_album_info()` | private | 布局渲染 | 专辑名称 + 艺术家 + 页码 |
| 310 | `draw_song_list()` | private | 布局渲染 | 歌曲列表：选中标记/播放中高亮/收藏标记 |
| 372 | `draw_lyrics()` | private | 布局渲染 | 歌词视图：当前行青色加粗滚动跟随 |

---

### `src/tui/event.rs`

**目的**: TUI 键盘事件封装与按键分发。

| 行号 | 函数 | 访问 | 类别 | 描述 |
|------|------|------|------|------|
| 6 | `poll()` | pub | 键盘事件 | 轮询键盘事件（非阻塞） |
| 11 | `read()` | pub | 键盘事件 | 读取下一个事件 |
| 18 | `handle_key()` | pub | 键盘事件 | 将 crossterm KeyCode 映射到 App 方法，返回 false 则退出 |

---

### `src/origin_gui/mod.rs`

**目的**: GUI 入口，创建 frameless 透明 egui 窗口，加载 CJK 字体。

| 行号 | 函数 | 访问 | 类别 | 描述 |
|------|------|------|------|------|
| 8 | `run()` | pub | 初始化 | 创建 frameless 透明窗口，加载 CJK 字体，启动 egui |
| 37 | `clear_color()` | pub | 布局渲染 | eframe::App trait：透明窗口背景 |
| 42 | `update()` | pub | 帧更新 | eframe::App trait：驱动 GUI 每帧渲染 |
| 48 | `setup_cjk_fonts()` | private | 初始化 | 加载系统 Noto Sans CJK 中文字体 |
| 69 | `find_cjk_font()` | private | 初始化 | 按优先级搜索系统 CJK 字体文件 |

---

### `src/origin_gui/app.rs`

**目的**: GUI 应用状态层，包装 Engine + 选中项/视图开关（与 TUI app.rs 结构镜像）。

| 行号 | 函数 | 访问 | 类别 | 描述 |
|------|------|------|------|------|
| 13 | `new()` | pub | 初始化 | 创建 Engine 实例，默认隐藏帮助/歌词 |
| 24 | `update()` | pub | 帧更新 | 驱动引擎更新，检测收藏/普通列表切换 |
| 37 | `play_selected()` | pub | 播放控制 | → engine.play_song_at(selected_song) |
| 42 | `toggle_pause()` | pub | 播放控制 | → engine.toggle_pause() |
| 47 | `next_album()` | pub | 专辑导航 | → engine.next_album()，选中归零 |
| 53 | `prev_album()` | pub | 专辑导航 | → engine.prev_album()，选中归零 |
| 59 | `song_count()` | private | 工具 | 根据视图返回歌曲数量（收藏/专辑） |
| 68 | `next_song()` | pub | 歌曲导航 | 选中下移，Single 模式则重播 |
| 80 | `prev_song()` | pub | 歌曲导航 | 选中上移，Single 模式则重播 |
| 95 | `cycle_mode()` | pub | 模式/音量 | → engine.cycle_mode() |
| 100 | `volume_up()` | pub | 模式/音量 | → engine.volume_up() |
| 105 | `volume_down()` | pub | 模式/音量 | → engine.volume_down() |
| 110 | `seek_forward()` | pub | 进度跳转 | → engine.seek_forward() |
| 115 | `seek_backward()` | pub | 进度跳转 | → engine.seek_backward() |
| 120 | `toggle_help()` | pub | 视图控制 | 切换帮助面板显示/隐藏 |
| 125 | `toggle_lyrics()` | pub | 视图控制 | 切换歌词视图显示/隐藏 |
| 130 | `toggle_love()` | pub | 收藏管理 | → engine.toggle_love() 切换当前选中歌曲收藏 |

---

### `src/origin_gui/ui.rs`

**目的**: GUI 布局渲染：封面显示、左右分栏、歌曲列表、歌词视图。

| 行号 | 函数 | 访问 | 类别 | 描述 |
|------|------|------|------|------|
| 18 | `new()` | pub | 初始化 | 创建 GuiState：尺寸参考 + 封面缓存 |
| 28 | `load_cover()` | pub | 缓存 | 下载封面图片，解码为 egui ColorImage 纹理 |
| 49 | `check_cover()` | pub | 缓存 | 检测专辑切换，触发封面加载 |
| 62 | `update()` | pub | 帧更新 | 主帧函数：键盘处理 + 左右分栏布局 |
| 224 | `draw_cover()` | private | 布局渲染 | 渲染封面纹理或灰色占位正方形 |
| 249 | `draw_bottom()` | private | 布局渲染 | 底部信息栏：播放状态/模式/音量/帮助 |
| 325 | `draw_right()` | private | 布局渲染 | 右侧分发：歌词/收藏/专辑+歌曲 |
| 472 | `draw_lyrics()` | private | 布局渲染 | 歌词视图：当前行放大加粗（20pt） |
| 522 | `draw_loved_view()` | private | 布局渲染 | 收藏歌曲视图 |

---

## Summary

| 指标 | 数值 |
|------|------|
| **Total files** | 16 |
| **Total lines** | 2747 |

### File Categorization

**Entry point**:
- `src/main.rs` (22 lines) — Feature-gated entry dispatching to TUI or GUI

**Kernel files (shared)** — 8 files, 1204 lines:
- `src/lib.rs` (10 lines) — Library root, module declarations
- `src/error.rs` (24 lines) — Error types and Result alias
- `src/kernel.rs` (698 lines) — Playback engine (albums, songs, loved, lyrics, caching)
- `src/player.rs` (176 lines) — rodio audio player wrapper
- `src/ascii_art.rs` (28 lines) — Cover image to ASCII art converter
- `src/api/mod.rs` (2 lines) — API module declaration
- `src/api/types.rs` (176 lines) — API request/response data structures
- `src/api/client.rs` (90 lines) — HTTP client for Monster Siren API

**TUI files** — 4 files, 672 lines:
- `src/tui/mod.rs` (48 lines) — TUI entry, alternate screen, event loop
- `src/tui/app.rs` (144 lines) — TUI application state (Engine wrapper)
- `src/tui/ui.rs` (424 lines) — TUI layout rendering (ratatui)
- `src/tui/event.rs` (56 lines) — TUI keyboard event dispatch

**GUI files** — 3 files, 849 lines:
- `src/origin_gui/mod.rs` (87 lines) — GUI entry, frameless window, CJK fonts
- `src/origin_gui/app.rs` (144 lines) — GUI application state (Engine wrapper)
- `src/origin_gui/ui.rs` (618 lines) — GUI layout rendering (egui)

### Key Design Patterns

1. **Shared Kernel**: `kernel.rs` Engine + `player.rs` Player + `api/` are shared across TUI and GUI via `lib.rs`
2. **Mirror App Pattern**: Both `tui/app.rs` and `origin_gui/app.rs` provide identical methods that delegate to `Engine` — enabling keyboard shortcut parity
3. **Async Polling**: `Engine::update()` is called every frame from both frontends; it non-blockingly polls `Arc<Mutex<Option<Result<...>>>>` handles from background threads
4. **Preload Strategy**: `preload_adjacent()` and `preload_song_details()` proactively cache adjacent albums and song details in background threads
5. **Feature-gated Build**: `#[cfg(feature = "tui")]` / `#[cfg(feature = "gui")]` in `main.rs` enables building either or both frontends
