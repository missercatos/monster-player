# GUI 大改造计划

## 阶段划分

### 阶段一：主题系统 + 设置弹窗 + 鼠标支持
### 阶段二：搜索图标 + 新搜索框 + 进度条重做
### 阶段三：Arknights 动画背景 + FFT 频谱
### 阶段四：Rutland 主题 + 破碎动画

---

## 阶段一：主题系统 + 设置弹窗 + 鼠标支持

### 1.1 主题系统

**新增文件**: `src/origin_gui/theme.rs`

```rust
pub enum ThemeName {
    Origin,    // 当前透明主题
    Tty,       // 终端黑白
    Tokyonight,// 深蓝+粉紫
    Arknights, // 深色金属+白色波纹
    Rutland,   // 黄蓝黑几何破碎
}

pub struct ThemeColors {
    pub bg_fill: Color32,           // 主面板背景
    pub bg_alpha: u8,              // 背景透明度
    pub border: Color32,           // 边框颜色
    pub text_primary: Color32,     // 主文字
    pub text_secondary: Color32,   // 次要文字
    pub accent: Color32,           // 强调色（当前播放、进度条）
    pub cursor: Color32,           // 选中项光标
    pub cursor_bg: Color32,        // 光标背景色
    pub loved: Color32,            // 收藏标记
    pub progress_bar: Color32,     // 进度条颜色
    pub search_icon: Color32,      // 搜索图标颜色
    pub search_bg: Color32,        // 搜索框背景
}
```

**各主题配色**：

| 主题 | bg_fill | bg_alpha | text_primary | accent | cursor | cursor_bg |
|------|---------|----------|--------------|--------|--------|-----------|
| Origin | rgba(0,0,0,120) | 120 | WHITE | cyan(0,200,200) | WHITE | rgba(0,200,200,60) |
| Tty | BLACK | 255 | GRAY(180) | WHITE | BLACK | WHITE |
| Tokyonight | rgba(26,27,38,240) | 240 | rgb(192,202,247) | rgb(122,162,247) | rgb(187,154,247) | rgba(122,162,247,80) |
| Arknights | rgba(15,15,20,250) | 250 | rgb(200,200,210) | rgb(0,180,255) | WHITE | rgba(0,180,255,60) |
| Rutland | rgba(240,240,240,255) | 255 | rgb(30,30,30) | rgb(0,80,200) | rgb(255,200,0) | rgba(255,200,0,60) |

**字体颜色特殊处理**：
- Tty: 光标为亮白色，其他文字灰色
- Arknights: 荧光蓝色字体，其他亮白/灰色
- Rutland: 天空蓝+银色+黑色

### 1.2 设置弹窗（Ctrl+T）

**新增文件**: `src/origin_gui/settings.rs`

弹窗规格：
- 正方形，边长 = 窗口默认大小(400) × 0.4 = 160px
- 居中显示，半透明背景跟随当前主题
- 左右比例 2:8（左边 32px，右边 128px）

**左侧导航栏**（jk 上下移动）：
```
┌────────┬──────────────────────────┐
│ > 项目简介 │                          │
│   快捷键   │   [右侧内容区域]          │
│   主题设置 │                          │
│   额外功能 │                          │
└────────┴──────────────────────────┘
```

**右侧内容**：

1. **项目简介**：
   - 项目名：monster-player
   - 内核：rodio + 自定义缓冲流
   - GitHub：github.com/missercatos/monster-player

2. **快捷键信息**（两列对齐）：
   ```
   Ctrl+T    设置/帮助
   h/l       上/下专辑
   j/k       上/下歌曲
   Space     播放选中
   x         暂停/恢复
   e         切换模式
   o/p       音量-
   s         收藏歌曲
   v         歌词显示
   A/D       快退/快进
   /         搜索
   ```

3. **主题设置**：
   - 主题 (h/l 切换)：[origin|tty|tokyonight|arknights|rutland]
   - 歌词动画 (h/l 切换)：[字体放大|颜色变化]

4. **额外功能**：
   - 下载开关 (h/l 切换)：[on|off]

**交互**：
- j/k：上下移动选中项
- Tab：切换左右面板焦点
- Escape：关闭弹窗

### 1.3 鼠标支持

