/// 外部 API 交互（请求、响应类型定义、HTTP 调用）
pub mod api;
/// 声明式 TOML 配置系统
pub mod config;
/// 错误类型与 Result 别名
pub mod error;
/// C FFI 绑定层（跨语言接口）
pub mod ffi;
/// 内核逻辑：数据库、配置、搜索、同步等核心功能
pub mod kernel;
/// 音频播放器封装（播放/暂停/进度/音量控制）
pub mod player;
