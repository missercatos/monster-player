# 🎵 msplayer

> **A Monster Siren Records streaming client written in Rust** — shared kernel driving multiple UIs.

[English](#english) | [中文](#中文) | [日本語](#日本語)

---

# English

## Overview

**msplayer** is an unofficial desktop music player for [Monster Siren Records](https://monster-siren.hypergryph.com), the label behind Arknights music. It features a shared-kernel architecture — all playback logic, data caching, and API interaction live in one core engine, with pluggable frontends (TUI and GUI).

## Features

| Feature | Description |
|---------|-------------|
| 🎵 **Streaming Playback** | Progressive download with 8MB buffer — songs start playing before the full file is downloaded |
| 🖥️ **Terminal UI (TUI)** | Full ratatui interface with keyboard-only navigation (vim-style hjkl) |
| 🪟 **Desktop GUI** | eframe/egui transparent overlay window with custom title bar, play controls, and search |
| ❤️ **Favorites** | Toggle love on any song with `s` — persisted to `~/.config/msplayer/loved.json` |
| 🔍 **Search** | Type `/` to open a Spotlight-style search popup — search across all albums |
| 🎤 **Synced Lyrics** | LRC lyric parsing with real-time highlighting as the song plays |
| 🔀 **Play Modes** | Album List / Album Random / Global List / Global Random / Single / Love List / Love Random |
| 🌍 **Cross-platform** | Linux, Windows, macOS — auto-detects system CJK fonts |
| 🎨 **3 Themes** | Origin (dark cyan), TTY (monochrome), Tokyonight (blue-purple) |

## Screenshots

| TUI - Main | TUI - Lyrics |
|------------|-------------|
| ![TUI Main](introduce/TUI-origin.png) | ![TUI Lyrics](introduce/TUI-origin-1.png) |

### GUI (Origin Theme)

![GUI](introduce/GUI-origin.png)

## Installation

### Prerequisites
- **Rust** toolchain 1.81+
- Audio output device

### Build from Source

```bash
git clone https://github.com/your-username/monster-player.git
cd monster-player

# TUI (default)
cargo build --release

# GUI
cargo build --release --features gui

# Run
cargo run --release
```

## Usage

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Space` | Play selected song |
| `x` | Pause / Resume |
| `h` / `l` or `←` / `→` | Previous / Next album |
| `j` / `k` or `↓` / `↑` | Previous / Next song (browse) |
| `Shift+A` / `Shift+D` | Skip to previous / next song (play immediately) |
| `a` / `d` | Seek backward / forward |
| `e` | Cycle play mode |
| `o` / `p` | Volume down / up |
| `v` | Toggle lyrics view |
| `s` | Toggle love (favorite) |
| `Ctrl+T` | Settings / Help |
| `/` | Search |
| `Esc` | Exit search / Close popup |

### Mouse Controls (GUI only)
- **Scroll** in right panel → browse songs
- **Click** play mode text → cycle modes
- **Click** `<` `||`/`>` `>` buttons → skip / pause / play
- **Drag** progress bar → seek
- **Click** search icon (top-right) → search popup
- **Double-click** search result → jump to song

## Architecture

```
┌─────────────────────────────────────┐
│              UI Layer               │
│  ┌─────────┐  ┌──────────────────┐  │
│  │   TUI   │  │       GUI        │  │
│  │ ratatui │  │  eframe / egui   │  │
│  └────┬────┘  └───────┬──────────┘  │
│       │               │             │
├───────┴───────────────┴─────────────┤
│           Shared Kernel             │
│  ┌─────────┐  ┌────────┐  ┌──────┐ │
│  │  engine │  │ player │  │ api  │ │
│  │  data   │  │ audio  │  │HTTP  │ │
│  └─────────┘  └────────┘  └──────┘ │
└─────────────────────────────────────┘
```

## Project Structure

```
src/
├── lib.rs              Library entry
├── main.rs             Binary entry (feature dispatch)
├── kernel.rs           Core engine (state, cache, streaming, lyrics)
├── player.rs           Audio player (rodio backend)
├── error.rs            Error types
├── api/
│   ├── mod.rs
│   ├── types.rs        API response types
│   └── client.rs       HTTP client (ureq)
├── tui/                Terminal UI
│   ├── mod.rs          crossterm init + event loop
│   ├── app.rs          UI state shell
│   ├── event.rs        Keyboard event mapping
│   └── ui.rs           Layout + rendering
└── origin_gui/         Desktop GUI
    ├── mod.rs          Frameless transparent window
    ├── app.rs          GUI state
    ├── ui.rs           Layout + rendering
    ├── theme.rs        Theme system (3 themes)
    └── settings.rs     Settings popup
```

## Roadmap

- [x]  TUI player — full keyboard operation
- [x]  GUI player — transparent window, custom title bar, search popup
- [ ]  Windows .exe installer — NSIS / WiX
- [ ]  Android port — cross-compile + Native Activity
- [ ]  Linux package — AUR / deb / rpm
- [ ]  More themes

## Credits

Music content powered by [Monster Siren Records](https://monster-siren.hypergryph.com) / Hypergryph.

*This is an unofficial community project, not affiliated with Hypergryph.*

---

# 中文

## 概述

**msplayer** 是一款非官方的 [塞壬唱片](https://monster-siren.hypergryph.com) 桌面音乐播放器，采用共享内核架构 — 所有播放逻辑、数据缓存和 API 交互集中在核心引擎中，UI 层可插拔替换（TUI 和 GUI）。

## 功能介绍

| 功能 | 说明 |
|------|------|
| 🎵 **流式播放** | 渐进式下载，8MB 缓冲 — 歌曲在完整下载完成前即可开始播放 |
| 🖥️ **终端界面 (TUI)** | 全 ratatui 界面，纯键盘操作（vim 风格 hjkl） |
| 🪟 **桌面 GUI** | eframe/egui 透明悬浮窗口，自定义标题栏、播放控件和搜索 |
| ❤️ **收藏系统** | 按 `s` 收藏/取消收藏歌曲，持久化到 `~/.config/msplayer/loved.json` |
| 🔍 **搜索** | 按 `/` 打开 Spotlight 风格搜索弹窗，跨专辑搜索 |
| 🎤 **同步歌词** | LRC 歌词解析，播放时实时高亮当前歌词行 |
| 🔀 **播放模式** | 专辑列表 / 专辑随机 / 全局列表 / 全局随机 / 单曲循环 / 收藏列表 / 收藏随机 |
| 🌍 **跨平台** | Linux、Windows、macOS — 自动检测系统 CJK 字体 |
| 🎨 **3 套主题** | Origin（暗色青）、TTY（黑白）、Tokyonight（蓝紫） |

## 截图

请参阅上方的 [English Screenshots](#screenshots)。

## 安装

### 环境要求
- **Rust** 工具链 1.81+
- 音频输出设备

### 从源码构建

```bash
git clone https://github.com/your-username/monster-player.git
cd monster-player

# TUI (默认)
cargo build --release

# GUI
cargo build --release --features gui
```

## 快捷键

| 按键 | 功能 |
|------|------|
| `Space` | 播放选中歌曲 |
| `x` | 暂停/恢复 |
| `h`/`l` 或 `←`/`→` | 上/下专辑 |
| `j`/`k` 或 `↓`/`↑` | 上/下歌曲（浏览模式） |
| `Shift+A`/`Shift+D` | 上一首/下一首（立即播放） |
| `a`/`d` | 进度后退/前进 |
| `e` | 切换播放模式 |
| `o`/`p` | 音量减/增 |
| `v` | 歌词显示切换 |
| `s` | 收藏/取消收藏 |
| `Ctrl+T` | 设置/帮助 |
| `/` | 搜索 |

### 鼠标操作（仅 GUI）
- **滚轮** 在右侧面板 → 浏览歌曲
- **点击** 播放模式文字 → 切换模式
- **点击** `<` `||`/`>` `>` 按钮 → 切歌/暂停/播放
- **拖拽** 进度条 → 跳转进度
- **点击** 搜索图标（右上角）→ 搜索弹窗
- **双击** 搜索结果 → 跳转

## 架构

请参阅上方的 [English Architecture](#architecture)。

## 致谢

音乐内容由 [塞壬唱片 (Monster Siren Records)](https://monster-siren.hypergryph.com) / 鹰角网络提供。

*本项目为社区开发的非官方客户端，与鹰角网络无附属关系。*

---

# 日本語

## 概要

**msplayer** は、[Monster Siren Records](https://monster-siren.hypergryph.com) の非公式デスクトップ音楽プレイヤーです。共有カーネルアーキテクチャを採用し、再生ロジック、データキャッシュ、API通信はすべてコアエンジンに集約。UIはプラグイン可能（TUI と GUI）。

## 機能

| 機能 | 説明 |
|------|------|
| 🎵 **ストリーミング再生** | プログレッシブダウンロード、8MBバッファ — ダウンロード完了前に再生開始 |
| 🖥️ **端末 UI (TUI)** | ratatui による全画面インターフェース、キーボードのみで操作（vim 風 hjkl） |
| 🪟 **デスクトップ GUI** | eframe/egui 透明オーバーレイウィンドウ、カスタムタイトルバー、再生コントロール、検索 |
| ❤️ **お気に入り** | `s` キーで楽曲をお気に入り登録 — `~/.config/msplayer/loved.json` に永続化 |
| 🔍 **検索** | `/` キーで Spotlight 風の検索ポップアップ — 全アルバム横断検索 |
| 🎤 **同期歌詞** | LRC 歌詞解析、再生位置に合わせてリアルタイムハイライト |
| 🔀 **再生モード** | アルバム順 / アルバムランダム / 全曲順 / 全曲ランダム / 単曲リピート / お気に入り順 / お気に入りランダム |
| 🌍 **クロスプラットフォーム** | Linux、Windows、macOS — システムの CJK フォントを自動検出 |
| 🎨 **3 テーマ** | Origin（ダークシアン）、TTY（モノクロ）、Tokyonight（青紫） |

## スクリーンショット

上記 [English Screenshots](#screenshots) を参照してください。

## インストール

### 必要条件
- **Rust** ツールチェーン 1.81+
- オーディオ出力デバイス

### ソースからビルド

```bash
git clone https://github.com/your-username/monster-player.git
cd monster-player

# TUI (デフォルト)
cargo build --release

# GUI
cargo build --release --features gui
```

## ショートカット

| キー | 操作 |
|------|------|
| `Space` | 選択曲を再生 |
| `x` | 一時停止 / 再開 |
| `h` / `l` または `←` / `→` | 前 / 次のアルバム |
| `j` / `k` または `↓` / `↑` | 前 / 次の曲（閲覧モード） |
| `Shift+A` / `Shift+D` | 前 / 次の曲へスキップ（即時再生） |
| `a` / `d` | シーク（戻る / 進む） |
| `e` | 再生モード切替 |
| `o` / `p` | 音量 下 / 上 |
| `v` | 歌詞表示切替 |
| `s` | お気に入り切替 |
| `Ctrl+T` | 設定 / ヘルプ |
| `/` | 検索 |

### マウス操作（GUI のみ）
- **右パネルでスクロール** → 曲の閲覧
- **再生モードテキストをクリック** → モード切替
- **`<` `||`/`>` `>` ボタンをクリック** → スキップ / 一時停止 / 再生
- **プログレスバーをドラッグ** → シーク
- **検索アイコン（右上）をクリック** → 検索ポップアップ
- **検索結果をダブルクリック** → 曲へジャンプ

## アーキテクチャ

上記 [English Architecture](#architecture) を参照してください。

## クレジット

音楽コンテンツは [Monster Siren Records](https://monster-siren.hypergryph.com) / Hypergryph により提供されています。

*本プロジェクトはコミュニティ開発の非公式クライアントであり、Hypergryph とは無関係です。*
