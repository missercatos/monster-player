use thiserror::Error;

/// 库内部统一错误类型。
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP 请求错误（网络、超时、非 2xx 状态码等）
    #[error("HTTP error: {0}")]
    Http(#[from] ureq::Error),

    /// 文件或内存 I/O 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 音频解码/播放错误，携带描述字符串
    #[error("Audio error: {0}")]
    Audio(String),

    /// 上游 API 返回的业务错误，包含错误码与消息
    #[error("API error: code={code}, msg={msg}")]
    Api { code: i32, msg: String },
}

/// 库内部统一 `Result` 别名。
pub type Result<T> = std::result::Result<T, Error>;
