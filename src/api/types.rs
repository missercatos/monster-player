use serde::Deserialize;

/// 通用 API 响应包装
///
/// `T` 为具体数据类型（如 `Vec<Album>`、`AlbumDetail`、`SearchResponse` 等）。
/// 所有接口返回的统一外层结构。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    /// 状态码，0 表示成功
    pub code: i32,
    /// 状态描述信息
    pub msg: String,
    /// 响应数据体
    pub data: T,
}

/// 专辑列表项
///
/// 用于 `/api/albums` 接口返回的专辑摘要信息。
#[derive(Debug, Deserialize)]
pub struct Album {
    /// 专辑唯一标识
    pub cid: String,
    /// 专辑名称
    pub name: String,
    /// 专辑封面图片 URL
    #[serde(rename = "coverUrl")]
    pub cover_url: String,
    /// 艺人/艺术家列表
    pub artistes: Vec<String>,
}

/// 专辑详情（含歌曲列表）
///
/// 用于 `/api/album/{cid}/detail` 接口。
/// 相比 `Album` 增加了专辑介绍、所属分类、封面主图等字段以及内含歌曲列表。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumDetail {
    /// 专辑唯一标识
    pub cid: String,
    /// 专辑名称
    pub name: String,
    /// 专辑简介
    pub intro: String,
    /// 所属分类/归属
    pub belong: String,
    /// 专辑封面 URL（方形）
    pub cover_url: String,
    /// 专辑封面 URL（横版 / decoration）
    pub cover_de_url: String,
    /// 专辑内歌曲列表
    pub songs: Vec<AlbumSong>,
}

/// 专辑内歌曲摘要
///
/// 嵌套在 `AlbumDetail.songs` 中，
/// 仅包含歌曲基本标识信息，不包含音源直链。
#[derive(Debug, Clone, Deserialize)]
pub struct AlbumSong {
    /// 歌曲唯一标识
    pub cid: String,
    /// 歌曲名称
    pub name: String,
    /// 艺人/艺术家列表
    pub artistes: Vec<String>,
}

/// 歌曲列表项
///
/// 用于 `/api/songs` 接口（全量歌曲列表），
/// 仅含基本元信息，不含音源地址。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    /// 歌曲唯一标识
    pub cid: String,
    /// 歌曲名称
    pub name: String,
    /// 所属专辑 cid
    pub album_cid: String,
    /// 艺人/艺术家列表
    pub artists: Vec<String>,
}

/// 歌曲详情（含音频直链 sourceUrl + 歌词链接 lyricUrl）
///
/// 用于 `/api/song/{cid}` 接口。
/// 包含可直接播放的 WAV 音频源 URL、歌词文件 URL 及可选的 MV 资源。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongDetail {
    /// 歌曲唯一标识
    pub cid: String,
    /// 歌曲名称
    pub name: String,
    /// 所属专辑 cid
    pub album_cid: String,
    /// 音频源直链（WAV 格式）
    pub source_url: String,
    /// 歌词文件 URL（LRC 格式），可能为空
    pub lyric_url: Option<String>,
    /// MV 视频 URL，可能为空
    pub mv_url: Option<String>,
    /// MV 封面图 URL，可能为空
    pub mv_cover_url: Option<String>,
    /// 艺人/艺术家列表
    pub artists: Vec<String>,
}

/// 歌曲列表接口的响应数据体
///
/// `/api/songs` 接口返回的内层包装结构。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongsResponse {
    /// 歌曲摘要列表
    pub list: Vec<Song>,
}

/// 新闻动态条目
///
/// 用于 `/api/news` 接口返回的新闻摘要信息。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsItem {
    /// 新闻唯一标识
    pub cid: String,
    /// 新闻标题
    pub title: String,
    /// 新闻分类
    pub cate: i32,
    /// 发布日期
    pub date: String,
}

/// 新闻列表接口的响应数据体
///
/// `/api/news` 接口返回的内层包装结构。
/// 支持分页判断。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsResponse {
    /// 新闻条目列表
    pub list: Vec<NewsItem>,
    /// 是否已到末页，`true` 表示无更多数据
    pub end: bool,
}

/// 搜索结果通用分页列表
///
/// `T` 为具体条目类型（`Album` 或 `NewsItem`）。
/// 用于 `/api/search` 接口中的内层嵌套数据结构。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchList<T> {
    /// 搜索结果列表
    pub list: Vec<T>,
    /// 是否已到末页，`true` 表示无更多数据
    pub end: bool,
}

/// 搜索接口的响应数据体
///
/// `/api/search?keyword=` 接口返回的内层包装结构。
/// 同时包含专辑和新闻两类搜索结果。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    /// 匹配到的专辑搜索结果
    pub albums: SearchList<Album>,
    /// 匹配到的新闻搜索结果
    pub news: SearchList<NewsItem>,
}
