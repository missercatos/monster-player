//! 塞壬唱片 HTTP 客户端（参考实现）。
//!
//! 目标服务器 <https://monster-siren.hypergryph.com>，
//! 只依赖公开只读接口，用于契约验证与 fixture 生成。

use serde::de::DeserializeOwned;

use crate::error::{Error, Result};
use crate::types::*;

/// 塞壬唱片 API 默认 Base URL。
pub const DEFAULT_BASE_URL: &str = "https://monster-siren.hypergryph.com";

/// HTTP 客户端，封装对塞壬唱片 API 的所有请求。
pub struct Client {
    /// 服务器根地址（可指向自建网关做对照验证）
    base_url: String,
    /// ureq HTTP agent 实例，复用连接池
    agent: ureq::Agent,
}

impl Client {
    /// 使用默认 Base URL 创建客户端。
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_BASE_URL)
    }

    /// 使用自定义 Base URL 创建客户端。
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            agent: ureq::Agent::new_with_defaults(),
        }
    }

    /// 发送 GET 请求并解析统一响应包装。
    ///
    /// 先解析外层 `{code, msg, data}` 校验业务码，
    /// 再按 `T` 反序列化 `data`，避免错误响应被误报为字段缺失。
    fn get_json<T: DeserializeOwned>(&self, url: &str, query: Option<(&str, &str)>) -> Result<T> {
        let req = self.agent.get(url);
        let req = match query {
            Some((key, value)) => req.query(key, value),
            None => req,
        };
        let resp: ApiResponse<serde_json::Value> = req.call()?.body_mut().read_json()?;
        if resp.code != 0 {
            return Err(Error::Api {
                code: resp.code,
                msg: resp.msg,
            });
        }
        let data = resp
            .data
            .ok_or_else(|| Error::Missing(format!("empty data: {}", resp.msg)))?;
        Ok(serde_json::from_value(data)?)
    }

    /// GET /api/albums — 全量专辑列表。
    pub fn albums(&self) -> Result<Vec<Album>> {
        self.get_json(&format!("{}/api/albums", self.base_url), None)
    }

    /// GET /api/album/{cid}/detail — 专辑详情及歌曲列表。
    pub fn album_detail(&self, cid: &str) -> Result<AlbumDetail> {
        self.get_json(&format!("{}/api/album/{}/detail", self.base_url, cid), None)
    }

    /// GET /api/songs — 全量歌曲列表。
    pub fn songs(&self) -> Result<Vec<Song>> {
        let resp: SongsResponse = self.get_json(&format!("{}/api/songs", self.base_url), None)?;
        Ok(resp.list)
    }

    /// GET /api/song/{cid} — 歌曲详情（含音频直链与歌词链接）。
    pub fn song_detail(&self, cid: &str) -> Result<SongDetail> {
        self.get_json(&format!("{}/api/song/{}", self.base_url, cid), None)
    }

    /// GET /api/news — 新闻动态。
    pub fn news(&self) -> Result<Vec<NewsItem>> {
        let resp: NewsResponse = self.get_json(&format!("{}/api/news", self.base_url), None)?;
        Ok(resp.list)
    }

    /// GET /api/search?keyword= — 按关键词搜索专辑与新闻。
    pub fn search(&self, keyword: &str) -> Result<SearchResponse> {
        self.get_json(
            &format!("{}/api/search", self.base_url),
            Some(("keyword", keyword)),
        )
    }

    /// 下载歌曲的 LRC 歌词文本；歌曲无歌词时返回 `Error::Missing`。
    pub fn lyrics(&self, cid: &str) -> Result<String> {
        let detail = self.song_detail(cid)?;
        let url = detail
            .lyric_url
            .ok_or_else(|| Error::Missing(format!("song {cid} has no lyric")))?;
        Ok(self.agent.get(&url).call()?.body_mut().read_to_string()?)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