**修改文件**: `src/origin_gui/ui.rs`

需要添加 `ctx.input(|i| i.pointer.any_click())` 事件处理：

| 点击区域 | 动作 |
|---------|------|
| 左侧封面区域 | 暂停/恢复 |
| 左侧进度条 | 拖动跳转 |
| 左侧底部状态栏 | 根据位置触发：模式切换、音量调节 |
| 右侧歌曲列表某行 | 播放该歌曲 |
| 右侧歌曲列表右键 | 收藏/取消收藏当前歌曲 |
| 右上角搜索图标 | 展开搜索框 |
| 上一曲/下一曲按钮 | 切歌 |
| 歌词区域 | 切换歌词显示 |

**鼠标悬停效果**：
- 歌曲列表行悬停时高亮
- 按钮悬停时变色

### 1.4 搜索图标

在右上角渲染一个自定义搜索图标（不用 emoji）：

```
搜索图标 = 圆形 + 斜线（放大镜）
```

用 `painter().circle()` + `painter().line_segment()` 手动绘制。

---

## 阶段二：搜索图标 + 新搜索框 + 进度条重做

### 2.1 新搜索框

从顶部弹出的白色框，与 TUI 不同：

```
┌──────────────────────────┐
│  🔍  [搜索关键词...]      │  ← 顶部弹出框
├──────────────────────────┤
│  > Song A - Artist       │
│    Song B - Artist       │
│    Song C - Artist       │
└──────────────────────────┘
```

- 动画：从顶部滑入（使用 egui 动画 API）
- 背景跟随当前主题
- 搜索结果点击可直接播放

### 2.2 进度条重做

参考图片3.png 的样式：
- 白色细线 + 圆点指示器
- 时间显示在进度条上方
- 支持鼠标拖动跳转

---

## 阶段三：Arknights 动画背景 + FFT 频谱

### 3.1 FFT 频谱分析

**新增依赖**: `rustfft = "1.0"`

**修改文件**: `src/player.rs`

在 Player 中添加频谱分析：
- 从 rodio Sink 的音频流中采样
- 应用 FFT 变换
- 返回频率带的幅度值（如 8 个频段）

```rust
pub struct Spectrum {
    pub bands: [f32; 8],  // 8 个频段的幅度 0.0-1.0
}
```

**修改文件**: `src/kernel.rs`

Engine 新增字段：
```rust
pub spectrum: [f32; 8],  // 当前音频频谱
```

在 `update()` 中从 Player 获取频谱数据。

### 3.2 白色波纹动画

参考图片1.png：
- 多条白色曲线从左到右
- 使用正弦波叠加 + 噪声
- 振幅和速度随频谱数据变化
- 使用 `painter().line_segment()` 渲染

### 3.3 金属纹理背景

参考图片2.png：
- 深色渐变背景
- 叠加半透明网格/点阵图案
- 从上到下的黑色→金属灰渐变

---

## 阶段四：Rutland 主题 + 破碎动画

### 4.1 破碎背景

- 基础背景：蓝白黄黑几何图案拼接
- 破碎动画：每隔几十秒触发
  1. 背景突然变成纯白
  2. 几何碎片四散飞开
  3. 碎片重新聚合回原位
- 使用 egui 动画 API 控制帧间插值

### 4.2 字体颜色

- 天空蓝：主文字
- 银色：次要文字
- 黑色：强调文字

---

## 涉及的文件修改

| 文件 | 阶段 | 修改内容 |
|------|------|---------|
| `src/origin_gui/theme.rs` | 1 | 新增：主题定义和颜色方案 |
| `src/origin_gui/settings.rs` | 1 | 新增：设置弹窗渲染和交互 |
| `src/origin_gui/app.rs` | 1 | 修改：新增主题状态、设置弹窗状态、鼠标交互方法 |
| `src/origin_gui/ui.rs` | 1-4 | 修改：主题应用、鼠标事件、搜索框、进度条、动画 |
| `src/origin_gui/mod.rs` | 1 | 修改：主题初始化 |
| `src/player.rs` | 3 | 修改：新增频谱分析 |
| `src/kernel.rs` | 3 | 修改：新增 spectrum 字段 |
| `Cargo.toml` | 3 | 修改：新增 rustfft 依赖 |
