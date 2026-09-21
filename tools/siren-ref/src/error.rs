//! 参考 CLI 的统一错误类型。

use thiserror::Error;

/// 塞壬参考客户端的错误类型。
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP 请求错误（网络、超时、非 2xx 状态码等）
    #[error("HTTP error: {0}")]
    Http(#[from] ureq::Error),

    /// 文件或内存 I/O 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// 上游 API 返回的业务错误，包含错误码与消息
    #[error("API error: code={code}, msg={msg}")]
    Api { code: i32, msg: String },

    /// 上游数据缺少必要内容（例如歌曲没有歌词）
    #[error("missing data: {0}")]
    Missing(String),
}

/// `Result` 别名。
pub type Result<T> = std::result::Result<T, Error>;
