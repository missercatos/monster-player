//! 塞壬唱片 API 参考实现（Rust 库）。
//!
//! 保留自旧版 monster-player 的 API 调用机制：`api::client` 负责 HTTP 调用，
//! `api::types` 定义响应结构，`error` 提供统一错误类型。
//! 任何 Rust 程序均可直接依赖本库调用塞壬 API，CLI（`siren`）只是其中一种消费方式。

pub mod api;
pub mod error;

pub use api::client::{Client, DEFAULT_BASE_URL};
pub use error::{Error, Result};
