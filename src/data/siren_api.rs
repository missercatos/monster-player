use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const BASE_URL: &str = "https://monster-siren.hypergryph.com/api";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BaseResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Album {
    pub cid: String,
    pub name: String,
    #[serde(rename = "coverUrl")]
    pub cover_url: String,
    pub artistes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlbumDetail {
    pub cid: String,
    pub name: String,
    pub intro: String,
    pub belong: String,
    #[serde(rename = "coverUrl")]
    pub cover_url: String,
    #[serde(rename = "coverDeUrl")]
    pub cover_de_url: String,
    pub songs: Vec<AlbumSong>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlbumSong {
    pub cid: String,
    pub name: String,
    pub artistes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Song {
    pub cid: String,
    pub name: String,
    #[serde(rename = "albumCid")]
    pub album_cid: String,
    pub artists: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SongsData {
    pub list: Vec<Song>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SongDetail {
    pub cid: String,
    pub name: String,
    #[serde(rename = "albumCid")]
    pub album_cid: String,
    #[serde(rename = "sourceUrl")]
    pub source_url: String,
    #[serde(rename = "lyricUrl")]
    pub lyric_url: Option<String>,
    #[serde(rename = "mvUrl")]
    pub mv_url: Option<String>,
    #[serde(rename = "mvCoverUrl")]
    pub mv_cover_url: Option<String>,
    pub artists: Vec<String>,
}

#[derive(Debug)]
pub struct Client {
    base_url: String,
}

impl Client {
    pub fn new() -> Self {
        Self {
            base_url: BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(base_url: String) -> Self {
        Self { base_url }
    }

    fn do_request<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let resp = ureq::get(&url)
            .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
            .set("Accept", "application/json")
            .call()
            .map_err(|e| anyhow!("请求失败: {}", e))?;
        
        if resp.status() != 200 {
            return Err(anyhow!("HTTP错误: {}", resp.status()));
        }

        let base_resp: BaseResponse<T> = resp.into_json()
            .map_err(|e| anyhow!("解析JSON失败: {}", e))?;
        
        if base_resp.code != 0 {
            return Err(anyhow!("API错误 {}: {}", base_resp.code, base_resp.msg));
        }

        Ok(base_resp.data)
    }

    pub fn get_albums(&self) -> Result<Vec<Album>> {
        self.do_request("/albums")
    }

    pub fn get_album_detail(&self, cid: &str) -> Result<AlbumDetail> {
        self.do_request(&format!("/album/{}", cid))
    }

    pub fn get_songs(&self) -> Result<Vec<Song>> {
        let data: SongsData = self.do_request("/songs")?;
        Ok(data.list)
    }

    pub fn get_song_detail(&self, cid: &str) -> Result<SongDetail> {
        self.do_request(&format!("/song/{}", cid))
    }

    pub fn get_lyric(&self, lyric_url: &str) -> Result<String> {
        let resp = ureq::get(lyric_url)
            .set("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
            .call()
            .map_err(|e| anyhow!("歌词请求失败: {}", e))?;
        
        if resp.status() != 200 {
            return Err(anyhow!("HTTP错误: {}", resp.status()));
        }

        resp.into_string()
            .map_err(|e| anyhow!("读取歌词失败: {}", e))
    }
}