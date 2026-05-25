use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct Album {
    pub cid: String,
    pub name: String,
    #[serde(rename = "coverUrl")]
    pub cover_url: String,
    pub artistes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumDetail {
    pub cid: String,
    pub name: String,
    pub intro: String,
    pub belong: String,
    pub cover_url: String,
    pub cover_de_url: String,
    pub songs: Vec<AlbumSong>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlbumSong {
    pub cid: String,
    pub name: String,
    pub artistes: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub cid: String,
    pub name: String,
    pub album_cid: String,
    pub artists: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongDetail {
    pub cid: String,
    pub name: String,
    pub album_cid: String,
    pub source_url: String,
    pub lyric_url: Option<String>,
    pub mv_url: Option<String>,
    pub mv_cover_url: Option<String>,
    pub artists: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongsResponse {
    pub list: Vec<Song>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsItem {
    pub cid: String,
    pub title: String,
    pub cate: i32,
    pub date: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsResponse {
    pub list: Vec<NewsItem>,
    pub end: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchList<T> {
    pub list: Vec<T>,
    pub end: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub albums: SearchList<Album>,
    pub news: SearchList<NewsItem>,
}
