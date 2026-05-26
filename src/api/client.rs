use crate::api::types::*;
use crate::error::Result;

/// HTTP 客户端，目标服务器 monster-siren.hypergryph.com
///
/// 封装了对塞壬唱片 API 的所有 HTTP 请求。
/// 内部使用 `ureq` 同步 HTTP client。
pub struct Client {
    /// 服务器根地址
    base_url: String,
    /// ureq HTTP agent 实例，复用连接池
    agent: ureq::Agent,
}

impl Client {
    /// 创建 HTTP 客户端，目标服务器 monster-siren.hypergryph.com
    ///
    /// 初始化 ureq agent 并配置基础 URL。
    pub fn new() -> Self {
        Self {
            base_url: "https://monster-siren.hypergryph.com".into(),
            agent: ureq::Agent::new_with_defaults(),
        }
    }

    /// GET /api/albums 获取全量专辑列表
    ///
    /// 返回所有专辑的摘要信息列表。
    pub fn albums(&self) -> Result<Vec<Album>> {
        let resp: ApiResponse<Vec<Album>> =
            self.agent.get(&format!("{}/api/albums", self.base_url)).call()?.body_mut().read_json()?;
        Ok(resp.data)
    }

    /// GET /api/album/{cid}/detail 获取专辑详情+歌曲列表
    ///
    /// `cid` 为专辑唯一标识，返回该专辑的完整信息及其包含的所有歌曲。
    pub fn album_detail(&self, cid: &str) -> Result<AlbumDetail> {
        let resp: ApiResponse<AlbumDetail> = self
            .agent
            .get(&format!("{}/api/album/{}/detail", self.base_url, cid))
            .call()?
            .body_mut()
            .read_json()?;
        Ok(resp.data)
    }

    /// GET /api/songs 获取全量歌曲列表
    ///
    /// 返回所有歌曲的摘要信息（不含音源直链）。
    pub fn songs(&self) -> Result<Vec<Song>> {
        let resp: ApiResponse<SongsResponse> =
            self.agent.get(&format!("{}/api/songs", self.base_url)).call()?.body_mut().read_json()?;
        Ok(resp.data.list)
    }

    /// GET /api/song/{cid} 获取歌曲详情（含 WAV 直链）
    ///
    /// `cid` 为歌曲唯一标识，
    /// 返回包含 `source_url`（WAV 音频直链）、`lyric_url`（LRC 歌词）及可选 MV 资源的完整歌曲信息。
    pub fn song_detail(&self, cid: &str) -> Result<SongDetail> {
        let resp: ApiResponse<SongDetail> =
            self.agent.get(&format!("{}/api/song/{}", self.base_url, cid)).call()?.body_mut().read_json()?;
        Ok(resp.data)
    }

    /// GET /api/news 获取新闻动态
    ///
    /// 返回新闻动态条目列表。
    pub fn news(&self) -> Result<Vec<NewsItem>> {
        let resp: ApiResponse<NewsResponse> =
            self.agent.get(&format!("{}/api/news", self.base_url)).call()?.body_mut().read_json()?;
        Ok(resp.data.list)
    }

    /// GET /api/search?keyword= 模糊搜索专辑和新闻
    ///
    /// `keyword` 为搜索关键词，同时匹配专辑名称和新闻标题，
    /// 返回分组的搜索结果。
    pub fn search(&self, keyword: &str) -> Result<SearchResponse> {
        let resp: ApiResponse<SearchResponse> = self
            .agent
            .get(&format!("{}/api/search", self.base_url))
            .query("keyword", keyword)
            .call()?
            .body_mut()
            .read_json()?;
        Ok(resp.data)
    }
}
